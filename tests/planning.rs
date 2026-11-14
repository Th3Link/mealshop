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

#[tokio::test]
async fn mealprep_allocates_across_days_but_counts_production_once_after_restart() {
    let dir = Directory::new();
    let app = dir.app();
    let (good, _) = setup(&app).await;
    let mut p = plan(&good);
    p["preparation"] = json!({"date":"2026-11-08","time":"17:30"});
    p["allocations"] = json!([{ "date":"2026-11-09","meal":"dinner","servings":2 },{ "date":"2026-11-10","meal":"lunch","servings":2 }]);
    let result = preview(&app, p.clone()).await;
    assert_eq!(result["feasible"], true);
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/apply",
            apply(&result, &p),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::OK
    );
    let listed = request(
        &app,
        "GET",
        "/api/household/mealplan",
        Value::Null,
        Some(READER),
    )
    .await
    .1;
    assert_eq!(
        listed["shopping_requirements"],
        json!([{"ingredient_id":"oats","grams":"200.000000"}])
    );
    let mut moved = p.clone();
    moved["allocations"][1]["date"] = json!("2026-11-11");
    assert_eq!(preview(&app, moved).await["shopping_delta"], json!([]));
    let mut over = p.clone();
    over["allocations"][1]["servings"] = json!(3);
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/preview",
            over,
            Some(READER)
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let mut duplicate = p.clone();
    duplicate["allocations"][1] = duplicate["allocations"][0].clone();
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/preview",
            duplicate,
            Some(READER)
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let mut clash = plan(&good);
    clash["date"] = json!("2026-11-10");
    clash["meal"] = json!("lunch");
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/preview",
            clash,
            Some(READER)
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    drop(app);
    let app = dir.app();
    let reopened = request(
        &app,
        "GET",
        "/api/household/mealplan",
        Value::Null,
        Some(READER),
    )
    .await
    .1;
    assert_eq!(reopened["meals"][0]["allocations"], p["allocations"]);
    assert_eq!(reopened["meals"][0]["preparation"], p["preparation"]);
    assert_eq!(
        reopened["shopping_requirements"],
        listed["shopping_requirements"]
    );
}

