//! Versioned, actor-scoped request receipts; legacy shopping responses stay unchanged.
use super::{
    api,
    model::{Data, text},
};
use crate::household::{self, Authority, Shared};
use axum::{
    Extension, Json, Router,
    extract::{State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn ingredient_groups(view: &Value, catalogue: &crate::nutrition::catalogue::Catalogue) -> Value {
    let mut groups = BTreeMap::new();
    for item in view["items"].as_array().into_iter().flatten() {
        let Some(ingredient) = item["ingredient_id"]
            .as_str()
            .and_then(|id| catalogue.find(id))
        else {
            continue;
        };
        let group = if ingredient.organization.is_group {
            Some(ingredient)
        } else {
            ingredient
                .organization
                .group_id
                .as_deref()
                .and_then(|id| catalogue.find(id))
        };
        if let Some(group) = group {
            let translations: BTreeMap<_, _> = group
                .translations
                .iter()
                .map(|(lang, value)| (lang, &value.name))
                .collect();
            groups.insert(
                &ingredient.id,
                json!({"id":group.id,"name":group.name,"translations":translations}),
            );
        }
    }
    json!(groups)
}

pub(super) const MAX_AGE: u64 = 7 * 86400;
pub(super) const RETENTION: u64 = 8 * 86400;
const CAPACITY: usize = 1024;
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Receipt {
    actor: String,
    operation_id: String,
    pub(super) issued_at: u64,
    pub(super) recorded_at: u64,
    applied_revision: u64,
    request: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    operation_id: String,
    issued_at: u64,
    expected_revision: u64,
    command: Value,
}
pub(super) fn routes() -> Router<Shared> {
    Router::new().route(
        "/api/household/shopping/mobile",
        get(capabilities).post(change),
    )
}
fn key(value: &str) -> bool {
    (16..=80).contains(&value.len())
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
pub(super) fn validate_receipts(data: &Data) -> Result<(), &'static str> {
    let mut keys = BTreeSet::new();
    let mut revisions = BTreeSet::new();
    if data.mobile_receipts.len() > CAPACITY
        || data.mobile_receipts.iter().any(|r| {
            !text(&r.actor, 255)
                || !key(&r.operation_id)
                || r.issued_at == 0
                || r.recorded_at == 0
                || r.issued_at > r.recorded_at.saturating_add(300)
                || r.issued_at.saturating_add(MAX_AGE) < r.recorded_at
                || r.applied_revision == 0
                || r.applied_revision > data.revision
                || !keys.insert((&r.actor, &r.operation_id))
                || !revisions.insert(r.applied_revision)
                || serde_json::to_vec(&r.request).map_or(true, |v| v.len() > 8192)
                || serde_json::from_value::<api::Mutation>(r.request.clone())
                    .map_or(true, |input| {
                        input.expected_revision.checked_add(1) != Some(r.applied_revision)
                    })
        })
    {
        return Err("shopping_receipt_invalid");
    }
    Ok(())
}
fn failure(code: &str) -> Response {
    let status = match code {
        "shopping_revision_conflict" | "shopping_operation_reused" => StatusCode::CONFLICT,
        "shopping_storage_failed" => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::UNPROCESSABLE_ENTITY,
    };
    household::reply(status, json!({"code":code}))
}
async fn capabilities(
    State(state): State<Shared>,
    headers: HeaderMap,
    who: Option<Extension<Authority>>,
) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    let authority = household::authority(&state, &headers, who);
    match tokio::task::spawn_blocking(move || {
        let ingredients = state.catalogue.lock().map_err(|_| "ingredient_storage_failed")?;
        let store = state.shopping.lock().map_err(|_| "shopping_storage_failed")?;
        let view = store.view();
        Ok::<_, &'static str>(json!({"version":1,"server_now":super::lookup::now()?,"max_operation_age_seconds":MAX_AGE,"role":authority.role,"ingredient_groups":ingredient_groups(&view, &ingredients.catalogue()),"shopping":view}))
    }).await {
        Ok(Ok(value)) => household::reply(StatusCode::OK, value),
        Ok(Err(code)) => failure(code), Err(_) => failure("shopping_storage_failed"),
    }
}
async fn change(
    State(state): State<Shared>,
    headers: HeaderMap,
    who: Option<Extension<Authority>>,
    body: Result<Json<Input>, JsonRejection>,
) -> Response {
    // ExApp dispatch rechecks membership on every request, including receipt replay.
    if !household::can_write(&state, &headers)
        || who
            .as_ref()
            .is_some_and(|a| a.role == crate::membership::Role::Reader)
    {
        return household::reply(
            StatusCode::FORBIDDEN,
            json!({"code":"household_write_required"}),
        );
    }
    let authority = household::authority(&state, &headers, who);
    let Ok(Json(input)) = body else {
        return failure("shopping_request_invalid");
    };
    let request = json!({"expected_revision":input.expected_revision,"command":input.command});
    if !key(&input.operation_id) || serde_json::to_vec(&request).map_or(true, |v| v.len() > 8192) {
        return failure("shopping_request_invalid");
    }
    let Ok(mutation) = serde_json::from_value::<api::Mutation>(request.clone()) else {
        return failure("shopping_request_invalid");
    };
    match tokio::task::spawn_blocking(move || {
        let now = super::lookup::now()?;
        if input.issued_at == 0 || input.issued_at > now.saturating_add(300) || input.issued_at.saturating_add(MAX_AGE) < now {
            return Err("shopping_operation_expired");
        }
        let recipes = state.recipes.as_ref().map(|s| s.lock()).transpose().map_err(|_| "recipe_store_unavailable")?;
        let plans = state.plans.lock().map_err(|_| "mealplan_storage_failed")?;
        let ingredients = state.catalogue.lock().map_err(|_| "ingredient_storage_failed")?;
        let mut store = state.shopping.lock().map_err(|_| "shopping_storage_failed")?;
        store.ready()?;
        let (applied_revision, replayed) = if let Some(receipt) = store.data.mobile_receipts.iter().find(|r| r.actor == authority.user && r.operation_id == input.operation_id) {
            if receipt.request != request || receipt.issued_at != input.issued_at { return Err("shopping_operation_reused"); }
            (receipt.applied_revision, true)
        } else {
            if store.data.mobile_receipts.iter().filter(|r| r.issued_at.saturating_add(RETENTION) >= now).count() >= CAPACITY { return Err("shopping_receipt_capacity"); }
            let applied_revision = input.expected_revision.checked_add(1).ok_or("shopping_capacity")?;
            let receipt = Receipt { actor:authority.user, operation_id:input.operation_id.clone(), issued_at:input.issued_at, recorded_at:now, applied_revision, request };
            api::execute(&mut store, &ingredients, recipes.as_deref(), &plans, mutation, Some(receipt))?;
            (applied_revision, false)
        };
        let view = store.view();
        Ok::<_, &'static str>(json!({"version":1,"operation_id":input.operation_id,"applied_revision":applied_revision,"replayed":replayed,"ingredient_groups":ingredient_groups(&view, &ingredients.catalogue()),"shopping":view}))
    }).await {
        Ok(Ok(value)) => household::reply(StatusCode::OK, value),
        Ok(Err(code)) => failure(code), Err(_) => failure("shopping_storage_failed"),
    }
}
