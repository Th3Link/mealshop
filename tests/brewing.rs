//! F14 uses only isolated synthetic households; no retained personal records are fixtures.
#![allow(clippy::unwrap_used)]
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use tower::ServiceExt;
const OWNER: &str = "brewing_owner_synthetic_credential_123456789";
const READER: &str = "brewing_reader_synthetic_credential_12345678";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "mealshop-brewing-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn app(&self) -> Result<axum::Router, String> {
        mealshop_server::family_router(
            include_str!("../data/catalogue.json"),
            self.0.join("recipes"),
            self.0.join("household"),
            OWNER.to_owned(),
            Some(READER.to_owned()),
        )
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
async fn call(
    app: &axum::Router,
    token: Option<&str>,
    method: &str,
    path: &str,
    value: Value,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json");
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(value.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), 8_388_608).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}
#[tokio::test]
async fn protected_method_catalogue_keeps_partial_mash_biab_and_named_systems_distinct() {
    let directory = Directory::new();
    let app = directory.app().unwrap();
    let path = "/api/household/brewing/templates";
    assert_eq!(
        call(&app, None, "GET", path, Value::Null).await.0,
        StatusCode::UNAUTHORIZED
    );
    let (status, catalogue) = call(&app, Some(READER), "GET", path, Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(catalogue["templates"].as_array().unwrap().len(), 12);
    let template = |id| {
        catalogue["templates"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["id"] == id)
            .unwrap()
    };
    assert_eq!(template(json!("biab"))["strategy"], "all_grain");
    assert_eq!(template(json!("partial-mash"))["strategy"], "partial_mash");
    assert_ne!(
        template(json!("grainfather"))["layout"],
        template(json!("braumeister"))["layout"]
    );
    assert!(
        catalogue["sources"]
            .as_array()
            .unwrap()
            .iter()
            .all(|s| s["url"].as_str().unwrap().starts_with("https://"))
    );
}
