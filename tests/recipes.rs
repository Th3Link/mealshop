//! Black-box acceptance coverage for durable, versioned local recipes.
#![allow(clippy::unwrap_used)]

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use tower::ServiceExt;

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "mealshop-recipes-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn app(&self) -> axum::Router {
        self.with_catalogue(include_str!("../data/catalogue.json"))
    }
    fn with_catalogue(&self, json: &str) -> axum::Router {
        mealshop_server::router_with_recipe_directory(json, &self.0).unwrap()
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
async fn request(
    app: &axum::Router,
    method: &str,
    path: &str,
    value: Value,
) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("content-type", "application/json")
                .body(if method == "GET" {
                    Body::empty()
                } else {
                    Body::from(value.to_string())
                })
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    assert_eq!(response.headers()["cache-control"], "no-store");
    let bytes = to_bytes(response.into_body(), 2_097_152).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}
fn recipe() -> Value {
    json!({"title":"Porridge","language":"en","author":"Local cook","servings":2,
        "preparation":"cooking_unmodelled","lines":[{"ingredient_id":"oats","grams":"100.500000"},{"ingredient_id":"chia","grams":"25"}],
        "steps":[{"instruction":"Mix and rest.","active_seconds":60,"passive_seconds":300}]})
}
fn nutrient<'a>(body: &'a Value, id: &str) -> &'a Value {
    body["nutrition"]["nutrients"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == id)
        .unwrap()
}

#[tokio::test]
async fn create_list_restart_and_retrieve_preserve_exact_inputs_and_portion_nutrition() {
    let dir = Directory::new();
    let app = dir.app();
    assert_eq!(
        request(&app, "GET", "/api/recipes", Value::Null).await.1,
        json!({"recipes":[]})
    );
    let (status, created) = request(&app, "POST", "/api/recipes", recipe()).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["revision"], 1);
    assert_eq!(created["recipe"], recipe());
    assert_eq!(nutrient(&created, "energy_kcal")["per_serving"], "256.22");
    let id = created["id"].as_str().unwrap();
    let list = request(&app, "GET", "/api/recipes", Value::Null).await.1;
    assert_eq!(list["recipes"][0]["id"], id);
    assert_eq!(list["recipes"][0]["title"], "Porridge");
    drop(app);
    let reopened = dir.app();
    let (status, read) =
        request(&reopened, "GET", &format!("/api/recipes/{id}"), Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(read, created);
    let disk: Value =
        serde_json::from_slice(&fs::read(dir.0.join(format!("{id}-1.json"))).unwrap()).unwrap();
    assert_eq!(disk["schema_version"], 1);
    assert_eq!(
        disk["catalogue"]["ingredients"].as_array().unwrap().len(),
        2
    );
    assert_eq!(disk["recipe"]["lines"][0]["grams"], "100.500000");
}

