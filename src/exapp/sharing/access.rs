use super::{Failure, Shared, failure};
use crate::{
    library::{Collection, Grant, Recipient, Resource, Store},
    membership::{Membership, Role},
};

pub(super) async fn locked<T: Send + 'static>(
    app: Shared,
    action: impl FnOnce(&Membership, &mut Store) -> Result<T, &'static str> + Send + 'static,
) -> Result<T, Failure> {
    tokio::task::spawn_blocking(move || {
        let members = app
            .membership
            .lock()
            .map_err(|_| "library_storage_failed")?;
        let mut store = app.library.lock().map_err(|_| "library_storage_failed")?;
        action(&members, &mut store)
    })
    .await
    .map_err(|_| failure("library_storage_failed"))?
    .map_err(failure)
}
pub(super) fn owner(members: &Membership, home: &str, actor: &str) -> Result<(), &'static str> {
    if members.role(home, actor) == Some(Role::Owner) {
        Ok(())
    } else {
        Err("share_owner_required")
    }
}
#[derive(Clone)]
pub(super) struct Context {
    pub(super) home: String,
    pub(super) owner: String,
    pub(super) grant: Grant,
    pub(super) collection: Option<Collection>,
    pub(super) pending: bool,
    pub(super) recipient_label: String,
}
pub(super) fn context(
    members: &Membership,
    store: &Store,
    home: &str,
    id: &str,
    actor: &str,
    allow_pending: bool,
) -> Result<Context, &'static str> {
    let data = store.home(home);
    let grant = data.shares.get(id).cloned().ok_or("share_not_found")?;
    let source_owner = members.owner(home).ok_or("share_not_found")?;
    let (recipient_allowed, pending, label) = match &grant.recipient {
        Recipient::Account { user } => (user == actor, false, user.clone()),
        Recipient::HouseholdInvitation { user } => (user == actor, true, user.clone()),
        Recipient::Household { home_id } => (
            members.role(home_id, actor).is_some(),
            false,
            members.name(home_id).unwrap_or("").to_owned(),
        ),
    };
    if source_owner != actor && (!recipient_allowed || (pending && !allow_pending)) {
        return Err("share_not_found");
    }
    let collection = match &grant.resource {
        Resource::Collection { collection_id } => Some(
            data.collections
                .get(collection_id)
                .cloned()
                .ok_or("share_not_found")?,
        ),
        Resource::Recipe { .. } => None,
    };
    Ok(Context {
        home: home.to_owned(),
        owner: source_owner.to_owned(),
        grant,
        collection,
        pending,
        recipient_label: label,
    })
}
pub(super) async fn ensure(app: Shared, expected: Context, actor: String) -> Result<(), Failure> {
    locked(app, move |m, s| {
        let current = context(m, s, &expected.home, &expected.grant.id, &actor, false)?;
        if current.grant != expected.grant || current.collection != expected.collection {
            return Err("share_changed");
        }
        Ok(())
    })
    .await
}
pub(super) fn addressed(m: &Membership, recipient: &Recipient, actor: &str) -> bool {
    match recipient {
        Recipient::Account { user } | Recipient::HouseholdInvitation { user } => user == actor,
        Recipient::Household { home_id } => m.role(home_id, actor).is_some(),
    }
}
