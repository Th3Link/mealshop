#![allow(clippy::unwrap_used)]
//! Acceptance checks for the bootstrap HTTP contract.

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn health_is_ok_without_storage() {
    let response = mealshop_server::router()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(to_bytes(response.into_body(), 1024).await.unwrap(), "ok\n");
}

#[tokio::test]
async fn unknown_routes_are_not_found() {
    let response = mealshop_server::router()
        .oneshot(
            Request::builder()
                .uri("/recipes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