#[tokio::test]
async fn optimistic_revision_conflicts_preserve_history_and_latest_recipe() {
    let dir = Directory::new();
    let app = dir.app();
    let created = request(&app, "POST", "/api/recipes", recipe()).await.1;
    let path = format!("/api/recipes/{}", created["id"].as_str().unwrap());
    let mut edited = recipe();
    edited["title"] = json!("Porridge for four");
    edited["servings"] = json!(4);
    let input = json!({"expected_revision":1,"recipe":edited});
    let (a, b) = tokio::join!(
        request(&app, "PUT", &path, input.clone()),
        request(&app, "PUT", &path, input)
    );
    assert!(matches!(
        (a.0, b.0),
        (StatusCode::OK, StatusCode::CONFLICT) | (StatusCode::CONFLICT, StatusCode::OK)
    ));
    let latest = request(&app, "GET", &path, Value::Null).await.1;
    assert_eq!(latest["revision"], 2);
    assert_eq!(latest["recipe"], edited);
    assert_eq!(nutrient(&latest, "energy_kcal")["per_serving"], "128.11");
    assert_eq!(
        request(&app, "GET", &format!("{path}?revision=1"), Value::Null)
            .await
            .1,
        created
    );
    assert_eq!(
        request(&app, "GET", &format!("{path}?revision=99"), Value::Null)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    drop(app);
    let reopened = dir.app();
    assert_eq!(
        request(&reopened, "GET", &path, Value::Null).await.1,
        latest
    );
}

#[tokio::test]
async fn saved_sources_survive_catalogue_changes_and_new_revisions_use_current_sources() {
    let dir = Directory::new();
    let app = dir.app();
    let saved = request(&app, "POST", "/api/recipes", recipe()).await.1;
    let path = format!("/api/recipes/{}", saved["id"].as_str().unwrap());
    drop(app);
    let mut catalogue: Value =
        serde_json::from_str(include_str!("../data/catalogue.json")).unwrap();
    catalogue["version"] = json!("changed-for-test");
    catalogue["ingredients"][0]["nutrients"]["energy_kcal"]["amount"] = json!("1");
    let app = dir.with_catalogue(&catalogue.to_string());
    assert_eq!(request(&app, "GET", &path, Value::Null).await.1, saved);
    let updated = request(
        &app,
        "PUT",
        &path,
        json!({"expected_revision":1,"recipe":recipe()}),
    )
    .await
    .1;
    assert_eq!(
        updated["nutrition"]["catalogue_version"],
        "changed-for-test"
    );
    assert_ne!(
        nutrient(&updated, "energy_kcal")["per_serving"],
        nutrient(&saved, "energy_kcal")["per_serving"]
    );
    assert_eq!(
        request(&app, "GET", &format!("{path}?revision=1"), Value::Null)
            .await
            .1,
        saved
    );
}

#[tokio::test]
async fn invalid_recipes_never_create_records_and_unknown_composition_stays_unknown() {
    let dir = Directory::new();
    let app = dir.app();
    for (field, value) in [
        ("title", json!(" ")),
        ("author", json!("")),
        ("language", json!("xx")),
        ("servings", json!(0)),
        ("lines", json!([])),
        (
            "steps",
            json!([{"instruction":"x","active_seconds":604_801,"passive_seconds":0}]),
        ),
    ] {
        let mut invalid = recipe();
        invalid[field] = value;
        assert_eq!(
            request(&app, "POST", "/api/recipes", invalid).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    for grams in [json!("0"), json!("1,5"), json!(1.5), json!("0.0000001")] {
        let mut invalid = recipe();
        invalid["lines"][0]["grams"] = grams;
        assert!(
            request(&app, "POST", "/api/recipes", invalid)
                .await
                .0
                .is_client_error()
        );
    }
    let mut unknown = recipe();
    unknown["lines"][0]["ingredient_id"] = json!("nonexistent");
    assert_eq!(
        request(&app, "POST", "/api/recipes", unknown).await.0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        request(&app, "GET", "/api/recipes", Value::Null).await.1,
        json!({"recipes":[]})
    );
    let mut unknown = recipe();
    unknown["lines"][0]["ingredient_id"] = json!("psyllium-husks");
    let (status, saved) = request(&app, "POST", "/api/recipes", unknown).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(nutrient(&saved, "energy_kcal")["status"], "partial");
    assert!(nutrient(&saved, "energy_kcal")["per_serving"].is_null());
    assert_eq!(
        request(&app, "GET", "/api/recipes/recipe-99999999", Value::Null)
            .await
            .1["code"],
        "recipe_not_found"
    );
    assert_eq!(
        request(
            &app,
            "GET",
            "/api/recipes/recipe-00000001?revision=x",
            Value::Null
        )
        .await
        .1["code"],
        "invalid_revision"
    );
}

#[test]
fn exclusive_lock_corrupt_history_and_interrupted_writes_fail_closed() {
    let dir = Directory::new();
    let app = dir.app();
    assert!(
        mealshop_server::router_with_recipe_directory(
            include_str!("../data/catalogue.json"),
            &dir.0
        )
        .is_err()
    );
    drop(app);
    fs::write(dir.0.join(".pending"), "interrupted").unwrap();
    assert!(
        mealshop_server::router_with_recipe_directory(
            include_str!("../data/catalogue.json"),
            &dir.0
        )
        .is_err()
    );
    assert!(!dir.0.join(".mealshop.lock").exists());
    fs::remove_file(dir.0.join(".pending")).unwrap();
    fs::write(dir.0.join("recipe-00000001-1.json"), "corrupt").unwrap();
    assert!(
        mealshop_server::router_with_recipe_directory(
            include_str!("../data/catalogue.json"),
            &dir.0
        )
        .is_err()
    );
}

#[tokio::test]
async fn filesystem_failure_never_overwrites_a_revision_or_accepts_further_writes() {
    let dir = Directory::new();
    let app = dir.app();
    let target = dir.0.join("recipe-00000001-1.json");
    fs::write(&target, "sentinel").unwrap();
    let (status, error) = request(&app, "POST", "/api/recipes", recipe()).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(error["code"], "recipe_storage_failed");
    assert_eq!(fs::read_to_string(target).unwrap(), "sentinel");
    assert_eq!(
        request(&app, "GET", "/api/recipes", Value::Null).await.1,
        json!({"recipes":[]})
    );
    assert_eq!(
        request(&app, "POST", "/api/recipes", recipe()).await.0,
        StatusCode::SERVICE_UNAVAILABLE
    );
}
