use super::{
    super::{self as household, Authority, Shared, Store},
    state::{Entry, Grant, Permission, Person, entry_id},
};
use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::get,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) fn activity_path(path: &str) -> bool {
    let parts: Vec<_> = path.trim_start_matches('/').split('/').collect();
    (parts.len() == 5 || parts.len() == 6)
        && parts[..3] == ["api", "household", "people"]
        && parts[4] == "activity"
        && !parts[3].is_empty()
        && parts[3]
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
pub(crate) fn routes() -> Router<Shared> {
    Router::new()
        .route("/api/household/people/access", get(capabilities))
        .route(
            "/api/household/people/{person}/activity/access",
            get(read_access).put(write_access),
        )
        .route(
            "/api/household/people/{person}/activity/export",
            get(export),
        )
        .route(
            "/api/household/people/{person}/activity",
            get(list).delete(clear),
        )
        .route(
            "/api/household/people/{person}/activity/{entry}",
            get(detail).put(save).delete(remove),
        )
}
fn error(code: &str) -> Response {
    let status = match code {
        "person_activity_access_required" | "person_owner_required" => StatusCode::FORBIDDEN,
        "person_not_found" | "activity_not_found" => StatusCode::NOT_FOUND,
        "revision_conflict" => StatusCode::CONFLICT,
        "household_storage_failed" => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::UNPROCESSABLE_ENTITY,
    };
    household::reply(status, json!({"code":code}))
}
async fn access<F>(
    state: Shared,
    headers: HeaderMap,
    who: Option<Extension<Authority>>,
    action: F,
) -> Response
where
    F: FnOnce(&mut Store, Authority) -> Result<Value, &'static str> + Send + 'static,
{
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    let actor = household::authority(&state, &headers, who);
    match tokio::task::spawn_blocking(move || {
        let mut store = state.store.lock().map_err(|_| "household_storage_failed")?;
        action(&mut store, actor)
    })
    .await
    {
        Ok(Ok(value)) => household::reply(StatusCode::OK, value),
        Ok(Err(code)) => error(code),
        Err(_) => error("household_storage_failed"),
    }
}
fn permission(person: Option<&Person>, actor: &Authority) -> Option<Permission> {
    person
        .and_then(|person| person.grants.get(&actor.user))
        .filter(|g| {
            g.epoch == actor.activity_epochs.get(&actor.user).copied().unwrap_or(0)
                && actor.members.contains_key(&actor.user)
        })
        .map(|g| g.permission)
}
fn metadata(store: &Store, id: &str, actor: &Authority) -> Value {
    let person = store.private.people.get(id);
    let perm = permission(person, actor);
    json!({"person_id":id,"generation":person.map(|person|person.generation),"access_revision":person.map_or(0,|person|person.access_revision),"can_read":perm.is_some(),"can_edit":perm==Some(Permission::Edit),"can_manage":actor.role==crate::membership::Role::Owner})
}
fn exists(store: &Store, id: &str) -> Result<(), &'static str> {
    if store.data.members.iter().any(|m| m.id == id) {
        Ok(())
    } else {
        Err("person_not_found")
    }
}
fn permitted<'authority>(
    store: &'authority Store,
    id: &str,
    actor: &Authority,
    edit: bool,
) -> Result<&'authority Person, &'static str> {
    exists(store, id)?;
    let person = store.private.people.get(id);
    if permission(person, actor).is_none_or(|permission| edit && permission != Permission::Edit) {
        return Err("person_activity_access_required");
    }
    person.ok_or("person_activity_access_required")
}
async fn capabilities(
    State(state): State<Shared>,
    headers: HeaderMap,
    authority: Option<Extension<Authority>>,
) -> Response {
    access(state,headers,authority,|store,actor|Ok(json!({"version":1,"people":store.data.members.iter().map(|m|metadata(store,&m.id,&actor)).collect::<Vec<_>>()}))).await
}
async fn read_access(
    State(state): State<Shared>,
    headers: HeaderMap,
    authority: Option<Extension<Authority>>,
    Path(id): Path<String>,
) -> Response {
    access(state,headers,authority,move|store,actor|{
        exists(store,&id)?;
        if actor.role!=crate::membership::Role::Owner {permitted(store,&id,&actor,false)?;}
        let mut result=metadata(store,&id,&actor);
        result["accounts"]=json!(actor.members);
        result["grants"]=json!(store.private.people.get(&id).map(|person|person.grants.iter().map(|(user,g)|json!({"user":user,"permission":g.permission,"active":actor.members.contains_key(user)&&g.epoch==actor.activity_epochs.get(user).copied().unwrap_or(0)})).collect::<Vec<_>>()).unwrap_or_default());
        Ok(result)
    }).await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AccessInput {
    expected_profile_revision: u64,
    expected_generation: Option<u64>,
    expected_access_revision: u64,
    grants: BTreeMap<String, Permission>,
    #[serde(default)]
    confirm_self_access: bool,
}
async fn write_access(
    State(state): State<Shared>,
    headers: HeaderMap,
    authority: Option<Extension<Authority>>,
    Path(id): Path<String>,
    body: Result<Json<AccessInput>, JsonRejection>,
) -> Response {
    let Ok(Json(input)) = body else {
        return error("invalid_person_activity");
    };
    access(state, headers, authority, move |store, actor| {
        exists(store, &id)?;
        if actor.role != crate::membership::Role::Owner {
            return Err("person_owner_required");
        }
        let current = store.private.people.get(&id);
        if input.expected_profile_revision != store.data.revision
            || input.expected_generation != current.map(|person| person.generation)
            || input.expected_access_revision != current.map_or(0, |person| person.access_revision)
        {
            return Err("revision_conflict");
        }
        if input.grants.len() > 100 || input.grants.keys().any(|u| !actor.members.contains_key(u)) {
            return Err("invalid_person_activity_grants");
        }
        if input.grants.contains_key(&actor.user)
            && permission(current, &actor).is_none()
            && !input.confirm_self_access
        {
            return Err("person_self_grant_confirmation_required");
        }
        let mut private = store.private.clone();
        let person = private.create(&id)?;
        person.access_revision = person
            .access_revision
            .checked_add(1)
            .ok_or("person_activity_capacity")?;
        person.grants = input
            .grants
            .into_iter()
            .map(|(user, permission)| {
                let epoch = actor.activity_epochs.get(&user).copied().unwrap_or(0);
                (user, Grant { permission, epoch })
            })
            .collect();
        store.publish(store.data.clone(), private)?;
        Ok(metadata(store, &id, &actor))
    })
    .await
}
#[derive(Deserialize, Default)]
struct Page {
    page: Option<usize>,
    per_page: Option<usize>,
}
fn view(person: &Person, id: &str, actor: &Authority) -> Value {
    json!({"person_id":id,"generation":person.generation,"access_revision":person.access_revision,"revision":person.revision,"can_edit":permission(Some(person),actor)==Some(Permission::Edit),"energy":null})
}
async fn list(
    State(state): State<Shared>,
    headers: HeaderMap,
    authority: Option<Extension<Authority>>,
    Path(id): Path<String>,
    Query(page): Query<Page>,
) -> Response {
    access(state, headers, authority, move |store, actor| {
        let person = permitted(store, &id, &actor, false)?;
        let number = page.page.unwrap_or(1);
        let size = page.per_page.unwrap_or(50);
        if number == 0 || number > 10000 || !(1..=50).contains(&size) {
            return Err("invalid_page");
        }
        let mut entries = person.entries.iter().collect::<Vec<_>>();
        entries.sort_by(|(aid, authority), (bid, b)| {
            b.date.cmp(&authority.date).then_with(|| bid.cmp(aid))
        });
        let mut value = view(person, &id, &actor);
        value["entries"] = json!(
            entries
                .into_iter()
                .skip((number - 1) * size)
                .take(size)
                .map(|(id, e)| json!({"id":id,"entry":e}))
                .collect::<Vec<_>>()
        );
        value["page"] = json!(number);
        value["per_page"] = json!(size);
        value["total"] = json!(person.entries.len());
        Ok(value)
    })
    .await
}
async fn export(
    State(state): State<Shared>,
    headers: HeaderMap,
    authority: Option<Extension<Authority>>,
    Path(id): Path<String>,
) -> Response {
    access(state, headers, authority, move |store, actor| {
        let person = permitted(store, &id, &actor, false)?;
        let mut value = view(person, &id, &actor);
        value["entries"] = json!(person.entries);
        Ok(value)
    })
    .await
}
async fn detail(
    State(state): State<Shared>,
    headers: HeaderMap,
    authority: Option<Extension<Authority>>,
    Path((id, entry)): Path<(String, String)>,
) -> Response {
    access(state, headers, authority, move |store, actor| {
        let person = permitted(store, &id, &actor, false)?;
        let mut value = view(person, &id, &actor);
        value["id"] = json!(entry);
        value["entry"] = json!(person.entries.get(&entry).ok_or("activity_not_found")?);
        Ok(value)
    })
    .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Guard {
    #[serde(rename = "expected_generation")]
    generation: u64,
    #[serde(rename = "expected_access_revision")]
    access_revision: u64,
    #[serde(rename = "expected_revision")]
    revision: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    expected_generation: u64,
    expected_access_revision: u64,
    expected_revision: u64,
    entry: Entry,
}
fn guard(person: &Person, g: &Guard) -> Result<(), &'static str> {
    if person.generation != g.generation
        || person.access_revision != g.access_revision
        || person.revision != g.revision
    {
        Err("revision_conflict")
    } else {
        Ok(())
    }
}
async fn save(
    State(state): State<Shared>,
    headers: HeaderMap,
    authority: Option<Extension<Authority>>,
    Path((id, key)): Path<(String, String)>,
    body: Result<Json<Input>, JsonRejection>,
) -> Response {
    let Ok(Json(input)) = body else {
        return error("invalid_person_activity");
    };
    access(state, headers, authority, move |store, actor| {
        let person = permitted(store, &id, &actor, true)?;
        guard(
            person,
            &Guard {
                generation: input.expected_generation,
                access_revision: input.expected_access_revision,
                revision: input.expected_revision,
            },
        )?;
        if !entry_id(&key) || !input.entry.valid() {
            return Err("invalid_person_activity");
        }
        let mut private = store.private.clone();
        let person = private
            .people
            .get_mut(&id)
            .ok_or("person_activity_access_required")?;
        if !person.entries.contains_key(&key) && person.entries.len() >= 1000 {
            return Err("person_activity_capacity");
        }
        person.entries.insert(key, input.entry);
        person.revision = person
            .revision
            .checked_add(1)
            .ok_or("person_activity_capacity")?;
        let value = view(person, &id, &actor);
        store.publish(store.data.clone(), private)?;
        Ok(value)
    })
    .await
}
async fn delete(
    state: Shared,
    headers: HeaderMap,
    authority: Option<Extension<Authority>>,
    id: String,
    key: Option<String>,
    body: Result<Json<Guard>, JsonRejection>,
) -> Response {
    let Ok(Json(input)) = body else {
        return error("invalid_person_activity");
    };
    access(state, headers, authority, move |store, actor| {
        guard(permitted(store, &id, &actor, true)?, &input)?;
        let mut private = store.private.clone();
        let person = private
            .people
            .get_mut(&id)
            .ok_or("person_activity_access_required")?;
        if let Some(key) = key {
            person.entries.remove(&key).ok_or("activity_not_found")?;
        } else {
            person.entries.clear();
        }
        person.revision = person
            .revision
            .checked_add(1)
            .ok_or("person_activity_capacity")?;
        let value = view(person, &id, &actor);
        store.publish(store.data.clone(), private)?;
        Ok(value)
    })
    .await
}
async fn remove(
    State(state): State<Shared>,
    headers: HeaderMap,
    authority: Option<Extension<Authority>>,
    Path((id, key)): Path<(String, String)>,
    b: Result<Json<Guard>, JsonRejection>,
) -> Response {
    delete(state, headers, authority, id, Some(key), b).await
}
async fn clear(
    State(state): State<Shared>,
    headers: HeaderMap,
    authority: Option<Extension<Authority>>,
    Path(id): Path<String>,
    b: Result<Json<Guard>, JsonRejection>,
) -> Response {
    delete(state, headers, authority, id, None, b).await
}
