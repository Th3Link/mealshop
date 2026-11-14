use super::{
    model::Data,
    sources::{self, MealSlot},
};
use crate::household::{self, Shared};
use axum::{
    Json, Router,
    extract::{State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::post,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    expected_revision: u64,
    plan_revision: u64,
    slots: Vec<MealSlot>,
}
pub(super) fn routes() -> Router<Shared> {
    Router::new().route("/api/household/shopping/plan-preview", post(preview))
}
fn changes(before: &Data, after: &Data) -> Vec<Value> {
    let old = before
        .rows()
        .into_iter()
        .map(|r| (r["key"].as_str().unwrap_or("").to_owned(), r))
        .collect::<BTreeMap<_, _>>();
    let new = after
        .rows()
        .into_iter()
        .map(|r| (r["key"].as_str().unwrap_or("").to_owned(), r))
        .collect::<BTreeMap<_, _>>();
    old.keys()
        .chain(new.keys())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .filter(|id| old.get(*id) != new.get(*id))
        .map(|id| json!({"before":old.get(id),"after":new.get(id)}))
        .collect()
}
async fn preview(
    State(state): State<Shared>,
    headers: HeaderMap,
    input: Result<Json<Input>, JsonRejection>,
) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    let Ok(Json(input)) = input else {
        return household::reply(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({"code":"shopping_plan_invalid"}),
        );
    };
    let result=tokio::task::spawn_blocking(move || {
        let recipes=state.recipes.as_ref().ok_or("recipe_store_unavailable")?.lock().map_err(|_|"recipe_store_unavailable")?;
        let plans=state.plans.lock().map_err(|_|"mealplan_storage_failed")?;
        let shopping=state.shopping.lock().map_err(|_|"shopping_storage_failed")?;
        let selected=plans.shopping_recipes(input.plan_revision,&input.slots)?;
        let after=shopping.prepare(input.expected_revision,|d|sources::sync(d,&recipes,selected))?;
        Ok::<_,&'static str>(json!({"expected_revision":input.expected_revision,
            "command":{"operation":"sync_plan","plan_revision":input.plan_revision,"slots":input.slots},
            "changes":changes(&shopping.data,&after),"manual_and_purchase_receipts_preserved":true}))
    }).await;
    match result {
        Ok(Ok(value)) => household::reply(StatusCode::OK, value),
        Ok(Err(code)) => household::reply(
            if code == "shopping_revision_conflict" {
                StatusCode::CONFLICT
            } else {
                StatusCode::UNPROCESSABLE_ENTITY
            },
            json!({"code":code}),
        ),
        Err(_) => household::reply(
            StatusCode::SERVICE_UNAVAILABLE,
            json!({"code":"shopping_storage_failed"}),
        ),
    }
}
