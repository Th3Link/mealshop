//! Completed cooks, household feedback privacy and conservative learned suggestions.
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
const OWNER: &str = "feedback_owner_test_credential_1234567890";
const READER: &str = "feedback_reader_test_credential_123456789";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "mealshop-feedback-api-{}-{}",
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
    let mut r = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json");
    if let Some(t) = token {
        r = r.header("authorization", format!("Bearer {t}"));
    }
    let r = app
        .clone()
        .oneshot(r.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = r.status();
    let bytes = to_bytes(r.into_body(), 4_194_304).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}
async fn fixture(app: &axum::Router) -> String {
    assert_eq!(call(app,Some(OWNER),"PUT","/api/household/profiles",json!({"expected_revision":0,"members":[{"id":"person-a","name":"Synthetic attendee","allergies":[],"intolerances":[],"excluded_ingredients":[],"disliked_ingredients":[]}]})).await.0,StatusCode::OK);
    let (status, r) = call(
        app,
        None,
        "POST",
        "/api/recipes",
        recipe("Synthetic history\nrecipe", "oats"),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    r["id"].as_str().unwrap().to_owned()
}
fn recipe(title: &str, ingredient: &str) -> Value {
    json!({"title":title,"language":"en","servings":2,"preparation":"as_recorded","lines":[{"ingredient_id":ingredient,"grams":"100.500000"}],"steps":[{"instruction":"Prepare synthetic fixture","active_seconds":60,"passive_seconds":0}]})
}
fn entry(id: &str) -> Value {
    json!({"expected_revision":0,"expected_profile_revision":1,"confirmed_completed":true,"source":{"kind":"manual","recipe_id":id,"revision":1,"servings":2},"input":{"date":"2026-11-01","timezone":"Europe/Berlin","attendees":["person-a"],"active_seconds":null,"household_rating":5,"ratings":{"person-a":4}}})
}
#[tokio::test]
async fn manual_completion_is_explicit_private_and_stale_safe() {
    let dir = Directory::new();
    let app = dir.app();
    let id = fixture(&app).await;
    let body = entry(&id);
    assert_eq!(
        call(&app, None, "GET", "/api/household/history", Value::Null)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(
            &app,
            Some(READER),
            "POST",
            "/api/household/history",
            body.clone()
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let mut unconfirmed = body.clone();
    unconfirmed["confirmed_completed"] = json!(false);
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/history",
            unconfirmed
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let (status, saved) = call(
        &app,
        Some(OWNER),
        "POST",
        "/api/household/history",
        body.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{saved}");
    let path = format!("/api/household/history/{}", saved["id"].as_str().unwrap());
    let value = call(&app, Some(READER), "GET", &path, Value::Null).await.1;
    assert_eq!(value["entry"]["input"]["active_seconds"], Value::Null);
    assert_eq!(value["entry"]["recipe"]["revision"], 1);
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/history",
            body.clone()
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
}
#[tokio::test]
async fn history_keeps_recipe_revision_and_supports_explicit_corrections_after_restart() {
    let dir = Directory::new();
    let app = dir.app();
    let id = fixture(&app).await;
    let body = entry(&id);
    let (status, saved) = call(
        &app,
        Some(OWNER),
        "POST",
        "/api/household/history",
        body.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let path = format!("/api/household/history/{}", saved["id"].as_str().unwrap());
    let current = recipe("Revised synthetic history recipe", "oats");
    assert_eq!(
        call(
            &app,
            None,
            "PUT",
            &format!("/api/recipes/{id}"),
            json!({"expected_revision":1,"recipe":current})
        )
        .await
        .0,
        StatusCode::OK
    );
    drop(app);
    let app = dir.app();
    let pinned = call(&app, Some(OWNER), "GET", &path, Value::Null).await.1;
    assert_eq!(
        pinned["entry"]["recipe"]["title"],
        "Synthetic history\nrecipe"
    );
    let mut edit = body;
    edit["expected_revision"] = json!(1);
    edit["input"]["active_seconds"] = json!(125);
    assert_eq!(
        call(&app, Some(OWNER), "PUT", &path, edit).await.0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "DELETE",
            &path,
            json!({"expected_revision":2,"confirmed":false})
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "DELETE",
            &path,
            json!({"expected_revision":2,"confirmed":true})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, Some(OWNER), "GET", &path, Value::Null).await.0,
        StatusCode::NOT_FOUND
    );
}
async fn session(app: &axum::Router, recipe_id: &str) -> String {
    let value=call(app,Some(OWNER),"GET",&format!("/api/household/cooking/template?recipe_id={recipe_id}&revision=1&servings=3&timezone=Europe%2FBerlin"),Value::Null).await.1;
    let preview = call(
        app,
        Some(OWNER),
        "POST",
        "/api/household/cooking/preview",
        json!({"expected_revision":0,"proposal":{"kind":"create","draft":value["draft"]}}),
    )
    .await
    .1;
    let (status,saved)=call(app,Some(OWNER),"POST","/api/household/cooking",json!({"expected_revision":0,"command":{"operation":"apply","preview_id":preview["preview_id"],"reviewed":true}})).await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    saved["run_id"].as_str().unwrap().to_owned()
}
#[tokio::test]
async fn only_completed_sessions_can_be_recorded_once_and_survive_session_removal() {
    let dir = Directory::new();
    let app = dir.app();
    let recipe_id = fixture(&app).await;
    let run = session(&app, &recipe_id).await;
    let mut body = entry(&recipe_id);
    body["source"] = json!({"kind":"session","run_id":run});
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "GET",
            "/api/household/history",
            Value::Null
        )
        .await
        .1["total"],
        0
    );
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/history",
            body.clone()
        )
        .await
        .1["code"],
        "history_session_incomplete"
    );
    let cooking = call(
        &app,
        Some(OWNER),
        "GET",
        &format!("/api/household/cooking/{run}"),
        Value::Null,
    )
    .await
    .1;
    let step = cooking["run"]["steps"][0]["stage"]["id"].clone();
    assert_eq!(call(&app,Some(OWNER),"POST","/api/household/cooking",json!({"expected_revision":1,"command":{"operation":"timer","run_id":run,"step_id":step,"action":"complete"}})).await.0,StatusCode::OK);
    let (status, saved) = call(
        &app,
        Some(OWNER),
        "POST",
        "/api/household/history",
        body.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{saved}");
    let path = format!("/api/household/history/{}", saved["id"].as_str().unwrap());
    let detail = call(&app, Some(OWNER), "GET", &path, Value::Null).await.1;
    assert_eq!(detail["entry"]["servings"], 3);
    body["expected_revision"] = json!(1);
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/history",
            body.clone()
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/cooking",
            json!({"expected_revision":2,"command":{"operation":"remove","run_id":run}})
        )
        .await
        .0,
        StatusCode::OK
    );
    body["input"]["household_rating"] = json!(3);
    assert_eq!(
        call(&app, Some(OWNER), "PUT", &path, body.clone()).await.0,
        StatusCode::OK
    );
    body["expected_revision"] = json!(2);
    body["source"] = json!({"kind":"manual","recipe_id":recipe_id,"revision":1,"servings":2});
    assert_eq!(
        call(&app, Some(OWNER), "PUT", &path, body).await.1["code"],
        "history_source_changed"
    );
    drop(app);
    let app = dir.app();
    assert_eq!(
        call(&app, Some(OWNER), "GET", &path, Value::Null).await.1["entry"]["input"]["household_rating"],
        3
    );
}
#[tokio::test]
async fn cancelled_cooking_sessions_never_count_as_completed_history() {
    let dir = Directory::new();
    let app = dir.app();
    let recipe_id = fixture(&app).await;
    let run = session(&app, &recipe_id).await;
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/cooking",
            json!({"expected_revision":1,"command":{"operation":"cancel","run_id":run}})
        )
        .await
        .0,
        StatusCode::OK
    );
    let mut body = entry(&recipe_id);
    body["source"] = json!({"kind":"session","run_id":run});
    assert_eq!(
        call(&app, Some(OWNER), "POST", "/api/household/history", body)
            .await
            .1["code"],
        "history_session_incomplete"
    );
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "GET",
            "/api/household/history",
            Value::Null
        )
        .await
        .1["total"],
        0
    );
}
#[tokio::test]
async fn learning_is_opt_in_and_reset_does_not_replay_retained_observations() {
    let dir = Directory::new();
    let app = dir.app();
    let id = fixture(&app).await;
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/history",
            entry(&id)
        )
        .await
        .0,
        StatusCode::CREATED
    );
    let config = json!({"expected_revision":1,"expected_profile_revision":1,"policy":{"enabled":true,"member_ids":["person-a"]}});
    assert_eq!(
        call(
            &app,
            Some(READER),
            "PUT",
            "/api/household/learning",
            config.clone()
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let policy = call(&app, Some(OWNER), "PUT", "/api/household/learning", config)
        .await
        .1;
    assert_eq!(policy["learned_cooks"], 0);
    let mut record = entry(&id);
    record["expected_revision"] = json!(2);
    assert_eq!(
        call(&app, Some(OWNER), "POST", "/api/household/history", record)
            .await
            .0,
        StatusCode::CREATED
    );
    assert_eq!(
        call(
            &app,
            Some(READER),
            "GET",
            "/api/household/learning",
            Value::Null
        )
        .await
        .1["learned_cooks"],
        1
    );
    let reset = json!({"expected_revision":3,"expected_profile_revision":1,"policy":{"enabled":false,"member_ids":[]}});
    let state = call(&app, Some(OWNER), "PUT", "/api/household/learning", reset)
        .await
        .1;
    assert_eq!(state["learned_cooks"], 0);
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "GET",
            "/api/household/history",
            Value::Null
        )
        .await
        .1["total"],
        2
    );
    drop(app);
    let app = dir.app();
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "GET",
            "/api/household/learning",
            Value::Null
        )
        .await
        .1["policy"]["enabled"],
        false
    );
    assert_eq!(call(&app,Some(OWNER),"PUT","/api/household/learning",json!({"expected_revision":4,"expected_profile_revision":1,"policy":{"enabled":true,"member_ids":[]}})).await.1["learned_cooks"],0);
}
#[tokio::test]
async fn unknown_people_and_future_completion_are_rejected_before_bulk_deletion() {
    let dir = Directory::new();
    let app = dir.app();
    let id = fixture(&app).await;
    let mut invalid = entry(&id);
    invalid["expected_revision"] = json!(0);
    invalid["input"]["date"] = json!("2099-01-01");
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/history",
            invalid.clone()
        )
        .await
        .1["code"],
        "history_future_date"
    );
    invalid["input"]["date"] = json!("2026-11-01");
    invalid["input"]["ratings"] = json!({"nonattendee":5});
    assert_eq!(
        call(&app, Some(OWNER), "POST", "/api/household/history", invalid)
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/history",
            entry(&id)
        )
        .await
        .0,
        StatusCode::CREATED
    );
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "DELETE",
            "/api/household/history",
            json!({"expected_revision":1,"confirmed":true})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "GET",
            "/api/household/history",
            Value::Null
        )
        .await
        .1["total"],
        0
    );
}
fn suggestion_request(candidates: &[Value]) -> Value {
    json!({"template":{"date":"2026-11-02","meal":"lunch","recipe":candidates[0],"attendees":["person-a"],"calendar":null,"servings":2,"active_seconds":600,"elapsed_seconds":600,"advance_seconds":0},"candidates":candidates})
}
#[tokio::test]
async fn strongest_positive_feedback_cannot_override_current_hard_exclusions() {
    let dir = Directory::new();
    let app = dir.app();
    let oats = fixture(&app).await;
    let (status, water) = call(
        &app,
        None,
        "POST",
        "/api/recipes",
        recipe("Synthetic water recipe", "drinking-water"),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(call(&app,Some(OWNER),"PUT","/api/household/profiles",json!({"expected_revision":1,"members":[{"id":"person-a","name":"Synthetic attendee","allergies":[],"intolerances":[],"excluded_ingredients":["oats"],"disliked_ingredients":[]}]})).await.0,StatusCode::OK);
    assert_eq!(call(&app,Some(OWNER),"PUT","/api/household/learning",json!({"expected_revision":0,"expected_profile_revision":2,"policy":{"enabled":true,"member_ids":[]}})).await.0,StatusCode::OK);
    let mut positive = entry(&oats);
    positive["expected_revision"] = json!(1);
    positive["expected_profile_revision"] = json!(2);
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/history",
            positive
        )
        .await
        .0,
        StatusCode::CREATED
    );
    let mut negative = entry(water["id"].as_str().unwrap());
    negative["expected_revision"] = json!(2);
    negative["expected_profile_revision"] = json!(2);
    negative["input"]["household_rating"] = json!(1);
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/history",
            negative
        )
        .await
        .0,
        StatusCode::CREATED
    );
    let request = suggestion_request(&[
        json!({"id":oats,"revision":1}),
        json!({"id":water["id"],"revision":1}),
    ]);
    let (status, ranked) = call(
        &app,
        Some(READER),
        "POST",
        "/api/household/mealplan/suggestions",
        request.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{ranked}");
    assert_eq!(ranked["total"], 1);
    assert_eq!(ranked["model_version"], "history-v1");
    assert_eq!(
        ranked["suggestions"][0]["plan"]["recipe"]["id"],
        water["id"]
    );
    assert_eq!(ranked["rejected"][0]["conflicts"][0], "dietary_exclusion");
    assert_eq!(
        ranked["suggestions"][0]["recommendation"]["evidence"]["cooks"],
        1
    );
    assert_eq!(call(&app,Some(OWNER),"PUT","/api/household/learning",json!({"expected_revision":3,"expected_profile_revision":2,"policy":{"enabled":false,"member_ids":[]}})).await.0,StatusCode::OK);
    let reset = call(
        &app,
        Some(OWNER),
        "POST",
        "/api/household/mealplan/suggestions",
        request,
    )
    .await
    .1;
    assert_eq!(reset["model_version"], "rules-v1");
    assert_eq!(reset["suggestions"][0]["plan"]["recipe"]["id"], water["id"]);
    assert_eq!(
        reset["suggestions"][0]["recommendation"]["evidence"]["cooks"],
        0
    );
}
#[tokio::test]
async fn whole_pool_ranking_paginates_without_hiding_new_recipes_or_accepting_duplicates() {
    let dir = Directory::new();
    let app = dir.app();
    let first = fixture(&app).await;
    let mut candidates = vec![json!({"id":first,"revision":1})];
    for n in 2..=25 {
        let (status, saved) = call(
            &app,
            None,
            "POST",
            "/api/recipes",
            recipe(&format!("Synthetic candidate {n}"), "drinking-water"),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        candidates.push(json!({"id":saved["id"],"revision":1}));
    }
    let request = suggestion_request(&candidates);
    let (status, page1) = call(
        &app,
        Some(OWNER),
        "POST",
        "/api/household/mealplan/suggestions",
        request.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{page1}");
    assert_eq!(page1["total"], 25);
    assert_eq!(page1["suggestions"].as_array().unwrap().len(), 20);
    assert_eq!(
        page1["suggestions"][2]["plan"]["recipe"]["id"],
        candidates[24]["id"]
    );
    assert_eq!(page1["suggestions"][2]["recommendation"]["discovery"], true);
    let mut next = request.clone();
    next["page"] = json!(2);
    let page2 = call(
        &app,
        Some(OWNER),
        "POST",
        "/api/household/mealplan/suggestions",
        next.clone(),
    )
    .await
    .1;
    assert_eq!(page2["suggestions"].as_array().unwrap().len(), 5);
    let unique = page1["suggestions"]
        .as_array()
        .unwrap()
        .iter()
        .chain(page2["suggestions"].as_array().unwrap())
        .map(|v| v["plan"]["recipe"]["id"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unique.len(), 25);
    next["page"] = json!(999);
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/mealplan/suggestions",
            next
        )
        .await
        .1["page"],
        2
    );
    let mut duplicate = request;
    duplicate["candidates"] = json!([candidates[0], candidates[0]]);
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/mealplan/suggestions",
            duplicate
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}
