//! Acceptance tests for F43 slice 1 (criteria 1–4 and 13).
#![allow(clippy::unwrap_used, clippy::expect_used, missing_docs)]

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use mealshop_interdata::{Config, MAX_BODY_BYTES, router};
use serde_json::{Value, json};
use std::path::PathBuf;
use tower::ServiceExt;

const SECRET: &str = "test-secret-test-secret-test-secret";

fn dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("interdata-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    path
}

fn app_in(data_dir: PathBuf, cdn: Option<&str>, capacity: usize, per_minute: u32) -> Router {
    router(Config {
        upload_secret: SECRET.to_owned(),
        data_dir,
        cdn_origin: cdn.map(str::to_owned),
        capacity,
        uploads_per_minute: per_minute,
    })
    .unwrap()
}

fn app(name: &str) -> Router {
    app_in(dir(name), Some("https://cdn.example"), 100, 100)
}

async fn send(app: &Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn upload(path: &str, key: &str, body: String) -> Request<Body> {
    Request::post(path)
        .header("authorization", format!("Bearer {SECRET}"))
        .header("content-type", "application/json")
        .header("idempotency-key", key)
        .body(Body::from(body))
        .unwrap()
}

async fn post(app: &Router, path: &str, key: &str, body: &Value) -> (StatusCode, Value) {
    send(app, upload(path, key, body.to_string())).await
}

fn recipe() -> Value {
    json!({
        "license": "CC0-1.0",
        "title": "Linsensuppe",
        "language": "de",
        "servings": 4,
        "ingredients": [
            {"text": "250 g rote Linsen", "ingredient_id": "red-lentils", "amount": "250", "unit": "g"},
            {"text": "1 Prise Salz"}
        ],
        "steps": [{"text": "Alles 20 Minuten kochen."}],
        "tags": ["vegan", "fatty-liver"],
        "image_links": [{"url": "https://cdn.example/r/1.jpg", "alt": "Suppe im Teller"}]
    })
}

type Change = Box<dyn Fn(&mut Value)>;

const RECIPES: &str = "/v1/submissions/recipes";
const INGREDIENTS: &str = "/v1/submissions/ingredients";
const KEY: &str = "0123456789abcdef-1";

fn code(body: &Value) -> &str {
    body["error"]["code"].as_str().unwrap_or_default()
}

#[tokio::test]
async fn upload_is_buffered_unchanged_and_readable() {
    let app = app("buffered");
    // Unusual spacing and key order must survive: the original is kept as sent.
    let original = r#"{ "ingredients":[{"text":"1 Ei"}],   "title":"Ei", "license":"CC0-1.0" }"#;
    let (status, body) = send(&app, upload(RECIPES, KEY, original.to_owned())).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["status"], "received");
    assert_eq!(body["kind"], "recipe");
    let id = body["id"].as_str().unwrap().to_owned();

    let stored = std::fs::read_to_string(
        dir_path("buffered")
            .join("submissions")
            .join(format!("{id}.json")),
    )
    .unwrap();
    let record: Value = serde_json::from_str(&stored).unwrap();
    assert_eq!(record["original"], original);

    let request = Request::get(format!("/v1/submissions/{id}"))
        .header("authorization", format!("Bearer {SECRET}"))
        .body(Body::empty())
        .unwrap();
    let (status, body) = send(&app, request).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["content"]["title"], "Ei");
    assert_eq!(body["status"], "received");
}

fn dir_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("interdata-{}-{name}", std::process::id()))
}

#[tokio::test]
async fn full_recipe_and_ingredient_are_accepted() {
    let app = app("full");
    let (status, _) = post(&app, RECIPES, KEY, &recipe()).await;
    assert_eq!(status, StatusCode::CREATED);
    let ingredient = json!({
        "name": "Red lentils",
        "aliases": ["Rote Linsen"],
        "category": "legumes",
        "source": {
            "name": "Example table", "url": "https://data.example/1", "record_id": "1",
            "version": "4.0", "retrieved_on": "2026-11-28", "license": "CC-BY-4.0"
        },
        "nutrients_per_100g": {"protein": {"amount": "23.9", "unit": "g"}},
        "tags": ["vegan"]
    });
    let (status, body) = post(&app, INGREDIENTS, "0123456789abcdef-2", &ingredient).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["kind"], "ingredient");
}

