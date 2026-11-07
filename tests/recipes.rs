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

#[tokio::test]
async fn custom_catalogue_snapshot_must_fit_the_restart_loader_before_publication() {
    let dir = Directory::new();
    let mut catalogue: Value =
        serde_json::from_str(include_str!("../data/catalogue.json")).unwrap();
    let mut oats = catalogue["ingredients"][0].clone();
    oats.as_object_mut().unwrap().remove("category");
    oats.as_object_mut().unwrap().remove("translations");
    let mut note = oats["education"][0].clone();
    note["summary"] = json!("x".repeat(4000));
    oats["education"] = json!([]);
    catalogue["ingredients"] = json!([oats]);
    catalogue.as_object_mut().unwrap().remove("language");
    let target = 1_048_575;
    while catalogue.to_string().len() + note.to_string().len() + 1 < target {
        catalogue["ingredients"][0]["education"]
            .as_array_mut()
            .unwrap()
            .push(note.clone());
    }
    for field in ["uncertainty", "form"] {
        let gap = target - catalogue.to_string().len();
        let previous = catalogue["ingredients"][0][field]
            .as_str()
            .unwrap()
            .to_owned();
        let padding = gap.min(4000 - previous.len());
        catalogue["ingredients"][0][field] = json!(previous + &"x".repeat(padding));
    }
    assert_eq!(catalogue.to_string().len(), target);
    let app = dir.with_catalogue(&catalogue.to_string());
    let mut input = recipe();
    input["lines"] = json!([{"ingredient_id":"oats","grams":"100"}]);
    assert_eq!(
        request(&app, "POST", "/api/recipes", input).await.0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        request(&app, "GET", "/api/recipes", Value::Null).await.1,
        json!({"recipes":[]})
    );
    drop(app);
    let _reopened = dir.app();
}

