//! Serves public catalogue records and stateless nutrition estimates.

use crate::nutrition::{
    self, Calculation,
    catalogue::{Catalogue, Ingredient},
    nutrients::{Nutrient, NutrientInfo},
};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Query, State, rejection::JsonRejection},
    http::{StatusCode, header},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use serde::Serialize;
use serde_json::json;
use std::{collections::BTreeMap, sync::Arc};

type SharedCatalogue = Arc<Catalogue>;

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    error: String,
}

struct ApiError(StatusCode, String);

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let code = match self.0 {
            StatusCode::NOT_FOUND => "ingredient_not_found",
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => "invalid_recipe",
            StatusCode::PAYLOAD_TOO_LARGE => "request_too_large",
            StatusCode::UNSUPPORTED_MEDIA_TYPE => "json_required",
            _ => "request_failed",
        };
        (
            self.0,
            Json(ErrorBody {
                code,
                error: self.1,
            }),
        )
            .into_response()
    }
}

pub(crate) fn router(catalogue: Catalogue) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok\n" }))
        .route("/", get(|| async { Html(include_str!("web/index.html")) }))
        .route(
            "/app.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("web/demo.mjs"),
                )
            }),
        )
        .route(
            "/style.css",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
                    include_str!("web/style.css"),
                )
            }),
        )
        .route(
            "/i18n.mjs",
            get(|| async { javascript(include_str!("web/i18n.mjs")) }),
        )
        .route(
            "/locales/en.mjs",
            get(|| async { javascript(include_str!("web/locales/en.mjs")) }),
        )
        .route(
            "/locales/de.mjs",
            get(|| async { javascript(include_str!("web/locales/de.mjs")) }),
        )
        .route("/api/ingredients", get(list))
        .route("/api/ingredients/{id}", get(detail))
        .route("/api/nutrition/calculate", post(estimate))
        .layer(DefaultBodyLimit::max(131_072))
        .with_state(Arc::new(catalogue))
}

#[derive(Serialize)]
struct IngredientList<'a> {
    catalogue_version: &'a str,
    language: &'a str,
    ingredients: Vec<&'a Ingredient>,
    nutrient_definitions: Vec<NutrientInfo>,
}

fn javascript(source: &'static str) -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        source,
    )
}

async fn list(
    State(catalogue): State<SharedCatalogue>,
    Query(query): Query<BTreeMap<String, String>>,
) -> Json<serde_json::Value> {
    let search = query
        .get("q")
        .map_or(String::new(), |q| q.trim().to_lowercase());
    let ingredients = catalogue
        .ingredients
        .iter()
        .filter(|i| i.matches(&search))
        .collect();
    Json(json!(IngredientList {
        catalogue_version: &catalogue.version,
        language: &catalogue.language,
        ingredients,
        nutrient_definitions: Nutrient::ALL.iter().map(|n| n.info()).collect()
    }))
}

async fn detail(
    State(catalogue): State<SharedCatalogue>,
    Path(id): Path<String>,
) -> Result<Json<Ingredient>, ApiError> {
    catalogue
        .find(&id)
        .cloned()
        .map(Json)
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "Ingredient not found".to_owned()))
}

async fn estimate(
    State(catalogue): State<SharedCatalogue>,
    body: Result<Json<Calculation>, JsonRejection>,
) -> Result<Json<nutrition::Estimate>, ApiError> {
    let Json(recipe) = body.map_err(|error| {
        let message = match &error {
            JsonRejection::JsonDataError(_) => "Check the recipe fields: gram amounts must be decimal strings from 0 to 1000000000 with at most six decimal places; servings must be a whole number and all fields must be supported.",
            JsonRejection::JsonSyntaxError(_) => "The recipe body is not valid JSON.",
            JsonRejection::MissingJsonContentType(_) => "Send the recipe with content type application/json.",
            _ if error.status() == StatusCode::PAYLOAD_TOO_LARGE => "Recipe request is too large; the limit is 128 KiB.",
            _ => "The recipe request could not be read.",
        };
        ApiError(error.status(), message.to_owned())
    })?;
    nutrition::calculate(&catalogue, &recipe)
        .map(Json)
        .map_err(|message| ApiError(StatusCode::UNPROCESSABLE_ENTITY, message))
}
