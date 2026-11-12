use super::{
    model::Command,
    operations::{self, Context},
    view,
};
use crate::household::{self, Authority, Shared};
use axum::{
    Extension, Json, Router,
    extract::{Query, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::json;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    expected_revision: u64,
    idempotency_key: String,
    command: Command,
    reviewed: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Page {
    #[serde(default)]
    offset: usize,
    #[serde(default = "page_limit")]
    limit: usize,
    #[serde(default)]
    history_offset: usize,
    #[serde(default = "page_limit")]
    history_limit: usize,
    #[serde(default)]
    run_id: Option<String>,
}
fn page_limit() -> usize {
    50
}
pub(crate) fn routes() -> Router<Shared> {
    Router::new()
        .route("/api/household/inventory", get(read).post(confirm))
        .route("/api/household/inventory/preview", post(preview))
}
fn failure(code: &str) -> Response {
    let status = match code {
        "shopping_revision_conflict"
        | "inventory_request_conflict"
        | "inventory_already_applied" => StatusCode::CONFLICT,
        "shopping_storage_failed"
        | "inventory_storage_failed"
        | "household_storage_failed"
        | "cooking_storage_failed"
        | "recipe_store_unavailable"
        | "ingredient_storage_failed" => StatusCode::SERVICE_UNAVAILABLE,
        "household_write_required" | "inventory_review_required" => StatusCode::FORBIDDEN,
        _ => StatusCode::UNPROCESSABLE_ENTITY,
    };
    household::reply(status, json!({"code":code}))
}
async fn read(State(state): State<Shared>, headers: HeaderMap, page: Query<Page>) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    if page.limit == 0 || page.limit > 500 || page.history_limit == 0 || page.history_limit > 200 {
        return failure("inventory_selection_invalid");
    }
    let result = tokio::task::spawn_blocking(move || {
        let profiles = state.store.lock().map_err(|_| "household_storage_failed")?;
        let recipes = state
            .recipes
            .as_ref()
            .map(|s| s.lock())
            .transpose()
            .map_err(|_| "recipe_store_unavailable")?;
        let cooking = state.cooking.lock().map_err(|_| "cooking_storage_failed")?;
        let ingredients = state
            .catalogue
            .lock()
            .map_err(|_| "ingredient_storage_failed")?;
        let store = state
            .shopping
            .lock()
            .map_err(|_| "shopping_storage_failed")?;
        store.ready()?;
        let mut value = view::view(
            &store.data,
            &Context {
                state: &state,
                profiles: &profiles.data,
                ingredients: &ingredients,
                recipes: recipes.as_deref(),
                cooking: &cooking,
                now: super::now()?,
            },
            page.offset,
            page.limit,
            page.history_offset,
            page.history_limit,
        )?;
        if let Some(id) = &page.run_id {
            let run = cooking.data.run(id)?;
            value["cooking_selection"] = super::cooking::selection(
                &store.data,
                recipes.as_deref().ok_or("recipe_store_unavailable")?,
                run,
            )?;
        }
        Ok(value)
    })
    .await;
    respond(result)
}
async fn preview(
    State(state): State<Shared>,
    headers: HeaderMap,
    actor: Option<Extension<Authority>>,
    input: Result<Json<Input>, JsonRejection>,
) -> Response {
    change(state, headers, actor, input, false).await
}
async fn confirm(
    State(state): State<Shared>,
    headers: HeaderMap,
    actor: Option<Extension<Authority>>,
    input: Result<Json<Input>, JsonRejection>,
) -> Response {
    change(state, headers, actor, input, true).await
}
async fn change(
    state: Shared,
    headers: HeaderMap,
    who: Option<Extension<Authority>>,
    input: Result<Json<Input>, JsonRejection>,
    apply: bool,
) -> Response {
    if !household::can_write(&state, &headers) {
        return failure("household_write_required");
    }
    let actor = household::authority(&state, &headers, who);
    if actor.role == crate::membership::Role::Reader {
        return failure("household_write_required");
    }
    let Ok(Json(input)) = input else {
        return failure("inventory_request_invalid");
    };
    if !crate::recipes::Creation::valid_key(&input.idempotency_key) {
        return failure("inventory_key_invalid");
    }
    if apply && !input.reviewed {
        return failure("inventory_review_required");
    }
    let result=tokio::task::spawn_blocking(move||{
        let profiles=state.store.lock().map_err(|_|"household_storage_failed")?;
        let recipes=state.recipes.as_ref().map(|s|s.lock()).transpose().map_err(|_|"recipe_store_unavailable")?;
        let cooking=state.cooking.lock().map_err(|_|"cooking_storage_failed")?;
        let ingredients=state.catalogue.lock().map_err(|_|"ingredient_storage_failed")?;
        let mut store=state.shopping.lock().map_err(|_|"shopping_storage_failed")?;
        store.ready()?;
        let ctx=Context{state:&state,profiles:&profiles.data,ingredients:&ingredients,recipes:recipes.as_deref(),cooking:&cooking,now:super::now()?};
        if let Some(id)=store.data.inventory.replay(&actor.user,&input.idempotency_key,input.expected_revision,&input.command)? {
            if !apply {return Err("inventory_already_applied");}
            return Ok(json!({"operation_id":id,"inventory":view::view(&store.data,&ctx,0,500,0,50)?,"replayed":true}));
        }
        if !apply {
            let next=store.prepare(input.expected_revision,|data|operations::execute(data,&ctx,&actor.user,&input.idempotency_key,input.expected_revision,input.command).map(|_|()))?;
            return view::preview(&store.data,&next);
        }
        let key=input.idempotency_key;
        store.change(input.expected_revision,|data|operations::execute(data,&ctx,&actor.user,&key,input.expected_revision,input.command).map(|_|()))?;
        let id=store.data.inventory.operations.iter().find(|op|op.actor==actor.user&&op.key==key).map(|op|op.id.clone()).ok_or("inventory_storage_failed")?;
        Ok(json!({"operation_id":id,"inventory":view::view(&store.data,&ctx,0,500,0,50)?,"replayed":false}))
    }).await;
    respond(result)
}
fn respond(
    result: Result<Result<serde_json::Value, &'static str>, tokio::task::JoinError>,
) -> Response {
    match result {
        Ok(Ok(value)) => household::reply(StatusCode::OK, value),
        Ok(Err(code)) => failure(code),
        Err(_) => failure("inventory_storage_failed"),
    }
}
