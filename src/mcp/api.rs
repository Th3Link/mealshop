use super::{Authority, Scope};
use crate::{
    household::{self, Shared},
    membership::Role,
};
use axum::{
    Extension, Json, Router,
    extract::{Query, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub(crate) fn routes() -> Router<Shared> {
    Router::new()
        .route("/api/household/mcp", post(super::protocol::handle))
        .route("/api/household/mcp/settings", get(settings).put(configure))
        .route("/api/household/mcp/activity", get(activity))
}
pub(super) fn authority(
    state: &crate::household::Household,
    headers: &HeaderMap,
    who: Option<Extension<Authority>>,
) -> Authority {
    who.map_or_else(
        || Authority {
            user: if household::can_write(state, headers) {
                "local"
            } else {
                "local-reader"
            }
            .to_owned(),
            role: if household::can_write(state, headers) {
                Role::Owner
            } else {
                Role::Reader
            },
            members: BTreeMap::from([
                ("local".to_owned(), Role::Owner),
                ("local-reader".to_owned(), Role::Reader),
            ]),
        },
        |Extension(a)| a,
    )
}
pub(super) fn failure(code: &'static str) -> Response {
    household::reply(
        match code {
            "mcp_owner_required" => StatusCode::FORBIDDEN,
            "mcp_revision_conflict" => StatusCode::CONFLICT,
            "mcp_storage_failed" => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::UNPROCESSABLE_ENTITY,
        },
        json!({"code":code}),
    )
}
fn view(store: &super::Store, who: &Authority) -> Result<Value, &'static str> {
    let scope = store.scope(&who.user, who.role)?;
    let owner = who.role == Role::Owner;
    let members = if owner {
        who.members.clone()
    } else {
        BTreeMap::from([(who.user.clone(), who.role)])
    };
    let grants = if owner {
        store.grants.users.clone()
    } else {
        BTreeMap::from([(who.user.clone(), scope)])
    };
    Ok(
        json!({"revision":store.grants.revision,"user":who.user,"owner":owner,"scope":scope,"members":members,"grants":grants}),
    )
}
async fn settings(
    State(state): State<Shared>,
    headers: HeaderMap,
    who: Option<Extension<Authority>>,
) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    let who = authority(&state, &headers, who);
    match state.mcp.lock() {
        Ok(store) => match view(&store, &who) {
            Ok(value) => household::reply(StatusCode::OK, value),
            Err(code) => failure(code),
        },
        Err(_) => failure("mcp_storage_failed"),
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Configure {
    expected_revision: u64,
    grants: BTreeMap<String, Scope>,
}
async fn configure(
    State(state): State<Shared>,
    headers: HeaderMap,
    who: Option<Extension<Authority>>,
    input: Result<Json<Configure>, JsonRejection>,
) -> Response {
    if !household::can_write(&state, &headers) {
        return failure("mcp_owner_required");
    }
    let who = authority(&state, &headers, who);
    if who.role != Role::Owner {
        return failure("mcp_owner_required");
    }
    let Ok(Json(input)) = input else {
        return failure("mcp_invalid_grants");
    };
    if input.grants.iter().any(|(id, scope)| {
        who.members
            .get(id)
            .is_none_or(|role| *scope == Scope::Create && *role == Role::Reader)
    }) {
        return failure("mcp_invalid_grants");
    }
    match tokio::task::spawn_blocking(move || {
        let mut store = state.mcp.lock().map_err(|_| "mcp_storage_failed")?;
        store.configure(input.expected_revision, input.grants)?;
        view(&store, &who)
    })
    .await
    {
        Ok(Ok(value)) => household::reply(StatusCode::OK, value),
        Ok(Err(code)) => failure(code),
        Err(_) => failure("mcp_storage_failed"),
    }
}
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Page {
    page: Option<usize>,
}
async fn activity(
    State(state): State<Shared>,
    headers: HeaderMap,
    query: Result<Query<Page>, axum::extract::rejection::QueryRejection>,
) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    let Ok(Query(query)) = query else {
        return failure("mcp_invalid_page");
    };
    let page = query.page.unwrap_or(1);
    if page == 0 || page.checked_mul(20).is_none() {
        return failure("mcp_invalid_page");
    }
    match tokio::task::spawn_blocking(move || {
        let recipes = state
            .recipes
            .as_ref()
            .ok_or("mcp_storage_failed")?
            .lock()
            .map_err(|_| "mcp_storage_failed")?;
        let records = recipes.creation_activity();
        let total = records.len();
        let page = page.min(total.div_ceil(20).max(1));
        let entries: Vec<_> = records.into_iter().skip((page - 1) * 20).take(20).collect();
        Ok::<_, &'static str>(json!({"page":page,"per_page":20,"total":total,"entries":entries}))
    })
    .await
    {
        Ok(Ok(value)) => household::reply(StatusCode::OK, value),
        Ok(Err(code)) => failure(code),
        Err(_) => failure("mcp_storage_failed"),
    }
}
pub(super) fn now() -> Result<i64, &'static str> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|n| i64::try_from(n.as_secs()).ok())
        .ok_or("mcp_clock_invalid")
}
