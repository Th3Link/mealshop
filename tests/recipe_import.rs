//! Reviewed import publication, role boundaries and durable source provenance.
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
const OWNER: &str = "import_owner_test_credential_123456789";
const READER: &str = "import_reader_test_credential_12345678";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "mealshop-import-api-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn app(&self) -> axum::Router {
        mealshop_server::family_router(
            include_str!("../data/catalogue.json"),
            self.0.join("recipes"),
            self.0.join("household"),
            OWNER.to_owned(),
            Some(READER.to_owned()),
        )
        .unwrap()
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
    body: Value,
) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json");
    if let Some(token) = token {
        req = req.header("authorization", format!("Bearer {token}"));
    }
    let reply = app
        .clone()
        .oneshot(req.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = reply.status();
    let body = to_bytes(reply.into_body(), 4_194_304).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}
async fn preview(app: &axum::Router) -> String {
    let (status, value) = call(
        app,
        Some(OWNER),
        "POST",
        "/api/household/imports/preview",
        json!({"mode":"manual","url":"https://example.org/import-original"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["method"], "manual");
    assert!(value["retrieved_at"].is_null());
    assert!(value["candidates"][0]["yield_text"].is_null());
    value["preview_id"].as_str().unwrap().to_owned()
}
fn recipe() -> Value {
    json!({"title":"Import acceptance recipe","language":"en","servings":2,"preparation":"as_recorded","origin":{"kind":"imported","source_url":"https://example.org/import-original"},"lines":[{"ingredient_id":"oats","quantity":{"amount":"0.100500","unit":"kg"}}],"steps":[{"instruction":"Prepare synthetic fixture","active_seconds":60,"passive_seconds":0}]})
}
fn save(id: &str) -> Value {
    json!({"preview_id":id,"candidate":0,"idempotency_key":"synthetic-import-request-0001","reviewed":true,"recipe":recipe()})
}
#[tokio::test]
async fn import_requires_current_write_authority_consent_and_public_fetch_targets() {
    let dir = Directory::new();
    let app = dir.app();
    for token in [None, Some(READER)] {
        assert!(
            !call(
                &app,
                token,
                "POST",
                "/api/household/imports/preview",
                json!({"mode":"manual","url":"https://example.org/source"})
            )
            .await
            .0
            .is_success()
        );
    }
    let denied = call(
        &app,
        Some(OWNER),
        "POST",
        "/api/household/imports/preview",
        json!({"mode":"url","url":"https://example.org/source","consent":false}),
    )
    .await;
    assert_eq!(denied.1["code"], "import_consent_required");
    for url in [
        "https://127.0.0.1/",
        "https://[::1]/",
        "https://169.254.169.254/",
        "http://example.org/",
        "https://example.org:8080/",
    ] {
        assert_eq!(
            call(
                &app,
                Some(OWNER),
                "POST",
                "/api/household/imports/preview",
                json!({"mode":"url","url":url,"consent":true})
            )
            .await
            .1["code"],
            "import_url_invalid"
        );
    }
    let id = preview(&app).await;
    assert_eq!(
        call(
            &app,
            Some(READER),
            "POST",
            "/api/household/imports",
            save(&id)
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let mut unreviewed = save(&id);
    unreviewed["reviewed"] = json!(false);
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/imports",
            unreviewed
        )
        .await
        .1["code"],
        "import_review_required"
    );
    assert_eq!(
        call(&app, Some(OWNER), "GET", "/api/recipes", Value::Null)
            .await
            .1,
        json!({"recipes":[]})
    );
}
#[tokio::test]
async fn reviewed_import_is_atomic_idempotent_across_restart_and_rejects_changed_payloads() {
    let dir = Directory::new();
    let app = dir.app();
    let id = preview(&app).await;
    let body = save(&id);
    let (one, two) = tokio::join!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/imports",
            body.clone()
        ),
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/imports",
            body.clone()
        )
    );
    assert_eq!(one.0, StatusCode::CREATED);
    assert_eq!(two.1["id"], one.1["id"]);
    assert_eq!(one.1["recipe"]["lines"][0]["grams"], "100.500000");
    assert_eq!(
        one.1["import_source"]["requested_url"],
        "https://example.org/import-original"
    );
    assert!(!one.1.to_string().contains("synthetic-import-request-0001"));
    let mut changed = body.clone();
    changed["recipe"]["title"] = json!("Changed import intent");
    assert_eq!(
        call(&app, Some(OWNER), "POST", "/api/household/imports", changed)
            .await
            .0,
        StatusCode::CONFLICT
    );
    drop(app);
    let app = dir.app();
    assert_eq!(
        call(&app, Some(OWNER), "POST", "/api/household/imports", body)
            .await
            .1["id"],
        one.1["id"]
    );
    assert_eq!(
        call(&app, Some(OWNER), "GET", "/api/recipes", Value::Null)
            .await
            .1["recipes"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let missing = save(&format!("import-{}", "f".repeat(64)));
    assert_eq!(
        call(&app, Some(OWNER), "POST", "/api/household/imports", missing)
            .await
            .0,
        StatusCode::CONFLICT
    );
}
#[tokio::test]
async fn imported_provenance_survives_edits_and_cannot_be_replaced_by_owned_or_other_source() {
    let dir = Directory::new();
    let app = dir.app();
    let id = preview(&app).await;
    let saved = call(
        &app,
        Some(OWNER),
        "POST",
        "/api/household/imports",
        save(&id),
    )
    .await
    .1;
    let path = format!("/api/recipes/{}", saved["id"].as_str().unwrap());
    let mut edited = saved["recipe"].clone();
    edited["title"] = json!("Revised import acceptance recipe");
    let result = call(
        &app,
        Some(OWNER),
        "PUT",
        &path,
        json!({"expected_revision":1,"recipe":edited}),
    )
    .await;
    assert_eq!(result.0, StatusCode::OK);
    assert_eq!(result.1["import_source"], saved["import_source"]);
    for origin in [
        json!({"kind":"owned"}),
        json!({"kind":"imported","source_url":"https://example.org/replaced"}),
    ] {
        let mut changed = result.1["recipe"].clone();
        changed["origin"] = origin;
        assert_eq!(
            call(
                &app,
                Some(OWNER),
                "PUT",
                &path,
                json!({"expected_revision":2,"recipe":changed})
            )
            .await
            .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "GET",
            &format!("{path}?revision=1"),
            Value::Null
        )
        .await
        .1["import_source"],
        saved["import_source"]
    );
}
