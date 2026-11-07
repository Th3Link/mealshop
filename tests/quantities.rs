//! Exact unit conversion contract.
#![allow(clippy::unwrap_used)]
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn convert(amount: &str, from: &str, to: &str) -> (StatusCode, Value) {
    let response = mealshop_server::router()
        .unwrap()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/quantities/convert")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"quantity":{"amount":amount,"unit":from},"target":to}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    (
        status,
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap(),
    )
}
#[tokio::test]
async fn preserves_input_and_converts_only_within_known_dimensions() {
    let (status, body) = convert("1.250000", "kg", "g").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["original"], json!({"amount":"1.250000","unit":"kg"}));
    assert_eq!(body["dimension"], "mass");
    assert_eq!(
        body["converted"],
        json!({"amount":"1250.000000","unit":"g"})
    );
    assert_eq!(
        convert("250", "ml", "l").await.1["converted"]["amount"],
        "0.250000"
    );
    assert_eq!(
        convert("1", "ml", "g").await.1["code"],
        "ingredient_conversion_required"
    );
    assert_eq!(
        convert("1", "tsp", "ml").await.1["code"],
        "spoon_standard_required"
    );
    assert_eq!(
        convert("0.000001", "g", "kg").await.1["code"],
        "precision_exceeded"
    );
    assert_eq!(convert("1", "piece", "piece").await.1["dimension"], "count");
    assert_eq!(convert("1,5", "g", "kg").await.0, StatusCode::BAD_REQUEST);
}
