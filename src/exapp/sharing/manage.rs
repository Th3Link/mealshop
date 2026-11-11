use super::{
    Failure, Shared,
    access::{self, locked, owner},
    response,
};
use crate::{
    exapp::{input, user},
    library::{Recipient, Resource},
    membership::Role,
};
use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    response::Response,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ShareFilter {
    kind: Option<String>,
    resource_id: Option<String>,
    page: usize,
    per_page: usize,
}
impl Default for ShareFilter {
    fn default() -> Self {
        Self {
            kind: None,
            resource_id: None,
            page: 1,
            per_page: 50,
        }
    }
}

pub(super) async fn list(
    State(app): State<Shared>,
    Path(home): Path<String>,
    request: Request,
) -> Result<Response, Failure> {
    let actor = user(&app, &request)?;
    let Query(filter) = Query::<ShareFilter>::try_from_uri(request.uri())
        .map_err(|_| Failure(StatusCode::BAD_REQUEST, "invalid_page"))?;
    if filter.page == 0
        || !(1..=50).contains(&filter.per_page)
        || filter.page.checked_mul(filter.per_page).is_none()
        || filter.kind.is_some() != filter.resource_id.is_some()
        || filter
            .kind
            .as_ref()
            .is_some_and(|k| !matches!(k.as_str(), "recipe" | "collection"))
    {
        return Err(Failure(StatusCode::BAD_REQUEST, "invalid_page"));
    }
    let value = locked(app, move |m, s| {
        owner(m, &home, &actor)?;
        let data = s.home(&home);
        let shares: Vec<_> = data
            .shares
            .values()
            .filter(|g| match (&filter.kind, &filter.resource_id, &g.resource) {
                (None, None, _) => true,
                (Some(kind), Some(id), Resource::Recipe { recipe_id, .. }) => {
                    kind == "recipe" && id == recipe_id
                }
                (Some(kind), Some(id), Resource::Collection { collection_id }) => {
                    kind == "collection" && id == collection_id
                }
                _ => false,
            })
            .map(|g| {
                let label = match &g.recipient {
                    Recipient::Account { user } | Recipient::HouseholdInvitation { user } => {
                        user.as_str()
                    }
                    Recipient::Household { home_id } => m.name(home_id).unwrap_or(""),
                };
                json!({"grant":g,"recipient_label":label})
            })
            .collect();
        let total = shares.len();
        let shares: Vec<_> = shares.into_iter()
            .skip((filter.page - 1) * filter.per_page)
            .take(filter.per_page)
            .collect();
        Ok(json!({"revision":data.revision,"shares":shares,"total":total,"page":filter.page,"per_page":filter.per_page}))
    })
    .await?;
    Ok(response(value))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Create {
    expected_revision: u64,
    resource: Resource,
    recipient: Recipient,
}
pub(super) async fn create(
    State(app): State<Shared>,
    Path(home): Path<String>,
    request: Request,
) -> Result<Response, Failure> {
    let actor = user(&app, &request)?;
    let check_home = home.clone();
    let check_actor = actor.clone();
    locked(app.clone(), move |m, _| owner(m, &check_home, &check_actor)).await?;
    let body: Create = input(request).await?;
    if let Recipient::Account { user } | Recipient::HouseholdInvitation { user } = &body.recipient {
        super::people::verify(app.clone(), &actor, user).await?;
    }
    if let Some(reference) = body.resource.reference() {
        crate::exapp::library::source(app.clone(), home.clone(), &reference).await?;
    }
    let value = locked(app, move |m, s| {
        owner(m, &home, &actor)?;
        match &body.recipient {
            Recipient::Account { user } if user == &actor => return Err("invalid_share_recipient"),
            Recipient::Household { home_id } if m.role(home_id, &actor) != Some(Role::Owner) => {
                return Err("household_owner_required");
            }
            _ => {}
        }
        let id = s.grant(&home, body.expected_revision, body.resource, body.recipient)?;
        Ok(json!({"id":id,"revision":s.home(&home).revision}))
    })
    .await?;
    Ok(response(value))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Publish {
    expected_revision: u64,
    resource: Resource,
}
pub(super) async fn publish(
    State(app): State<Shared>,
    Path((home, id)): Path<(String, String)>,
    request: Request,
) -> Result<Response, Failure> {
    let actor = user(&app, &request)?;
    let check_home = home.clone();
    let check_actor = actor.clone();
    locked(app.clone(), move |m, _| owner(m, &check_home, &check_actor)).await?;
    let body: Publish = input(request).await?;
    if let Some(reference) = body.resource.reference() {
        crate::exapp::library::source(app.clone(), home.clone(), &reference).await?;
    }
    let value = locked(app, move |m, s| {
        owner(m, &home, &actor)?;
        s.publish(&home, body.expected_revision, &id, body.resource)?;
        Ok(json!({"revision":s.home(&home).revision}))
    })
    .await?;
    Ok(response(value))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Revision {
    expected_revision: u64,
}
pub(super) async fn revoke(
    State(app): State<Shared>,
    Path((home, id)): Path<(String, String)>,
    request: Request,
) -> Result<Response, Failure> {
    let actor = user(&app, &request)?;
    let check_home = home.clone();
    let check_actor = actor.clone();
    locked(app.clone(), move |m, _| owner(m, &check_home, &check_actor)).await?;
    let body: Revision = input(request).await?;
    let value = locked(app, move |m, s| {
        owner(m, &home, &actor)?;
        s.revoke(&home, body.expected_revision, &id)?;
        Ok(json!({"revision":s.home(&home).revision}))
    })
    .await?;
    Ok(response(value))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Accept {
    expected_version: u64,
    household_id: String,
}
pub(super) async fn accept(
    State(app): State<Shared>,
    Path((home, id)): Path<(String, String)>,
    request: Request,
) -> Result<Response, Failure> {
    let actor = user(&app, &request)?;
    let body: Accept = input(request).await?;
    let value = locked(app, move |m, s| {
        let context = access::context(m, s, &home, &id, &actor, true)?;
        if !matches!(&context.grant.recipient, Recipient::HouseholdInvitation {user} if user == &actor) {
            return Err("share_not_found");
        }
        if m.role(&body.household_id, &actor) != Some(Role::Owner) {
            return Err("household_owner_required");
        }
        s.accept(&home, &id, body.expected_version, body.household_id)?;
        Ok(json!({}))
    })
    .await?;
    Ok(response(value))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Version {
    expected_version: u64,
}
pub(super) async fn decline(
    State(app): State<Shared>,
    Path((home, id)): Path<(String, String)>,
    request: Request,
) -> Result<Response, Failure> {
    let actor = user(&app, &request)?;
    let body: Version = input(request).await?;
    let value = locked(app, move |m, s| {
        let context = access::context(m, s, &home, &id, &actor, true)?;
        if let Recipient::Household { home_id } = &context.grant.recipient {
            if m.role(home_id, &actor) != Some(Role::Owner) {
                return Err("household_owner_required");
            }
        } else if !access::addressed(m, &context.grant.recipient, &actor) {
            return Err("share_not_found");
        }
        if context.grant.version != body.expected_version {
            return Err("revision_conflict");
        }
        s.revoke(&home, s.home(&home).revision, &id)?;
        Ok(json!({}))
    })
    .await?;
    Ok(response(value))
}
