//! Local recipe API; blocking filesystem work stays off async runtime workers.

use crate::{
    nutrition::{self, catalogue::Catalogue},
    recipes::{self, Recipe, Record, Store},
};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Query, State, rejection::JsonRejection},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::{Arc, Mutex};

type Shared = Arc<RecipeState>;
struct RecipeState {
    catalogue: Catalogue,
    store: Arc<Mutex<Store>>,
}

pub(crate) fn router(catalogue: Catalogue, store: Store) -> Router {
    router_shared(catalogue, Arc::new(Mutex::new(store)))
}
pub(crate) fn router_shared(catalogue: Catalogue, store: Arc<Mutex<Store>>) -> Router {
    Router::new()
        .route("/api/recipes", get(list).post(create))
        .route("/api/recipes/{id}", get(detail).put(update))
        .route("/api/recipes/{id}/scale", get(scale))
        .layer(DefaultBodyLimit::max(131_072))
        .with_state(Arc::new(RecipeState { catalogue, store }))
}

struct ApiError(StatusCode, &'static str, String);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.0,
            [(header::CACHE_CONTROL, "no-store")],
            Json(json!({"code":self.1,"error":self.2})),
        )
            .into_response()
    }
}
impl From<recipes::Error> for ApiError {
    fn from(error: recipes::Error) -> Self {
        match error {
            recipes::Error::Invalid(message) => {
                Self(StatusCode::UNPROCESSABLE_ENTITY, "invalid_recipe", message)
            }
            recipes::Error::NotFound => Self(
                StatusCode::NOT_FOUND,
                "recipe_not_found",
                "Recipe revision not found".to_owned(),
            ),
            recipes::Error::Conflict => Self(
                StatusCode::CONFLICT,
                "revision_conflict",
                "Recipe changed; reload before saving a new revision".to_owned(),
            ),
            recipes::Error::Storage => Self(
                StatusCode::SERVICE_UNAVAILABLE,
                "recipe_storage_failed",
                "Recipe storage failed; stop writers and follow the recovery guide before retrying"
                    .to_owned(),
            ),
            recipes::Error::Full => Self(
                StatusCode::INSUFFICIENT_STORAGE,
                "recipe_storage_full",
                "Local store limit: 1000 revisions, each at most 2 MiB".to_owned(),
            ),
        }
    }
}
fn body<T>(body: Result<Json<T>, JsonRejection>) -> Result<T, ApiError> {
    body.map(|Json(value)| value).map_err(|error| {
        let (code, message) = match error.status() {
            StatusCode::PAYLOAD_TOO_LARGE => {
                ("request_too_large", "Recipe request exceeds 128 KiB")
            }
            StatusCode::UNSUPPORTED_MEDIA_TYPE => ("json_required", "Send application/json"),
            _ => (
                "invalid_recipe",
                "Check the recipe fields, decimal gram strings, steps and integer servings",
            ),
        };
        ApiError(error.status(), code, message.to_owned())
    })
}
async fn blocking<T: Send + 'static>(
    state: Shared,
    action: impl FnOnce(&mut Store, &Catalogue) -> Result<T, recipes::Error> + Send + 'static,
) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(move || {
        let mut store = state.store.lock().map_err(|_| recipes::Error::Storage)?;
        action(&mut store, &state.catalogue)
    })
    .await
    .map_err(|_| ApiError::from(recipes::Error::Storage))?
    .map_err(ApiError::from)
}
fn response(record: &Record, status: StatusCode) -> Result<Response, ApiError> {
    let nutrition = nutrition::calculate(&record.catalogue, &record.recipe.calculation())
        .map_err(|_| ApiError::from(recipes::Error::Storage))?;
    Ok((
        status,
        [(header::CACHE_CONTROL, "no-store")],
        Json(json!({
            "id":record.id,"revision":record.revision,"recipe":record.recipe,
            "nutrition":nutrition,
        })),
    )
        .into_response())
}
async fn list(State(state): State<Shared>) -> Result<Response, ApiError> {
    let recipes = blocking(state, |store, _| Ok(store.list())).await?;
    Ok((
        [(header::CACHE_CONTROL, "no-store")],
        Json(json!({"recipes":recipes})),
    )
        .into_response())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RevisionQuery {
    revision: Option<u64>,
}
async fn detail(
    State(state): State<Shared>,
    Path(id): Path<String>,
    query: Result<Query<RevisionQuery>, axum::extract::rejection::QueryRejection>,
) -> Result<Response, ApiError> {
    let Query(query) = query.map_err(|_| {
        ApiError(
            StatusCode::BAD_REQUEST,
            "invalid_revision",
            "Use an integer revision query".to_owned(),
        )
    })?;
    let record = blocking(state, move |store, _| store.get(&id, query.revision)).await?;
    response(&record, StatusCode::OK)
}
async fn create(
    State(state): State<Shared>,
    input: Result<Json<Recipe>, JsonRejection>,
) -> Result<Response, ApiError> {
    let recipe = body(input)?;
    let record = blocking(state, move |store, catalogue| {
        store.save(recipe, None, catalogue)
    })
    .await?;
    response(&record, StatusCode::CREATED)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Update {
    expected_revision: u64,
    recipe: Recipe,
}
async fn update(
    State(state): State<Shared>,
    Path(id): Path<String>,
    input: Result<Json<Update>, JsonRejection>,
) -> Result<Response, ApiError> {
    let input = body(input)?;
    let record = blocking(state, move |store, catalogue| {
        store.save(
            input.recipe,
            Some((&id, input.expected_revision)),
            catalogue,
        )
    })
    .await?;
    response(&record, StatusCode::OK)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScaleQuery {
    servings: u32,
    revision: Option<u64>,
}
async fn scale(
    State(state): State<Shared>,
    Path(id): Path<String>,
    query: Result<Query<ScaleQuery>, axum::extract::rejection::QueryRejection>,
) -> Result<Response, ApiError> {
    let Query(query) = query.map_err(|_| {
        ApiError(
            StatusCode::BAD_REQUEST,
            "invalid_recipe",
            "Provide integer servings and optional revision".to_owned(),
        )
    })?;
    if query.servings == 0 || query.servings > 1_000_000 {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_recipe",
            "Servings must be between 1 and 1000000".to_owned(),
        ));
    }
    let record = blocking(state, move |store, _| store.get(&id, query.revision)).await?;
    let original = record.recipe.servings;
    let mut calculation = record.recipe.calculation();
    for line in &mut calculation.lines {
        line.grams.0 = line
            .grams
            .0
            .scaled(query.servings, original)
            .map_err(|e| ApiError(StatusCode::UNPROCESSABLE_ENTITY, "invalid_recipe", e))?;
    }
    // Preview lines are normalized masses; original inputs remain in the saved revision.
    for line in &mut calculation.lines {
        line.quantity = None;
        line.conversion = None;
    }
    calculation.servings = query.servings;
    let nutrition = nutrition::scaled_nutrition(
        &record.catalogue,
        &record.recipe.calculation(),
        query.servings,
    )
    .map_err(|e| ApiError(StatusCode::UNPROCESSABLE_ENTITY, "invalid_recipe", e))?;
    Ok(([(header::CACHE_CONTROL,"no-store")], Json(json!({"id":record.id,"revision":record.revision,"original_servings":original,"servings":query.servings,"lines":calculation.lines,"nutrition":nutrition,"rounding":"grams: nearest 0.000001 g, half up; nutrition: exact original serving ratio before display rounding"}))).into_response())
}
