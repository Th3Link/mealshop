//! Protected cooking instances, reviewed schedules, exact ingredients and recovery.
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
const OWNER: &str = "cooking_test_owner_credential_1234567890";
const READER: &str = "cooking_test_reader_credential_123456789";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "mealshop-cooking-{}-{}",
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
    let body = to_bytes(response.into_body(), 4_194_304).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap())
}
async fn create_recipe(app: &axum::Router) -> String {
    let (status,recipe)=request(app,"POST","/api/recipes",json!({"title":"Cooking acceptance recipe","language":"en","servings":2,"preparation":"as_recorded","lines":[{"ingredient_id":"oats","grams":"100.000001"}],"steps":[{"instruction":"Mix ingredients","active_seconds":30,"passive_seconds":60},{"instruction":"Prepare another component","active_seconds":20,"passive_seconds":40}]}),None).await;
    assert_eq!(status, StatusCode::CREATED, "{recipe}");
    recipe["id"].as_str().unwrap().to_owned()
}
async fn template(app: &axum::Router, recipe: &str) -> Value {
    let (status,value)=request(app,"GET",&format!("/api/household/cooking/template?recipe_id={recipe}&revision=1&servings=3&timezone=Europe%2FBerlin"),Value::Null,Some(OWNER)).await;
    assert_eq!(status, StatusCode::OK, "{value}");
    value["draft"].clone()
}
async fn preview(app: &axum::Router, revision: u64, proposal: Value) -> Value {
    let (status, value) = request(
        app,
        "POST",
        "/api/household/cooking/preview",
        json!({"expected_revision":revision,"proposal":proposal}),
        Some(OWNER),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    value
}
async fn command(app: &axum::Router, revision: u64, command: Value) -> Value {
    let (status, value) = request(
        app,
        "POST",
        "/api/household/cooking",
        json!({"expected_revision":revision,"command":command}),
        Some(OWNER),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    value
}
async fn detail(app: &axum::Router, id: &str) -> Value {
    let (status, value) = request(
        app,
        "GET",
        &format!("/api/household/cooking/{id}"),
        Value::Null,
        Some(READER),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    value
}
#[tokio::test]
async fn concurrent_named_timers_preserve_pause_deadlines_sources_and_restart() {
    let dir = Directory::new();
    let app = dir.app().unwrap();
    let recipe = create_recipe(&app).await;
    let mut draft = template(&app, &recipe).await;
    draft["stages"][1]["dependencies"] = json!([]);
    let reviewed = preview(&app, 0, json!({"kind":"create","draft":draft})).await;
    assert!(
        !dir.0.join("household/cooking.json").exists(),
        "Preview must not create a durable instance"
    );
    let result = command(
        &app,
        0,
        json!({"operation":"apply","preview_id":reviewed["preview_id"],"reviewed":true}),
    )
    .await;
    let id = result["run_id"].as_str().unwrap().to_owned();
    let initial = detail(&app, &id).await;
    assert_eq!(initial["ingredients"][0]["grams"], "150.000002");
    assert_eq!(initial["run"]["recipe"]["revision"], 1);
    assert_eq!(
        initial["run"]["steps"][0]["stage"]["instruction"],
        "Mix ingredients"
    );
    let first = initial["run"]["steps"][0]["stage"]["id"].clone();
    let second = initial["run"]["steps"][1]["stage"]["id"].clone();
    command(
        &app,
        1,
        json!({"operation":"timer","run_id":id,"step_id":first,"action":"start"}),
    )
    .await;
    command(
        &app,
        2,
        json!({"operation":"timer","run_id":id,"step_id":second,"action":"start"}),
    )
    .await;
    let running = detail(&app, &id).await;
    let started = running["run"]["steps"][0]["timer"]["started_at"]
        .as_i64()
        .unwrap();
    let deadline = running["run"]["steps"][0]["timer"]["deadline"]
        .as_i64()
        .unwrap();
    assert_eq!(deadline - started, 90);
    assert_eq!(running["run"]["steps"][1]["timer"]["state"], "running");
    command(
        &app,
        3,
        json!({"operation":"timer","run_id":id,"step_id":first,"action":"pause"}),
    )
    .await;
    let paused = detail(&app, &id).await;
    assert_eq!(paused["run"]["steps"][0]["timer"]["state"], "paused");
    assert_eq!(
        paused["run"]["steps"][1]["timer"],
        running["run"]["steps"][1]["timer"]
    );
    drop(app);
    let app = dir.app().unwrap();
    let restored = detail(&app, &id).await;
    assert_eq!(restored["run"], paused["run"]);
    command(
        &app,
        4,
        json!({"operation":"timer","run_id":id,"step_id":first,"action":"resume"}),
    )
    .await;
    let resumed = detail(&app, &id).await;
    let remaining = paused["run"]["steps"][0]["timer"]["remaining"]
        .as_i64()
        .unwrap();
    assert!(
        (resumed["run"]["steps"][0]["timer"]["deadline"]
            .as_i64()
            .unwrap()
            - resumed["server_now"].as_i64().unwrap()
            - remaining)
            .abs()
            <= 2
    );
    command(
        &app,
        5,
        json!({"operation":"timer","run_id":id,"step_id":first,"action":"complete"}),
    )
    .await;
    let before = detail(&app, &id).await;
    assert_eq!(before["run"]["status"], "active");
    command(
        &app,
        6,
        json!({"operation":"timer","run_id":id,"step_id":second,"action":"complete"}),
    )
    .await;
    assert_eq!(detail(&app, &id).await["run"]["status"], "completed");
}
#[tokio::test]
async fn cooking_requires_review_current_revisions_and_writer_roles_and_fails_closed_on_corruption()
{
    let dir = Directory::new();
    let app = dir.app().unwrap();
    let recipe = create_recipe(&app).await;
    let draft = template(&app, &recipe).await;
    assert_eq!(
        request(&app, "GET", "/api/household/cooking", Value::Null, None)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let reviewed = preview(&app, 0, json!({"kind":"create","draft":draft})).await;
    let body = json!({"expected_revision":0,"command":{"operation":"apply","preview_id":reviewed["preview_id"],"reviewed":true}});
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/cooking",
            body.clone(),
            Some(READER)
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let mut unreviewed = body.clone();
    unreviewed["command"]["reviewed"] = json!(false);
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/cooking",
            unreviewed,
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let applied = request(
        &app,
        "POST",
        "/api/household/cooking",
        body.clone(),
        Some(OWNER),
    )
    .await;
    assert_eq!(applied.0, StatusCode::OK);
    let id = applied.1["run_id"].as_str().unwrap();
    assert_eq!(
        request(&app, "POST", "/api/household/cooking", body, Some(OWNER))
            .await
            .0,
        StatusCode::GONE
    );
    let original = std::fs::read(dir.0.join("household/cooking.json")).unwrap();
    for (revision, operation) in [
        (0, json!({"operation":"cancel","run_id":id})),
        (
            1,
            json!({"operation":"timer","run_id":id,"step_id":"s2-1","action":"start"}),
        ),
        (
            1,
            json!({"operation":"subscribe","run_id":id,"enabled":true,"language":"en","permission":"denied"}),
        ),
    ] {
        assert!(
            !request(
                &app,
                "POST",
                "/api/household/cooking",
                json!({"expected_revision":revision,"command":operation}),
                Some(OWNER)
            )
            .await
            .0
            .is_success()
        );
        assert_eq!(
            std::fs::read(dir.0.join("household/cooking.json")).unwrap(),
            original
        );
    }
    drop(app);
    let path = dir.0.join("household/cooking.json");
    std::fs::write(&path, b"corrupt").unwrap();
    assert!(dir.app().is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"corrupt");
    std::fs::write(&path, original).unwrap();
    std::fs::write(dir.0.join("household/.cooking.pending"), b"interrupted").unwrap();
    assert!(dir.app().is_err());
}

#[tokio::test]
async fn multiday_replans_save_the_reviewed_deadlines_and_keep_delays_across_later_changes() {
    let dir = Directory::new();
    let app = dir.app().unwrap();
    let recipe = create_recipe(&app).await;
    let mut draft = template(&app, &recipe).await;
    draft["kind"] = json!("baguette");
    draft["target"] = json!({"local":"2027-10-31T02:30","timezone":"Europe/Berlin","fold":"later"});
    draft["stages"][0]["passive_seconds"] = json!(172_800);
    draft["stages"][0]["passive_min_seconds"] = json!(169_200);
    draft["stages"][0]["passive_max_seconds"] = json!(176_400);
    let reviewed = preview(&app, 0, json!({"kind":"create","draft":draft})).await;
    assert!(
        reviewed["run"]["local_schedule"][0]["start"]
            .as_str()
            .unwrap()
            .contains("+02:00")
    );
    assert!(
        reviewed["run"]["local_schedule"][1]["finish"]
            .as_str()
            .unwrap()
            .contains("+01:00")
    );
    let result = command(
        &app,
        0,
        json!({"operation":"apply","preview_id":reviewed["preview_id"],"reviewed":true}),
    )
    .await;
    let id = result["run_id"].as_str().unwrap();
    assert_eq!(
        detail(&app, id).await["run"]["schedule"],
        reviewed["run"]["schedule"]
    );
    let original = std::fs::read(dir.0.join("household/cooking.json")).unwrap();
    let delayed=preview(&app,1,json!({"kind":"replan","run_id":id,"change":{"kind":"delay","step_id":"s1-1","start_after":{"local":"2027-10-30T12:00","timezone":"Europe/Berlin","fold":"reject"}}})).await;
    assert_eq!(
        std::fs::read(dir.0.join("household/cooking.json")).unwrap(),
        original
    );
    assert!(
        delayed["proposal"]["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w["code"] == "cooking_target_missed")
    );
    assert_ne!(delayed["run"]["schedule"], delayed["previous"]["schedule"]);
    command(
        &app,
        1,
        json!({"operation":"apply","preview_id":delayed["preview_id"],"reviewed":true}),
    )
    .await;
    let again=preview(&app,2,json!({"kind":"replan","run_id":id,"change":{"kind":"target","target":{"local":"2027-10-31T08:00","timezone":"Europe/Berlin","fold":"reject"}}})).await;
    assert!(
        again["run"]["schedule"][0]["start"].as_i64().unwrap()
            >= delayed["run"]["schedule"][0]["start"].as_i64().unwrap()
    );
    for (local, fold, code) in [
        ("2027-03-28T02:30", "reject", "cooking_time_gap"),
        ("2027-10-31T02:30", "reject", "cooking_time_fold"),
    ] {
        let (status,value)=request(&app,"POST","/api/household/cooking/preview",json!({"expected_revision":2,"proposal":{"kind":"replan","run_id":id,"change":{"kind":"target","target":{"local":local,"timezone":"Europe/Berlin","fold":fold}}}}),Some(OWNER)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(value["code"], code);
    }
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/household/cooking/internal",
            json!({"operation":"claim"}),
            Some(OWNER)
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
}
