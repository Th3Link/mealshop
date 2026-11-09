//! Family planning acceptance: constraints, pinned revisions, consent and transactional previews.
#![allow(clippy::unwrap_used, clippy::too_many_lines)]
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
const OWNER: &str = "planning_test_owner_credential_1234567890";
const READER: &str = "planning_test_reader_credential_123456789";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "mealshop-plans-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
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
async fn request(
    app: &axum::Router,
    method: &str,
    path: &str,
    value: Value,
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
        .oneshot(builder.body(Body::from(value.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), 4_194_304).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap())
}
fn recipe(id: &str) -> Value {
    json!({"title":"Planning fixture","language":"en","author":"Test","servings":2,"preparation":"as_recorded","lines":[{"ingredient_id":id,"grams":"100"}],"steps":[{"instruction":"Prepare and wait","active_seconds":600,"passive_seconds":1800}]})
}
async fn setup(app: &axum::Router) -> (Value, Value) {
    let members = json!([{"id":"one","name":"One","allergies":[],"allergen_ids":["milk"],"excluded_ingredients":[],"disliked_ingredients":[]},{"id":"two","name":"Two","allergies":[],"excluded_ingredients":[],"disliked_ingredients":["oats"]}]);
    assert_eq!(
        request(
            app,
            "PUT",
            "/api/household/profiles",
            json!({"expected_revision":0,"members":members}),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::OK
    );
    let good = request(app, "POST", "/api/recipes", recipe("oats"), None)
        .await
        .1;
    let bad = request(
        app,
        "POST",
        "/api/recipes",
        recipe("milk-1-5-pasteurised"),
        None,
    )
    .await
    .1;
    (good, bad)
}
fn plan(recipe: &Value) -> Value {
    json!({"date":"2026-11-09","meal":"dinner","recipe":{"id":recipe["id"],"revision":recipe["revision"]},"attendees":["one","two"],"calendar":null,"servings":4,"active_seconds":600,"elapsed_seconds":1200,"advance_seconds":1200})
}
async fn preview(app: &axum::Router, p: Value) -> Value {
    let (status, value) = request(
        app,
        "POST",
        "/api/household/mealplan/preview",
        p,
        Some(READER),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    value
}
fn apply(preview: &Value, p: &Value) -> Value {
    json!({"plan":p,"expected_revision":preview["plan_revision"],"profile_revision":preview["profile_revision"],"reviewed":true})
}
#[tokio::test]
async fn preview_apply_rescale_alternative_and_restart_preserve_recipe_and_shopping_history() {
    let dir = Directory::new();
    let app = dir.app();
    let (good, bad) = setup(&app).await;
    let p = plan(&good);
    let result = preview(&app, p.clone()).await;
    assert_eq!(result["feasible"], true);
    assert_eq!(result["timing"]["active_seconds"], 600);
    assert_eq!(result["timing"]["elapsed_seconds"], 2400);
    assert_eq!(result["timing"]["advance_preparation_seconds"], 1200);
    assert_eq!(
        result["shopping_delta"],
        json!([{"ingredient_id":"oats","grams_delta":"200.000000"}])
    );
    assert_eq!(
        request(
            &app,
            "GET",
            "/api/household/mealplan",
            Value::Null,
            Some(READER)
        )
        .await
        .1["meals"],
        json!([])
    );
    let body = apply(&result, &p);
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/apply",
            body.clone(),
            Some(READER)
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let mut unreviewed = body.clone();
    unreviewed["reviewed"] = json!(false);
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/apply",
            unreviewed,
            Some(OWNER)
        )
        .await
        .1["code"],
        "dietary_review_required"
    );
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/apply",
            body.clone(),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/apply",
            body,
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let mut alternative = plan(&bad);
    alternative["attendees"] = json!(["two"]);
    alternative["servings"] = json!(1);
    let result = preview(&app, alternative.clone()).await;
    assert_eq!(
        result["shopping_delta"],
        json!([{"ingredient_id":"milk-1-5-pasteurised","grams_delta":"50.000000"},{"ingredient_id":"oats","grams_delta":"-200.000000"}])
    );
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/apply",
            apply(&result, &alternative),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::OK
    );
    let saved = request(
        &app,
        "GET",
        "/api/household/mealplan",
        Value::Null,
        Some(READER),
    )
    .await
    .1;
    drop(app);
    let app = dir.app();
    assert_eq!(
        request(
            &app,
            "GET",
            "/api/household/mealplan",
            Value::Null,
            Some(READER)
        )
        .await
        .1,
        saved
    );
    let result = preview(&app, p.clone()).await;
    assert_eq!(result["plan"]["recipe"]["revision"], 1);
    let mut edited = recipe("oats");
    edited["lines"][0]["grams"] = json!("200");
    edited["title"] = json!("Renamed current recipe");
    request(
        &app,
        "PUT",
        &format!("/api/recipes/{}", good["id"].as_str().unwrap()),
        json!({"expected_revision":1,"recipe":edited}),
        None,
    )
    .await;
    let listing = request(
        &app,
        "GET",
        "/api/household/mealplan",
        Value::Null,
        Some(READER),
    )
    .await
    .1;
    assert_eq!(listing["recipe_revisions"][0]["title"], "Planning fixture");
    assert_eq!(listing["recipe_revisions"][0]["revision"], 1);
    assert_eq!(
        preview(&app, p).await["shopping_delta"],
        result["shopping_delta"]
    );
}
#[tokio::test]
async fn suggestions_explain_diet_time_unknowns_and_respect_calendar_consent_override() {
    let dir = Directory::new();
    let app = dir.app();
    let (good, bad) = setup(&app).await;
    let candidates = json!([{"id":good["id"],"revision":1},{"id":bad["id"],"revision":1}]);
    let mut p = plan(&good);
    let result = request(
        &app,
        "POST",
        "/api/household/mealplan/suggestions",
        json!({"template":p,"candidates":candidates}),
        Some(READER),
    )
    .await
    .1;
    assert_eq!(result["suggestions"].as_array().unwrap().len(), 1);
    assert_eq!(
        result["rejected"][0]["conflicts"],
        json!(["dietary_exclusion"])
    );
    let blocked = preview(&app, plan(&bad)).await;
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/apply",
            apply(&blocked, &plan(&bad)),
            Some(OWNER)
        )
        .await
        .1["code"],
        "meal_constraints_conflict"
    );
    p["active_seconds"] = json!(0);
    let result = request(
        &app,
        "POST",
        "/api/household/mealplan/suggestions",
        json!({"template":p,"candidates":candidates}),
        Some(READER),
    )
    .await
    .1;
    assert_eq!(result["reason"], "no_feasible_meal");
    assert!(
        result["rejected"][0]["conflicts"]
            .as_array()
            .unwrap()
            .contains(&json!("active_time_exceeded"))
    );
    p = plan(&good);
    p["attendees"] = Value::Null;
    p["calendar"] = json!({"consent":false,"source":"Imported family calendar","event_id":"event-1","date":"2026-11-09","member_ids":["one"]});
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/preview",
            p.clone(),
            Some(READER)
        )
        .await
        .1["code"],
        "calendar_consent_or_source_required"
    );
    p["calendar"]["consent"] = json!(true);
    let result = preview(&app, p.clone()).await;
    assert_eq!(result["attendees"], json!(["one"]));
    assert_eq!(result["attendance_source"], "Imported family calendar");
    p["attendees"] = json!(["two"]);
    assert_eq!(
        preview(&app, p.clone()).await["attendance_overridden"],
        true
    );
    p["date"] = json!("2026-02-30");
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/preview",
            p,
            Some(READER)
        )
        .await
        .1["code"],
        "invalid_meal_plan"
    );
    assert_eq!(
        request(&app, "GET", "/api/household/mealplan", Value::Null, None)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn profile_changes_revoke_preview_assumptions_and_removal_previews_subtract_requirements() {
    let dir = Directory::new();
    let app = dir.app();
    let (good, _) = setup(&app).await;
    let p = plan(&good);
    let before = preview(&app, p.clone()).await;
    let profiles = request(
        &app,
        "GET",
        "/api/household/profiles",
        Value::Null,
        Some(OWNER),
    )
    .await
    .1;
    let mut members = profiles["members"].clone();
    members[0]["excluded_ingredients"] = json!(["oats"]);
    assert_eq!(
        request(
            &app,
            "PUT",
            "/api/household/profiles",
            json!({"expected_revision":1,"members":members}),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/apply",
            apply(&before, &p),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(preview(&app, p.clone()).await["feasible"], false);
    let mut p = p;
    p["attendees"] = json!(["two"]);
    let checked = preview(&app, p.clone()).await;
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/apply",
            apply(&checked, &p),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::OK
    );
    let list = request(
        &app,
        "GET",
        "/api/household/mealplan",
        Value::Null,
        Some(READER),
    )
    .await
    .1;
    assert_eq!(
        list["shopping_requirements"],
        json!([{"ingredient_id":"oats","grams":"200.000000"}])
    );
    let preview = request(
        &app,
        "POST",
        "/api/household/mealplan/remove-preview",
        p.clone(),
        Some(READER),
    )
    .await
    .1;
    assert_eq!(
        preview["shopping_delta"],
        json!([{"ingredient_id":"oats","grams_delta":"-200.000000"}])
    );
    let mut change = apply(&preview, &p);
    change["remove"] = json!(true);
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/apply",
            change,
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &app,
            "GET",
            "/api/household/mealplan",
            Value::Null,
            Some(READER)
        )
        .await
        .1["shopping_requirements"],
        json!([])
    );
}

