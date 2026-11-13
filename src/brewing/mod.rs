//! Source-linked beer workflows use the existing protected durable process engine.
mod api;
pub(crate) mod model;
mod operations;
pub(crate) mod templates;
pub(crate) mod validation;
mod view;
use crate::household::{self, Shared};
use axum::{
    Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::get,
};
pub(crate) fn routes() -> Router<Shared> {
    Router::new()
        .route("/api/household/brewing/templates", get(read_templates))
        .merge(api::routes())
}
async fn read_templates(State(state): State<Shared>, headers: HeaderMap) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    match templates::catalogue() {
        Ok(catalogue) => household::reply(StatusCode::OK, serde_json::json!(catalogue)),
        Err(code) => household::reply(
            StatusCode::SERVICE_UNAVAILABLE,
            serde_json::json!({"code":code}),
        ),
    }
}