#[tokio::test]
async fn scaling_uses_original_revision_and_keeps_it_unchanged() {
    let dir = Directory::new();
    let app = dir.app();
    let created = request(&app, "POST", "/api/recipes", recipe()).await.1;
    let path = format!("/api/recipes/{}", created["id"].as_str().unwrap());
    let preview = request(
        &app,
        "GET",
        &format!("{path}/scale?servings=3"),
        Value::Null,
    )
    .await;
    assert_eq!(preview.0, StatusCode::OK);
    assert_eq!(preview.1["original_servings"], 2);
    assert_eq!(preview.1["lines"][0]["grams"], "150.750000");
    assert_eq!(
        request(
            &app,
            "GET",
            &format!("{path}/scale?servings=0"),
            Value::Null
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(request(&app, "GET", &path, Value::Null).await.1, created);
}

#[tokio::test]
async fn source_quantities_and_conversion_evidence_survive_restart() {
    let dir = Directory::new();
    let app = dir.app();
    let mut input = recipe();
    input["lines"] = json!([{"ingredient_id":"oats","quantity":{"amount":"0.100500","unit":"kg"}}]);
    let saved = request(&app, "POST", "/api/recipes", input).await;
    assert_eq!(saved.0, StatusCode::CREATED);
    assert_eq!(saved.1["recipe"]["lines"][0]["grams"], "100.500000");
    let path = format!("/api/recipes/{}", saved.1["id"].as_str().unwrap());
    drop(app);
    let app = dir.app();
    assert_eq!(request(&app, "GET", &path, Value::Null).await.1, saved.1);
    let catalogue: Value = serde_json::from_str(include_str!("../data/catalogue.json")).unwrap();
    let form = catalogue["ingredients"][0]["form"].clone();
    let mut input = recipe();
    input["lines"] = json!([{"ingredient_id":"oats","quantity":{"amount":"2","unit":"piece"},"conversion":{"ingredient_id":"oats","form":form,"source_url":"https://example.org/measurement","version":"test measurement","uncertainty":"Declared fixture, not reference data","grams_per_base_unit":"5","spoon_standard":null,"millilitres_per_spoon":null}}]);
    let saved = request(&app, "POST", "/api/recipes", input.clone()).await;
    assert_eq!(saved.0, StatusCode::CREATED);
    assert_eq!(saved.1["recipe"]["lines"][0]["grams"], "10.000000");
    input["lines"][0]["quantity"]["unit"] = json!("ml");
    assert_eq!(
        request(&app, "POST", "/api/recipes", input.clone()).await.1["recipe"]["lines"][0]["grams"],
        "10.000000"
    );
    input["lines"][0]["quantity"]["unit"] = json!("tsp");
    assert!(
        request(&app, "POST", "/api/recipes", input.clone())
            .await
            .0
            .is_client_error()
    );
    input["lines"][0]["conversion"]["spoon_standard"] = json!("Declared test teaspoon, 5 ml");
    input["lines"][0]["conversion"]["millilitres_per_spoon"] = json!("5");
    assert_eq!(
        request(&app, "POST", "/api/recipes", input.clone()).await.1["recipe"]["lines"][0]["grams"],
        "50.000000"
    );
    input["lines"][0]["conversion"]["ingredient_id"] = json!("chia");
    assert!(
        request(&app, "POST", "/api/recipes", input)
            .await
            .0
            .is_client_error()
    );
}

#[tokio::test]
async fn curated_reference_portions_round_trip_without_inventing_density() {
    let dir = Directory::new();
    let app = dir.app();
    let data: Value = serde_json::from_str(include_str!("../data/conversions.json")).unwrap();
    for entry in data["entries"].as_array().unwrap() {
        let mut input = recipe();
        input["lines"] = json!([{"ingredient_id":entry["evidence"]["ingredient_id"],"quantity":{"amount":"1","unit":entry["unit"]},"conversion":entry["evidence"]}]);
        let saved = request(&app, "POST", "/api/recipes", input).await;
        assert_eq!(saved.0, StatusCode::CREATED, "{}", entry["id"]);
        assert_eq!(
            saved.1["recipe"]["lines"][0]["conversion"],
            entry["evidence"]
        );
        let path = format!("/api/recipes/{}", saved.1["id"].as_str().unwrap());
        assert_eq!(request(&app, "GET", &path, Value::Null).await.1, saved.1);
    }
    drop(app);
    let app = dir.app();
    assert_eq!(
        request(&app, "GET", "/api/recipes", Value::Null).await.1["recipes"]
            .as_array()
            .unwrap()
            .len(),
        43
    );
}

#[tokio::test]
async fn scaling_keeps_rational_nutrition_and_original_yield_across_changes_and_restart() {
    let dir = Directory::new();
    let mut catalogue: Value =
        serde_json::from_str(include_str!("../data/catalogue.json")).unwrap();
    catalogue["ingredients"][0]["nutrients"]["energy_kcal"]["amount"] = json!("3000000");
    let app = dir.with_catalogue(&catalogue.to_string());
    let mut input = recipe();
    input["servings"] = json!(3);
    input["lines"] = json!([{"ingredient_id":"oats","grams":"1"}]);
    let created = request(&app, "POST", "/api/recipes", input).await.1;
    let path = format!("/api/recipes/{}", created["id"].as_str().unwrap());
    for target in [1, 7, 2, 1] {
        let (status, preview) = request(
            &app,
            "GET",
            &format!("{path}/scale?servings={target}"),
            Value::Null,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(preview["original_servings"], 3);
        assert_eq!(nutrient(&preview, "energy_kcal")["per_serving"], "10000.00");
        if target == 1 {
            assert_eq!(preview["lines"][0]["grams"], "0.333333");
        }
    }
    for target in ["0", "1000001", "1.5", "-1"] {
        assert!(
            request(
                &app,
                "GET",
                &format!("{path}/scale?servings={target}"),
                Value::Null
            )
            .await
            .0
            .is_client_error()
        );
    }
    assert_eq!(request(&app, "GET", &path, Value::Null).await.1, created);
    drop(app);
    let app = dir.app();
    assert_eq!(
        request(
            &app,
            "GET",
            &format!("{path}/scale?servings=1"),
            Value::Null
        )
        .await
        .1["nutrition"]["nutrients"][0]["per_serving"],
        "10000.00"
    );
}

#[tokio::test]
async fn scaling_limits_and_missing_nutrition_are_explicit() {
    let dir = Directory::new();
    let app = dir.app();
    for (grams, servings, target) in [("1000000000", 1, 2), ("0.000001", 1_000_000, 1)] {
        let mut input = recipe();
        input["servings"] = json!(servings);
        input["lines"] = json!([{"ingredient_id":"oats","grams":grams}]);
        let created = request(&app, "POST", "/api/recipes", input).await.1;
        let path = format!(
            "/api/recipes/{}/scale?servings={target}",
            created["id"].as_str().unwrap()
        );
        assert_eq!(
            request(&app, "GET", &path, Value::Null).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    let mut input = recipe();
    input["lines"] = json!([{"ingredient_id":"psyllium-husks","grams":"1"}]);
    let created = request(&app, "POST", "/api/recipes", input).await.1;
    let path = format!(
        "/api/recipes/{}/scale?servings=3",
        created["id"].as_str().unwrap()
    );
    let preview = request(&app, "GET", &path, Value::Null).await.1;
    assert_eq!(nutrient(&preview, "energy_kcal")["status"], "unknown");
    assert!(nutrient(&preview, "energy_kcal")["per_serving"].is_null());
}
