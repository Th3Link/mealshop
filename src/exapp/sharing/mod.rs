//! Explicit, read-only recipe grants; unrelated household data stays behind its own boundary.
mod access;
mod manage;
mod people;
mod read;
use super::{Failure, Shared};
use axum::{
    Json, Router,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde_json::Value;

pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route(
            "/api/households/{home}/shares",
            get(manage::list).post(manage::create),
        )
        .route(
            "/api/households/{home}/shares/{id}",
            axum::routing::put(manage::publish).delete(manage::revoke),
        )
        .route("/api/shared", get(read::list))
        .route(
            "/api/shared/{home}/{id}",
            get(read::detail).delete(manage::decline),
        )
        .route(
            "/api/shared/{home}/{id}/accept",
            axum::routing::put(manage::accept),
        )
        .route("/api/shared/{home}/{id}/recipes/{recipe}", get(read::entry))
        .route("/api/shared/{home}/{id}/image", get(read::image))
        .route(
            "/api/shared/{home}/{id}/recipes/{recipe}/image",
            get(read::entry_image),
        )
}
fn failure(code: &'static str) -> Failure {
    Failure(
        match code {
            "share_owner_required" | "household_owner_required" => StatusCode::FORBIDDEN,
            "share_not_found" => StatusCode::NOT_FOUND,
            "share_exists" | "revision_conflict" | "share_changed" => StatusCode::CONFLICT,
            "library_capacity" => StatusCode::INSUFFICIENT_STORAGE,
            "library_storage_failed" | "share_user_lookup_failed" => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            _ => StatusCode::UNPROCESSABLE_ENTITY,
        },
        code,
    )
}
fn response(value: Value) -> Response {
    ([(header::CACHE_CONTROL, "no-store")], Json(value)).into_response()
}