#[tokio::test]
async fn upload_needs_the_secret() {
    let app = app("secret");
    for authorization in [None, Some("Bearer wrong"), Some(SECRET)] {
        let mut request = Request::post(RECIPES)
            .header("content-type", "application/json")
            .header("idempotency-key", KEY);
        if let Some(value) = authorization {
            request = request.header("authorization", value);
        }
        let request = request.body(Body::from(recipe().to_string())).unwrap();
        let (status, body) = send(&app, request).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(code(&body), "upload_secret_required");
    }
    let request = Request::get("/v1/submissions/sub-00000000000000000000000000000000")
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(&app, request).await.0, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn short_secret_and_bad_cdn_origin_refuse_to_start() {
    let config = |secret: &str, cdn: Option<&str>| Config {
        upload_secret: secret.to_owned(),
        data_dir: dir("start"),
        cdn_origin: cdn.map(str::to_owned),
        capacity: 1,
        uploads_per_minute: 1,
    };
    assert!(router(config("short", None)).is_err());
    assert!(router(config(SECRET, Some("http://cdn.example"))).is_err());
}

#[tokio::test]
async fn replay_returns_the_same_submission_and_changed_body_conflicts() {
    let app = app("replay");
    let (_, first) = post(&app, RECIPES, KEY, &recipe()).await;
    let (status, again) = post(&app, RECIPES, KEY, &recipe()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(again["id"], first["id"]);

    let mut changed = recipe();
    changed["title"] = json!("Andere Suppe");
    let (status, body) = post(&app, RECIPES, KEY, &changed).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(code(&body), "idempotency_conflict");
}

#[tokio::test]
async fn replay_survives_restart() {
    let data = dir("restart");
    let first = app_in(data.clone(), None, 100, 100);
    let body = json!({"license": "CC0-1.0", "title": "Ei", "ingredients": [{"text": "1 Ei"}]});
    let (_, created) = post(&first, RECIPES, KEY, &body).await;
    let second = app_in(data, None, 100, 100);
    let (status, again) = post(&second, RECIPES, KEY, &body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(again["id"], created["id"]);
}

#[tokio::test]
async fn idempotency_key_is_required_and_safe() {
    let app = app("key");
    for key in [
        "short",
        "../../../../etc/passwd-0123",
        "has space 0123456789",
    ] {
        let (status, body) = post(&app, RECIPES, key, &recipe()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{key}");
        assert_eq!(code(&body), "idempotency_key_required");
    }
}

#[tokio::test]
async fn invalid_recipes_are_rejected_with_a_reason() {
    let app = app("invalid");
    let cases: Vec<(&str, Change)> = vec![
        ("invalid_recipe", Box::new(|r| r["title"] = json!("  "))),
        (
            "license_required",
            Box::new(|r| r["license"] = json!("CC-BY-4.0")),
        ),
        ("license_required", Box::new(|r| r["license"] = Value::Null)),
        (
            "invalid_ingredients",
            Box::new(|r| r["ingredients"] = json!([])),
        ),
        (
            "invalid_ingredients",
            Box::new(|r| r["ingredients"] = json!([{"amount": "1"}])),
        ),
        (
            "invalid_amount",
            Box::new(|r| r["ingredients"][0]["amount"] = json!("1/2")),
        ),
        (
            "invalid_json",
            Box::new(|r| r["ingredients"][0]["amount"] = json!(0.5)),
        ),
        ("invalid_json", Box::new(|r| r["surprise"] = json!(true))),
        (
            "invalid_tags",
            Box::new(|r| r["tags"] = json!(["Gut bei Fettleber"])),
        ),
        (
            "invalid_source_url",
            Box::new(|r| r["source_url"] = json!("http://example.org/r")),
        ),
        (
            "invalid_image_links",
            Box::new(|r| r["image_links"][0]["url"] = json!("https://evil.example/1.jpg")),
        ),
        (
            "invalid_image_links",
            Box::new(|r| r["image_links"][0]["url"] = json!("https://cdn.example.evil.test/1.jpg")),
        ),
    ];
    for (index, (expected, change)) in cases.iter().enumerate() {
        let mut body = recipe();
        change(&mut body);
        let key = format!("0123456789abcdef-{index}");
        let (status, answer) = post(&app, RECIPES, &key, &body).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "case {index}");
        assert_eq!(code(&answer), *expected, "case {index}");
        assert_ne!(answer["error"]["message"].as_str().unwrap(), "");
    }
    assert_eq!(
        std::fs::read_dir(dir_path("invalid").join("submissions"))
            .unwrap()
            .count(),
        0
    );
}

#[tokio::test]
async fn image_links_wait_for_a_cdn_origin() {
    let app = app_in(dir("nocdn"), None, 100, 100);
    let (status, body) = post(&app, RECIPES, KEY, &recipe()).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(code(&body), "image_links_unavailable");
}

#[tokio::test]
async fn ingredient_nutrients_need_a_source() {
    let app = app("ingredient");
    let body =
        json!({"name": "Mystery", "nutrients_per_100g": {"protein": {"amount": "1", "unit": "g"}}});
    let (status, answer) = post(&app, INGREDIENTS, KEY, &body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(code(&answer), "nutrients_need_source");
    let (status, _) = post(&app, INGREDIENTS, KEY, &json!({"name": "Mystery"})).await;
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn oversized_and_non_json_uploads_are_refused() {
    let app = app("bounds");
    let big = format!(r#"{{"title":"{}"}}"#, "a".repeat(MAX_BODY_BYTES));
    let (status, _) = send(&app, upload(RECIPES, KEY, big)).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);

    let request = Request::post(RECIPES)
        .header("authorization", format!("Bearer {SECRET}"))
        .header("content-type", "text/plain")
        .header("idempotency-key", KEY)
        .body(Body::from(recipe().to_string()))
        .unwrap();
    let (status, body) = send(&app, request).await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(code(&body), "json_required");
}

#[tokio::test]
async fn rate_limit_and_capacity_refuse_new_uploads_but_not_replays() {
    let body = json!({"license": "CC0-1.0", "title": "Ei", "ingredients": [{"text": "1 Ei"}]});

    let limited = app_in(dir("rate"), None, 100, 1);
    assert_eq!(
        post(&limited, RECIPES, KEY, &body).await.0,
        StatusCode::CREATED
    );
    let (status, answer) = post(&limited, RECIPES, "0123456789abcdef-2", &body).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(code(&answer), "too_many_uploads");
    assert_eq!(post(&limited, RECIPES, KEY, &body).await.0, StatusCode::OK);

    let full = app_in(dir("capacity"), None, 1, 100);
    assert_eq!(
        post(&full, RECIPES, KEY, &body).await.0,
        StatusCode::CREATED
    );
    let (status, answer) = post(&full, RECIPES, "0123456789abcdef-2", &body).await;
    assert_eq!(status, StatusCode::INSUFFICIENT_STORAGE);
    assert_eq!(code(&answer), "buffer_full");
}

#[tokio::test]
async fn unknown_submission_is_not_found() {
    let app = app("missing");
    for id in ["sub-00000000000000000000000000000000", "..%2Fkeys%2Fx"] {
        let request = Request::get(format!("/v1/submissions/{id}"))
            .header("authorization", format!("Bearer {SECRET}"))
            .body(Body::empty())
            .unwrap();
        let (status, body) = send(&app, request).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(code(&body), "submission_not_found");
    }
}
