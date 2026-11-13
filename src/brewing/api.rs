use super::{
    operations::{self, Command},
    view,
};
use crate::{
    cooking::time,
    household::{self, Authority, Shared},
};
use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
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
struct Version {
    revision: Option<u64>,
}
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/api/household/brewing", get(read).post(confirm))
        .route("/api/household/brewing/preview", post(preview))
        .route("/api/household/brewing/recipes/{id}", get(recipe))
}
fn failure(code: &str) -> Response {
    let status = match code {
        "household_write_required" | "brewing_review_required" => StatusCode::FORBIDDEN,
        "cooking_revision_conflict"
        | "brewing_recipe_conflict"
        | "brewing_request_conflict"
        | "brewing_already_applied" => StatusCode::CONFLICT,
        "cooking_storage_failed"
        | "brewing_storage_failed"
        | "brewing_templates_unavailable"
        | "household_storage_failed" => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::UNPROCESSABLE_ENTITY,
    };
    household::reply(status, json!({"code":code}))
}
async fn read(State(state): State<Shared>, headers: HeaderMap) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    respond(
        tokio::task::spawn_blocking(move || {
            let store = state.cooking.lock().map_err(|_| "cooking_storage_failed")?;
            store.ready()?;
            Ok(view::view(&store.data, time::now()?))
        })
        .await,
    )
}
async fn recipe(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<Version>,
) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    respond(
        tokio::task::spawn_blocking(move || {
            let store = state.cooking.lock().map_err(|_| "cooking_storage_failed")?;
            store.ready()?;
            let revision = q
                .revision
                .or_else(|| {
                    store
                        .data
                        .brewing
                        .recipes
                        .iter()
                        .filter(|r| r.id == id)
                        .map(|r| r.revision)
                        .max()
                })
                .ok_or("brewing_recipe_not_found")?;
            Ok(json!(store.data.brewing.recipe(&id, revision)?))
        })
        .await,
    )
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
    actor: Option<Extension<Authority>>,
    input: Result<Json<Input>, JsonRejection>,
    apply: bool,
) -> Response {
    if !household::can_write(&state, &headers) {
        return failure("household_write_required");
    }
    let actor = household::authority(&state, &headers, actor);
    if actor.role == crate::membership::Role::Reader {
        return failure("household_write_required");
    }
    let Ok(Json(input)) = input else {
        return failure("brewing_request_invalid");
    };
    if !crate::recipes::Creation::valid_key(&input.idempotency_key) {
        return failure("brewing_key_invalid");
    }
    if apply && !input.reviewed {
        return failure("brewing_review_required");
    }
    let result = tokio::task::spawn_blocking(move || transact(&state, &actor, input, apply)).await;
    respond(result)
}
fn transact(
    state: &household::Household,
    actor: &Authority,
    input: Input,
    apply: bool,
) -> Result<Value, &'static str> {
    let mut store = state.cooking.lock().map_err(|_| "cooking_storage_failed")?;
    store.ready()?;
    if let Some(id) = operations::replay(
        &store.data,
        &actor.user,
        &input.idempotency_key,
        input.expected_revision,
        &input.command,
    )? {
        if !apply {
            return Err("brewing_already_applied");
        }
        return Ok(
            json!({"operation_id":id,"replayed":true,"brewing":view::view(&store.data,time::now()?)}),
        );
    }
    if let Command::SaveRecipe { recipe, .. } = &input.command {
        let ingredients = state
            .catalogue
            .lock()
            .map_err(|_| "ingredient_storage_failed")?;
        let catalogue = ingredients.catalogue();
        if recipe.ingredients.iter().any(|line| {
            line.ingredient_id
                .as_ref()
                .is_some_and(|id| catalogue.find(id).is_none())
        }) {
            return Err("unknown_ingredient");
        }
    }
    let now = time::now()?;
    let mut operation_id = String::new();
    let mutate = |data: &mut crate::cooking::model::Data| {
        operation_id = operations::execute(
            data,
            &actor.user,
            &input.idempotency_key,
            input.expected_revision,
            input.command,
            now,
        )?;
        Ok(())
    };
    if apply {
        store.change(Some(input.expected_revision), mutate)?;
        Ok(
            json!({"operation_id":operation_id,"replayed":false,"brewing":view::view(&store.data,now)}),
        )
    } else {
        let next = store.prepare_change(Some(input.expected_revision), mutate)?;
        Ok(
            json!({"expected_revision":input.expected_revision,"before":view::view(&store.data,now),"after":view::view(&next,now)}),
        )
    }
}

fn respond(result: Result<Result<Value, &'static str>, tokio::task::JoinError>) -> Response {
    match result {
        Ok(Ok(value)) => household::reply(StatusCode::OK, value),
        Ok(Err(code)) => failure(code),
        Err(_) => failure("brewing_storage_failed"),
    }
}
