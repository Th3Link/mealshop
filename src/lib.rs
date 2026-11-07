//! HTTP entry points for the Mealshop server bootstrap.

use axum::{Router, routing::get};

/// Builds the public bootstrap API; no product data is stored yet.
pub fn router() -> Router {
    Router::new().route("/health", get(|| async { "ok\n" }))
}

/// Project documentation published together with the API reference.
pub mod docs;
