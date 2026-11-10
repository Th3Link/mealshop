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

#[tokio::test]
async fn shops_fallback_order_and_opt_in_sessions_preserve_provenance() {
    let dir = Directory::new();
    let app = dir.app().unwrap();
    let mut revision = 0;
    macro_rules! act {
        ($body:expr) => {{
            let result = command(&app, revision, $body).await;
            revision += 1;
            result
        }};
    }
    let first =
        act!(json!({"operation":"save_shop","name":"First"}))["shops"]["entries"][0]["id"].clone();
    let second =
        act!(json!({"operation":"save_shop","name":"Fallback"}))["shops"]["entries"][1]["id"]
            .clone();
    act!(json!({"operation":"preference","ingredient_id":"oats","first":first,"second":second}));
    let data = act!(manual("Oats", Some("oats"), "100", "g"));
    let oats = data["items"][0]["key"].clone();
    assert_eq!(data["items"][0]["shop_id"], first);
    assert_eq!(data["items"][0]["fallback_shop_id"], second);
    let sources = data["sources"].clone();
    let data = act!(json!({"operation":"assign_shop","key":oats,"shop_id":second}));
    assert_eq!(data["sources"], sources);
    assert_eq!(data["items"][0]["remaining"], "100.000000");
    assert_eq!(data["items"][0]["shop_id"], second);
    assert_eq!(request(&app,"POST",json!({"expected_revision":revision,"command":{"operation":"start_session","shop_id":first}}),Some(OWNER)).await.0,StatusCode::UNPROCESSABLE_ENTITY);
    act!(json!({"operation":"assign_shop","key":oats,"shop_id":first}));
    let data = act!(manual("Apples", Some("apple-raw"), "2", "piece"));
    let apple = data["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["ingredient_id"] == "apple-raw")
        .unwrap()["key"]
        .clone();
    act!(json!({"operation":"assign_shop","key":apple,"shop_id":first}));
    act!(json!({"operation":"learning","enabled":true}));
    for trip in 0..3 {
        if trip > 0 {
            act!(manual("Oats", Some("oats"), "100", "g"));
            act!(manual("Apples", Some("apple-raw"), "2", "piece"));
        }
        let session =
            act!(json!({"operation":"start_session","shop_id":first}))["shops"]["sessions"]
                .as_array()
                .unwrap()
                .last()
                .unwrap()["id"]
                .clone();
        act!(json!({"operation":"check","key":oats,"checked":true}));
        act!(json!({"operation":"check","key":apple,"checked":true}));
        let data = act!(json!({"operation":"finish_session","id":session,"discard":false}));
        assert_eq!(
            data["items"][0]["order_basis"],
            if trip == 2 { "learned" } else { "uncertain" }
        );
    }
    let data = act!(json!({"operation":"order","shop_id":first,"keys":[apple,oats]}));
    assert_eq!(data["items"][0]["key"], apple);
    assert_eq!(data["items"][0]["order_basis"], "manual");
    let data = act!(json!({"operation":"order","shop_id":first,"keys":[]}));
    assert_eq!(data["items"][0]["key"], oats);
    let data = act!(json!({"operation":"check","key":oats,"checked":false}));
    assert!(
        data["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|i| i["order_basis"] == "uncertain")
    );
    let sources = data["sources"].clone();
    let purchases = std::fs::read(dir.0.join("household/shopping.json")).unwrap();
    let purchases: Value = serde_json::from_slice(&purchases).unwrap();
    let data = act!(json!({"operation":"learning","enabled":false}));
    assert_eq!(data["sources"], sources);
    let data = act!(json!({"operation":"reset_history"}));
    assert_eq!(data["sources"], sources);
    let after: Value =
        serde_json::from_slice(&std::fs::read(dir.0.join("household/shopping.json")).unwrap())
            .unwrap();
    assert_eq!(after["purchases"], purchases["purchases"]);
    assert_eq!(revision, data["revision"].as_u64().unwrap());
    drop(app);
    let app = dir.app().unwrap();
    assert_eq!(
        request(&app, "GET", Value::Null, Some(READER)).await.1,
        data
    );
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn barcode_products_keep_packages_match_consent_and_purchased_snapshots_separate() {
    let dir = Directory::new();
    let app = dir.app().unwrap();
    let product = json!({"barcode":"034000470693","name":"Test packaged oats","brand":"Test brand","label_quantity":"2 x 50 g","quantity":{"amount":"0.100001","unit":"kg"},"ingredient_id":"oats","match_confirmed":false});
    let (status, value) = request(
        &app,
        "POST",
        json!({"expected_revision":0,"command":{"operation":"save_product","product":product}}),
        Some(OWNER),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(value["code"], "shopping_match_required");
    for consent in [false, true] {
        let (status,_) = request_path(&app,"POST","/api/household/shopping/lookup",json!({"expected_revision":0,"barcode":if consent {"http://127.0.0.1"} else {"034000470693"},"consent":consent}),Some(OWNER)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }
    assert!(!dir.0.join("household/shopping.json").exists());
    let mut confirmed = product;
    confirmed["match_confirmed"] = json!(true);
    let data = command(
        &app,
        0,
        json!({"operation":"save_product","product":confirmed}),
    )
    .await;
    assert_eq!(
        data["products"]["00034000470693"]["barcode"],
        "034000470693"
    );
    assert!(data["products"]["00034000470693"]["evidence"].is_null());
    let data = command(
        &app,
        1,
        json!({"operation":"add_product","barcode":"0034000470693","packages":3}),
    )
    .await;
    assert_eq!(data["items"][0]["remaining"], "300.003000");
    assert_eq!(
        data["sources"][0]["origin"]["product"]["quantity"]["amount"],
        "0.100001"
    );
    assert_eq!(data["sources"][0]["origin"]["packages"], 3);
    let key = data["items"][0]["key"].clone();
    command(
        &app,
        2,
        json!({"operation":"check","key":key,"checked":true}),
    )
    .await;
    confirmed["name"] = json!("Revised product");
    confirmed["quantity"]["amount"] = json!("2");
    let data = command(
        &app,
        3,
        json!({"operation":"save_product","product":confirmed}),
    )
    .await;
    assert_eq!(data["items"][0]["purchased"], "300.003000");
    assert_eq!(
        data["items"][0]["contributions"][0]["receipts"][0]["origin"]["product"]["name"],
        "Test packaged oats"
    );
    let data = command(
        &app,
        4,
        json!({"operation":"remove_product","barcode":"034000470693"}),
    )
    .await;
    assert_eq!(data["items"][0]["purchased"], "300.003000");
    assert!(data["products"].as_object().unwrap().is_empty());
    let original = std::fs::read(dir.0.join("household/shopping.json")).unwrap();
    let mut forged = confirmed;
    forged["candidate_id"] = json!("lookup-999");
    let (status, _) = request(
        &app,
        "POST",
        json!({"expected_revision":5,"command":{"operation":"save_product","product":forged}}),
        Some(OWNER),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        std::fs::read(dir.0.join("household/shopping.json")).unwrap(),
        original
    );
    assert_eq!(
        request_path(
            &app,
            "POST",
            "/api/household/shopping/lookup",
            json!({"expected_revision":5,"barcode":"034000470693","consent":true}),
            Some(READER)
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    drop(app);
    let app = dir.app().unwrap();
    assert_eq!(
        request(&app, "GET", Value::Null, Some(READER)).await.1,
        data
    );
}

#[tokio::test]
async fn homemade_ingredients_purchase_only_inputs_and_keep_dietary_and_cooking_guards() {
    let dir = Directory::new();
    let app = dir.app().unwrap();
    let recipe_body = json!({"title":"Homemade fixture","language":"en","servings":2,"preparation":"as_recorded","parts":[{"id":"prep","name":"Input preparation"}],"lines":[{"id":"output","ingredient_id":"oats","grams":"60.000001","prepared_part":"prep"},{"id":"input","ingredient_id":"chia","grams":"100.000001","part_id":"prep"}],"steps":[{"instruction":"Use the prepared component","active_seconds":1,"passive_seconds":0},{"part_id":"prep","instruction":"Prepare the fictional input","active_seconds":1,"passive_seconds":0}]});
    let (status, recipe) =
        request_path(&app, "POST", "/api/recipes", recipe_body, Some(OWNER)).await;
    assert_eq!(status, StatusCode::CREATED, "{recipe}");
    let bought = command(
        &app,
        0,
        json!({"operation":"add_recipe","recipe":{"recipe_id":recipe["id"],"revision":1,"servings":4}}),
    )
    .await;
    assert_eq!(bought["items"].as_array().unwrap().len(), 1);
    assert_eq!(bought["items"][0]["ingredient_id"], "chia");
    assert_eq!(bought["items"][0]["remaining"], "200.000002");
    let template_path = format!(
        "/api/household/cooking/template?recipe_id={}&revision=1&servings=4&timezone=Europe%2FBerlin",
        recipe["id"].as_str().unwrap()
    );
    let (status, template) =
        request_path(&app, "GET", &template_path, Value::Null, Some(OWNER)).await;
    assert_eq!(status, StatusCode::OK, "{template}");
    assert_eq!(
        template["draft"]["stages"][0]["dependencies"],
        json!(["s2"])
    );
    let plan = json!({"date":"2026-11-11","meal":"lunch","recipe":{"id":recipe["id"],"revision":1},"attendees":["one"],"calendar":null,"servings":4,"active_seconds":86400,"elapsed_seconds":86400,"advance_seconds":2_592_000});
    for (revision, excluded) in [(0, "oats"), (1, "chia")] {
        let profile = json!({"id":"one","name":"Test diner","allergies":[],"intolerances":[],"allergen_ids":[],"excluded_ingredients":[excluded],"disliked_ingredients":[]});
        assert_eq!(
            request_path(
                &app,
                "PUT",
                "/api/household/profiles",
                json!({"expected_revision":revision,"members":[profile]}),
                Some(OWNER)
            )
            .await
            .0,
            StatusCode::OK
        );
        let (_, preview) = request_path(
            &app,
            "POST",
            "/api/household/mealplan/preview",
            plan.clone(),
            Some(OWNER),
        )
        .await;
        assert_eq!(preview["feasible"], false, "{preview}");
        assert_eq!(request_path(&app,"POST","/api/household/mealplan/apply",json!({"plan":plan,"expected_revision":0,"profile_revision":revision+1,"reviewed":true}),Some(OWNER)).await.0,StatusCode::UNPROCESSABLE_ENTITY);
    }
    let profile = json!({"id":"one","name":"Test diner","allergies":[],"intolerances":[],"allergen_ids":[],"excluded_ingredients":[],"disliked_ingredients":[]});
    assert_eq!(
        request_path(
            &app,
            "PUT",
            "/api/household/profiles",
            json!({"expected_revision":2,"members":[profile]}),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::OK
    );
    let (status, applied) = request_path(
        &app,
        "POST",
        "/api/household/mealplan/apply",
        json!({"plan":plan,"expected_revision":0,"profile_revision":3,"reviewed":true}),
        Some(OWNER),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{applied}");
    let synced = command(&app,1,json!({"operation":"sync_plan","plan_revision":1,"slots":[{"date":"2026-11-11","meal":"lunch"}]})).await;
    assert_eq!(synced["items"].as_array().unwrap().len(), 1);
    assert_eq!(synced["items"][0]["ingredient_id"], "chia");
    assert_eq!(synced["items"][0]["remaining"], "400.000004");
    let again = command(&app,2,json!({"operation":"sync_plan","plan_revision":1,"slots":[{"date":"2026-11-11","meal":"lunch"}]})).await;
    assert_eq!(again["items"], synced["items"]);
    drop(app);
    let reopened = dir.app().unwrap();
    let (_, list) = request(&reopened, "GET", Value::Null, Some(OWNER)).await;
    assert_eq!(list["items"], synced["items"]);
}
