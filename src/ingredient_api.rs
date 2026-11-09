//! Authenticated household ingredient management and reverse recipe lookup.
use crate::{
    household::{self, Shared},
    ingredients::{Annotation, Declaration},
    nutrition::nutrients::Nutrient,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::get,
};
use serde::Deserialize;
use serde_json::json;

pub(crate) fn routes() -> Router<Shared> {
    Router::new()
        .route("/api/household/ingredients", get(list).post(create))
        .route(
            "/api/household/ingredients/{id}",
            axum::routing::put(annotate).delete(archive),
        )
        .route("/api/household/ingredients/{id}/recipes", get(recipes))
}
fn failure(code: &'static str) -> Response {
    let status = match code {
        "revision_conflict" => StatusCode::CONFLICT,
        "ingredient_storage_failed" => StatusCode::SERVICE_UNAVAILABLE,
        "ingredient_capacity" => StatusCode::INSUFFICIENT_STORAGE,
        _ => StatusCode::UNPROCESSABLE_ENTITY,
    };
    household::reply(status, json!({"code":code}))
}
async fn list(State(state): State<Shared>, headers: HeaderMap) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    let Ok(store) = state.catalogue.lock() else {
        return failure("ingredient_storage_failed");
    };
    let catalogue = store.catalogue();
    household::reply(
        StatusCode::OK,
        json!({"revision":store.revision(), "catalogue_version":catalogue.version,"language":catalogue.language,"ingredients":catalogue.ingredients,"nutrient_definitions":Nutrient::ALL.iter().map(|n|n.info()).collect::<Vec<_>>()}),
    )
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Create {
    expected_revision: u64,
    ingredient: Declaration,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Update {
    expected_revision: u64,
    annotations: Annotation,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Revision {
    expected_revision: u64,
}
async fn create(
    State(state): State<Shared>,
    headers: HeaderMap,
    body: Result<Json<Create>, axum::extract::rejection::JsonRejection>,
) -> Response {
    if !household::can_write(&state, &headers) {
        return household::reply(
            StatusCode::FORBIDDEN,
            json!({"code":"household_write_required"}),
        );
    }
    let Ok(Json(input)) = body else {
        return failure("invalid_ingredient");
    };
    tokio::task::spawn_blocking(move || {
        let Ok(mut store) = state.catalogue.lock() else {
            return failure("ingredient_storage_failed");
        };
        match store.create(input.ingredient, input.expected_revision) {
            Ok(id) => household::reply(
                StatusCode::CREATED,
                json!({"id":id,"revision":store.revision()}),
            ),
            Err(code) => failure(code),
        }
    })
    .await
    .unwrap_or_else(|_| failure("ingredient_storage_failed"))
}
async fn annotate(
    State(state): State<Shared>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Result<Json<Update>, axum::extract::rejection::JsonRejection>,
) -> Response {
    if !household::can_write(&state, &headers) {
        return household::reply(
            StatusCode::FORBIDDEN,
            json!({"code":"household_write_required"}),
        );
    }
    let Ok(Json(input)) = body else {
        return failure("invalid_ingredient");
    };
    persist(state, id, input.annotations, input.expected_revision).await
}
async fn archive(
    State(state): State<Shared>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Result<Json<Revision>, axum::extract::rejection::JsonRejection>,
) -> Response {
    if !household::can_write(&state, &headers) {
        return household::reply(
            StatusCode::FORBIDDEN,
            json!({"code":"household_write_required"}),
        );
    }
    let Ok(Json(input)) = body else {
        return failure("invalid_ingredient");
    };
    let annotation = {
        let Ok(store) = state.catalogue.lock() else {
            return failure("ingredient_storage_failed");
        };
        let Some(i) = store.catalogue().find(&id).cloned() else {
            return failure("invalid_ingredient");
        };
        Annotation {
            tags: i.tags,
            components: i.components,
            declared_allergens: i.declared_allergens,
            archived: true,
            favorite: i.favorite,
        }
    };
    persist(state, id, annotation, input.expected_revision).await
}
async fn persist(state: Shared, id: String, annotation: Annotation, expected: u64) -> Response {
    tokio::task::spawn_blocking(move || {
        let Ok(mut store) = state.catalogue.lock() else {
            return failure("ingredient_storage_failed");
        };
        match store.annotate(id, annotation, expected) {
            Ok(()) => household::reply(StatusCode::OK, json!({"revision":store.revision()})),
            Err(code) => failure(code),
        }
    })
    .await
    .unwrap_or_else(|_| failure("ingredient_storage_failed"))
}
async fn recipes(
    State(state): State<Shared>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    tokio::task::spawn_blocking(move || {
        let Some(recipes) = &state.recipes else {
            return household::reply(StatusCode::OK, json!({"recipes":[]}));
        };
        let Ok(recipes) = recipes.lock() else {
            return failure("ingredient_storage_failed");
        };
        household::reply(StatusCode::OK, json!({"recipes":recipes.containing(&id)}))
    })
    .await
    .unwrap_or_else(|_| failure("ingredient_storage_failed"))
}