fn week_input(good: &Value, bad: &Value) -> Value {
    json!({"start_date":"2026-11-09","meals":["dinner"],"candidates":[
        {"id":good["id"],"revision":good["revision"]},
        {"id":bad["id"],"revision":bad["revision"]}],
        "active_seconds":1800,"elapsed_seconds":3600,"advance_seconds":0,
        "servings_per_person":1,"prefer_quick":true,"prefer_reuse":true})
}
fn week_confirm(proposal: &Value) -> Value {
    json!({"proposal_id":proposal["id"],"reviewed":true,
        "confirmed_slots":proposal["slots"].as_array().unwrap().iter().filter(|s| !s["plan"].is_null())
            .map(|s| json!({"date":s["date"],"meal":s["meal"]})).collect::<Vec<_>>()})
}
#[tokio::test]
async fn week_proposal_is_reviewed_atomic_dietary_safe_and_replayable_after_restart() {
    let dir = Directory::new();
    let app = dir.app();
    let (good, bad) = setup(&app).await;
    let schedule = json!({"expected_revision":0,"attendance":{"recurring":[
        {"member_id":"one","weekdays":[0,1,2,3,4],"meals":["dinner"]}],
        "overrides":[{"date":"2026-11-12","meal":"dinner","member_ids":["two"]},
        {"date":"2026-11-14","meal":"dinner","member_ids":[]}]}});
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/attendance",
            schedule,
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::OK
    );
    let (status, result) = request(
        &app,
        "POST",
        "/api/household/mealplan/week/preview",
        week_input(&good, &bad),
        Some(OWNER),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    let p = &result["proposal"];
    assert_eq!(p["slots"].as_array().unwrap().len(), 7);
    assert_eq!(p["slots"][3]["plan"]["attendees"], json!(["two"]));
    assert_eq!(p["slots"][5]["reason"], "no_attendees");
    assert_eq!(p["slots"][0]["plan"]["recipe"]["id"], good["id"]);
    let initial = request(
        &app,
        "GET",
        "/api/household/mealplan",
        Value::Null,
        Some(READER),
    )
    .await
    .1;
    assert_eq!(initial["meals"], json!([]));
    let body = week_confirm(p);
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/week/apply",
            body.clone(),
            Some(READER)
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let mut partial = body.clone();
    partial["confirmed_slots"].as_array_mut().unwrap().pop();
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/week/apply",
            partial,
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/week/apply",
            body.clone(),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::OK
    );
    drop(app);
    let app = dir.app();
    let (status, replay) = request(
        &app,
        "POST",
        "/api/household/mealplan/week/apply",
        body,
        Some(OWNER),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{replay}");
    assert_eq!(replay["replayed"], true);
    let saved = request(
        &app,
        "GET",
        "/api/household/mealplan",
        Value::Null,
        Some(READER),
    )
    .await
    .1;
    assert_eq!(saved["meals"].as_array().unwrap().len(), 6);
    assert_eq!(saved["revision"], 2);
    assert_eq!(saved["attendance"], initial["attendance"]);
    let invalid = plan(&good); // Thursday excludes person one on every legacy preview/apply path.
    let mut invalid = invalid;
    invalid["date"] = json!("2026-11-12");
    let v = preview(&app, invalid).await;
    assert_eq!(v["feasible"], false);
    assert!(
        v["conflicts"]
            .as_array()
            .unwrap()
            .contains(&json!("attendance_excluded"))
    );
}
#[tokio::test]
async fn week_stale_profiles_and_recipe_heads_cannot_be_confirmed() {
    let dir = Directory::new();
    let app = dir.app();
    let (good, bad) = setup(&app).await;
    let p = request(
        &app,
        "POST",
        "/api/household/mealplan/week/preview",
        week_input(&good, &bad),
        Some(OWNER),
    )
    .await
    .1;
    let revision = json!({"expected_revision":good["revision"],"recipe":recipe("oats")});
    assert_eq!(
        request(
            &app,
            "PUT",
            &format!("/api/recipes/{}", good["id"].as_str().unwrap()),
            revision,
            None
        )
        .await
        .0,
        StatusCode::OK
    );
    let v = request(
        &app,
        "POST",
        "/api/household/mealplan/week/apply",
        week_confirm(&p["proposal"]),
        Some(OWNER),
    )
    .await;
    assert_eq!(v.0, StatusCode::CONFLICT, "{}", v.1);
    assert_eq!(
        request(
            &app,
            "GET",
            "/api/household/mealplan",
            Value::Null,
            Some(OWNER)
        )
        .await
        .1["meals"],
        json!([])
    );
}
#[tokio::test]
async fn impossible_week_keeps_exclusions_and_reports_every_unfilled_slot() {
    let dir = Directory::new();
    let app = dir.app();
    let (good, bad) = setup(&app).await;
    let mut input = week_input(&good, &bad);
    input["candidates"] = json!([{"id":bad["id"],"revision":bad["revision"]}]);
    let (status, v) = request(
        &app,
        "POST",
        "/api/household/mealplan/week/preview",
        input,
        Some(OWNER),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{v}");
    for slot in v["proposal"]["slots"].as_array().unwrap() {
        assert!(slot["plan"].is_null());
        assert_eq!(slot["reason"], "no_feasible_meal");
        assert_eq!(
            slot["evidence"]["rejected_by_constraint"]["dietary_exclusion"],
            1
        );
    }
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/week/apply",
            week_confirm(&v["proposal"]),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn an_expired_proposal_cannot_publish_after_server_restart() {
    let dir = Directory::new();
    let app = dir.app();
    let (good, bad) = setup(&app).await;
    let p = request(
        &app,
        "POST",
        "/api/household/mealplan/week/preview",
        week_input(&good, &bad),
        Some(OWNER),
    )
    .await
    .1;
    drop(app);
    let path = dir.0.join("household/mealplan.json");
    let mut saved: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    saved["proposals"][0]["expires_at"] = json!(1);
    std::fs::write(path, saved.to_string()).unwrap();
    let app = dir.app();
    let v = request(
        &app,
        "POST",
        "/api/household/mealplan/week/apply",
        week_confirm(&p["proposal"]),
        Some(OWNER),
    )
    .await;
    assert_eq!(v.0, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(v.1["code"], "week_expired");
}
#[tokio::test]
async fn real_shopping_preview_preserves_manual_and_purchased_amounts_during_thursday_swap() {
    let dir = Directory::new();
    let app = dir.app();
    let (good, bad) = setup(&app).await;
    let mut p = plan(&good);
    p["date"] = json!("2026-11-12");
    let v = preview(&app, p.clone()).await;
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/apply",
            apply(&v, &p),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::OK
    );
    let sync = json!({"operation":"sync_plan","plan_revision":1,"slots":[{"date":"2026-11-12","meal":"dinner"}]});
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/shopping",
            json!({"expected_revision":0,"command":sync}),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::OK
    );
    for (revision, command) in [
        (
            1,
            json!({"operation":"add_manual","label":"Synthetic extra oats","ingredient_id":"oats","quantity":{"amount":"5.000001","unit":"g"}}),
        ),
        (
            2,
            json!({"operation":"check","key":"ingredient:oats:g","checked":true}),
        ),
    ] {
        assert_eq!(
            request(
                &app,
                "POST",
                "/api/household/shopping",
                json!({"expected_revision":revision,"command":command}),
                Some(OWNER)
            )
            .await
            .0,
            StatusCode::OK
        );
    }
    p["servings"] = json!(6);
    let v = preview(&app, p.clone()).await;
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/mealplan/apply",
            apply(&v, &p),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::OK
    );
    let before = request(
        &app,
        "GET",
        "/api/household/shopping",
        Value::Null,
        Some(OWNER),
    )
    .await
    .1;
    let (status, change) = request(
        &app,
        "POST",
        "/api/household/shopping/plan-preview",
        json!({"expected_revision":3,"plan_revision":2,
        "slots":[{"date":"2026-11-12","meal":"dinner"}]}),
        Some(READER),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{change}");
    assert_eq!(change["changes"][0]["after"]["remaining"], "100.000000");
    assert_eq!(change["changes"][0]["after"]["purchased"], "205.000001");
    assert_eq!(
        request(
            &app,
            "GET",
            "/api/household/shopping",
            Value::Null,
            Some(OWNER)
        )
        .await
        .1,
        before
    );
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/shopping",
            json!({"expected_revision":3,"command":change["command"]}),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::OK
    );
    let after = request(
        &app,
        "GET",
        "/api/household/shopping",
        Value::Null,
        Some(OWNER),
    )
    .await
    .1;
    let manual = after["sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["origin"]["kind"] == "manual")
        .unwrap();
    let prior = before["sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == manual["id"])
        .unwrap();
    assert_eq!(manual, prior);
    let _ = bad;
}

#[tokio::test]
async fn maximum_week_pool_is_bounded_and_keeps_all_twenty_eight_meals_feasible() {
    let dir = Directory::new();
    let app = dir.app();
    let (good, bad) = setup(&app).await;
    let mut candidates = vec![
        json!({"id":good["id"],"revision":1}),
        json!({"id":bad["id"],"revision":1}),
    ];
    for _ in 2..100 {
        let r = request(&app, "POST", "/api/recipes", recipe("oats"), None)
            .await
            .1;
        candidates.push(json!({"id":r["id"],"revision":1}));
    }
    let mut input = week_input(&good, &bad);
    input["candidates"] = json!(candidates);
    input["meals"] = json!(["breakfast", "lunch", "snack", "dinner"]);
    let started = std::time::Instant::now();
    let (status, result) = request(
        &app,
        "POST",
        "/api/household/mealplan/week/preview",
        input,
        Some(OWNER),
    )
    .await;
    eprintln!(
        "P3 synthetic 100-candidate/28-slot in-process debug proposal: {:?}",
        started.elapsed()
    );
    assert_eq!(status, StatusCode::OK, "{}", result["code"]);
    let slots = result["proposal"]["slots"].as_array().unwrap();
    assert_eq!(slots.len(), 28);
    assert!(slots.iter().all(|s| !s["plan"].is_null()));
}
