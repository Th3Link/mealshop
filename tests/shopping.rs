//! Protected shopping-list quantities, provenance, revision and recovery contracts.
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
const OWNER: &str = "shopping_test_owner_credential_1234567890";
const READER: &str = "shopping_test_reader_credential_123456789";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "mealshop-shopping-{}-{}",
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
async fn request(
    app: &axum::Router,
    method: &str,
    body: Value,
    token: Option<&str>,
) -> (StatusCode, Value) {
    request_path(app, method, "/api/household/shopping", body, token).await
}
async fn request_path(
    app: &axum::Router,
    method: &str,
    path: &str,
    body: Value,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json");
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 4_194_304).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}
async fn command(app: &axum::Router, revision: u64, body: Value) -> Value {
    let (status, value) = request(
        app,
        "POST",
        json!({"expected_revision":revision,"command":body}),
        Some(OWNER),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    value
}
fn manual(label: &str, ingredient: Option<&str>, amount: &str, unit: &str) -> Value {
    json!({"operation":"add_manual","label":label,"ingredient_id":ingredient,"quantity":{"amount":amount,"unit":unit}})
}
#[tokio::test]
async fn manual_shopping_keeps_dimensions_purchases_undo_and_restart() {
    let dir = Directory::new();
    let app = dir.app().unwrap();
    assert_eq!(
        request(&app, "GET", Value::Null, None).await.0,
        StatusCode::UNAUTHORIZED
    );
    let a = command(&app, 0, manual("Oats", Some("oats"), "0.100001", "kg")).await;
    let key = a["items"][0]["key"].as_str().unwrap().to_owned();
    assert_eq!(a["items"][0]["remaining"], "100.001000");
    let a = command(&app, 1, manual("Drink", Some("oats"), "0.5", "l")).await;
    assert_eq!(
        a["items"].as_array().unwrap().len(),
        2,
        "Volume must not be converted to mass"
    );
    let a = command(
        &app,
        2,
        json!({"operation":"check","key":key,"checked":true}),
    )
    .await;
    let row = a["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["key"] == key)
        .unwrap();
    assert_eq!(row["remaining"], "0.000000");
    assert_eq!(row["purchased"], "100.001000");
    let source = a["sources"][0]["id"].clone();
    let a = command(&app, 3, json!({"operation":"remove_source","id":source})).await;
    let row = a["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["key"] == key)
        .unwrap();
    assert_eq!(row["purchased"], "100.001000");
    assert_eq!(
        row["contributions"][0]["receipts"][0]["origin"]["quantity"]["amount"],
        "0.100001"
    );
    drop(app);
    let app = dir.app().unwrap();
    assert_eq!(request(&app, "GET", Value::Null, Some(READER)).await.1, a);
    let a = command(
        &app,
        4,
        json!({"operation":"check","key":key,"checked":false}),
    )
    .await;
    assert_eq!(
        a["items"].as_array().unwrap().len(),
        1,
        "Undo must not resurrect a removed requirement"
    );
    let a = command(&app, 5, manual("Kitchen roll", None, "2", "piece")).await;
    assert!(
        a["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["name"] == "Kitchen roll" && i["remaining"] == "2.000000")
    );
}
#[tokio::test]
async fn shopping_rejects_stale_read_only_invalid_and_corrupt_writes() {
    let dir = Directory::new();
    let app = dir.app().unwrap();
    let body = json!({"expected_revision":0,"command":manual("Oats",Some("oats"),"1","g")});
    assert_eq!(
        request(&app, "POST", body.clone(), Some(READER)).await.0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(&app, "POST", body.clone(), Some(OWNER)).await.0,
        StatusCode::OK
    );
    assert_eq!(
        request(&app, "POST", body, Some(OWNER)).await.0,
        StatusCode::CONFLICT
    );
    let path = dir.0.join("household/shopping.json");
    let original = std::fs::read(&path).unwrap();
    for command in [
        manual("Zero", None, "0", "g"),
        manual("Unknown", Some("no-such-food"), "1", "g"),
        json!({"operation":"add_manual","label":"Numeric","quantity":{"amount":1,"unit":"g"}}),
        json!({"operation":"check","key":"missing","checked":true}),
    ] {
        assert_eq!(
            request(
                &app,
                "POST",
                json!({"expected_revision":1,"command":command}),
                Some(OWNER)
            )
            .await
            .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }
    drop(app);
    std::fs::write(&path, b"corrupt").unwrap();
    assert!(dir.app().is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"corrupt");
    std::fs::write(&path, original).unwrap();
    std::fs::write(dir.0.join("household/.shopping.pending"), b"interrupted").unwrap();
    assert!(dir.app().is_err());
}

fn recipe_body(grams: &str) -> Value {
    json!({"title":"Shopping recipe","language":"en","author":"","servings":2,"preparation":"as_recorded","lines":[{"ingredient_id":"oats","grams":grams}],"steps":[{"instruction":"Prepare","active_seconds":60,"passive_seconds":0}]})
}
#[tokio::test]
async fn recipes_rescale_only_outstanding_requirements_and_keep_original_purchased_provenance() {
    let dir = Directory::new();
    let app = dir.app().unwrap();
    let (status, recipe) =
        request_path(&app, "POST", "/api/recipes", recipe_body("100"), None).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = recipe["id"].as_str().unwrap();
    let first = command(
        &app,
        0,
        json!({"operation":"add_recipe","recipe":{"recipe_id":id,"revision":1,"servings":4}}),
    )
    .await;
    let source = first["sources"][0]["id"].clone();
    let key = first["items"][0]["key"].clone();
    assert_eq!(first["items"][0]["remaining"], "200.000000");
    command(&app, 1, manual("Extra oats", Some("oats"), "50", "g")).await;
    let bought = command(
        &app,
        2,
        json!({"operation":"check","key":key,"checked":true}),
    )
    .await;
    assert_eq!(bought["items"][0]["purchased"], "250.000000");
    let path = format!("/api/recipes/{id}");
    assert_eq!(
        request_path(
            &app,
            "PUT",
            &path,
            json!({"expected_revision":1,"recipe":recipe_body("300")}),
            None
        )
        .await
        .0,
        StatusCode::OK
    );
    let scaled=command(&app,3,json!({"operation":"add_recipe","source_id":source,"recipe":{"recipe_id":id,"revision":2,"servings":4}})).await;
    assert_eq!(scaled["items"][0]["remaining"], "400.000000");
    assert_eq!(scaled["items"][0]["purchased"], "250.000000");
    assert_eq!(
        scaled["items"][0]["contributions"][0]["receipts"][0]["origin"]["revision"],
        1
    );
    assert_eq!(
        scaled["items"][0]["contributions"][0]["origin"]["revision"],
        2
    );
    let removed = command(&app, 4, json!({"operation":"remove_source","id":source})).await;
    assert_eq!(removed["items"][0]["remaining"], "0.000000");
    assert_eq!(removed["items"][0]["purchased"], "250.000000");
    let corrected = command(
        &app,
        5,
        json!({"operation":"check","key":key,"checked":false}),
    )
    .await;
    assert_eq!(corrected["items"][0]["remaining"], "50.000000");
    drop(app);
    let app = dir.app().unwrap();
    assert_eq!(
        request(&app, "GET", Value::Null, Some(OWNER)).await.1,
        corrected
    );
}

#[tokio::test]
async fn selected_plan_sync_is_revision_checked_idempotent_and_keeps_manual_and_purchased_sources()
{
    let dir = Directory::new();
    let app = dir.app().unwrap();
    let recipe = request_path(&app, "POST", "/api/recipes", recipe_body("100"), None)
        .await
        .1;
    let profile = json!({"id":"one","name":"One","allergies":[],"intolerances":[],"allergen_ids":[],"excluded_ingredients":[],"disliked_ingredients":[]});
    assert_eq!(
        request_path(
            &app,
            "PUT",
            "/api/household/profiles",
            json!({"expected_revision":0,"members":[profile]}),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::OK
    );
    let mut plan = json!({"date":"2026-11-09","meal":"dinner","recipe":{"id":recipe["id"],"revision":1},"attendees":["one"],"calendar":null,"servings":2,"active_seconds":60,"elapsed_seconds":60,"advance_seconds":0});
    assert_eq!(
        request_path(
            &app,
            "POST",
            "/api/household/mealplan/apply",
            json!({"plan":plan,"expected_revision":0,"profile_revision":1,"reviewed":true}),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::OK
    );
    command(&app, 0, manual("Kitchen roll", None, "2", "piece")).await;
    let sync = json!({"operation":"sync_plan","plan_revision":1,"slots":[{"date":"2026-11-09","meal":"dinner"}]});
    let one = command(&app, 1, sync.clone()).await;
    let two = command(&app, 2, sync).await;
    assert_eq!(one["items"], two["items"]);
    assert_eq!(one["sources"], two["sources"]);
    let key = two["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["ingredient_id"] == "oats")
        .unwrap()["key"]
        .clone();
    command(
        &app,
        3,
        json!({"operation":"check","key":key,"checked":true}),
    )
    .await;
    plan["servings"] = json!(4);
    assert_eq!(
        request_path(
            &app,
            "POST",
            "/api/household/mealplan/apply",
            json!({"plan":plan,"expected_revision":1,"profile_revision":1,"reviewed":true}),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::OK
    );
    let (status,_)=request(&app,"POST",json!({"expected_revision":4,"command":{"operation":"sync_plan","plan_revision":1,"slots":[]}}),Some(OWNER)).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let updated=command(&app,4,json!({"operation":"sync_plan","plan_revision":2,"slots":[{"date":"2026-11-09","meal":"dinner"}]})).await;
    let row = updated["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["ingredient_id"] == "oats")
        .unwrap();
    assert_eq!(row["remaining"], "100.000000");
    assert_eq!(row["purchased"], "100.000000");
    let removed = command(
        &app,
        5,
        json!({"operation":"sync_plan","plan_revision":2,"slots":[]}),
    )
    .await;
    assert!(
        removed["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["name"] == "Kitchen roll" && i["remaining"] == "2.000000")
    );
    let row = removed["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["ingredient_id"] == "oats")
        .unwrap();
    assert_eq!(row["remaining"], "0.000000");
    assert_eq!(row["purchased"], "100.000000");
}