#[tokio::test]
async fn missing_timings_and_storage_failures_never_silently_apply() {
    let dir = Directory::new();
    let app = dir.app();
    let (good, _) = setup(&app).await;
    let mut draft = recipe("oats");
    draft["steps"] = json!([]);
    let unknown = request(&app, "POST", "/api/recipes", draft, None).await.1;
    let result = preview(&app, plan(&unknown)).await;
    assert!(
        result["conflicts"]
            .as_array()
            .unwrap()
            .contains(&json!("preparation_time_unknown"))
    );
    let mut too_late = plan(&good);
    too_late["advance_seconds"] = json!(0);
    assert!(
        preview(&app, too_late).await["conflicts"]
            .as_array()
            .unwrap()
            .contains(&json!("elapsed_time_exceeded"))
    );
    let p = plan(&good);
    let checked = preview(&app, p.clone()).await;
    std::fs::write(dir.0.join("household/.mealplan.pending"), "interrupted").unwrap();
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/apply",
            apply(&checked, &p),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        request(
            &app,
            "GET",
            "/api/household/mealplan",
            Value::Null,
            Some(READER)
        )
        .await
        .1["meals"],
        json!([])
    );
    drop(app);
    assert!(
        mealshop_server::family_router(
            include_str!("../data/catalogue.json"),
            dir.0.join("recipes"),
            dir.0.join("household"),
            OWNER.to_owned(),
            Some(READER.to_owned())
        )
        .is_err()
    );
    assert!(!dir.0.join("household/.household.lock").exists());
    std::fs::remove_file(dir.0.join("household/.mealplan.pending")).unwrap();
    std::fs::write(dir.0.join("household/mealplan.json"), "corrupt").unwrap();
    assert!(
        mealshop_server::family_router(
            include_str!("../data/catalogue.json"),
            dir.0.join("recipes"),
            dir.0.join("household"),
            OWNER.to_owned(),
            None
        )
        .is_err()
    );
}
