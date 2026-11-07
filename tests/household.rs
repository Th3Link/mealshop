//! Protected household acceptance tests.
#![allow(clippy::unwrap_used)]
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use tower::ServiceExt;
const TOKEN: &str = "test_household_operator_token_123456789";
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
    assert_eq!(response.headers()["cache-control"], "no-store");
    let status = response.status();
    (
        status,
        serde_json::from_slice(&to_bytes(response.into_body(), 1_048_576).await.unwrap()).unwrap(),
    )
}
#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn profiles_enforce_access_revision_constraints_restart_export_and_removal() {
    let path = std::env::temp_dir().join(format!("mealshop-household-{}", std::process::id()));
    std::fs::create_dir(&path).unwrap();
    let catalogue = include_str!("../data/catalogue.json");
    let app = mealshop_server::household_router(catalogue, &path, TOKEN.to_owned()).unwrap();
    for token in [None, Some("wrong")] {
        assert_eq!(
            request(&app, "GET", "/api/household/profiles", Value::Null, token)
                .await
                .0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            request(
                &app,
                "PUT",
                "/api/household/profiles",
                json!({"expected_revision":0,"members":[]}),
                token
            )
            .await
            .0,
            StatusCode::UNAUTHORIZED
        );
    }
    let members = json!([{"id":"parent","name":"Local parent","allergies":["milk"],"excluded_ingredients":["oats"],"disliked_ingredients":["chia"]},{"id":"child","name":"Local child","allergies":[],"excluded_ingredients":[],"disliked_ingredients":[]}]);
    let write = json!({"expected_revision":0,"members":members});
    let saved = request(
        &app,
        "PUT",
        "/api/household/profiles",
        write.clone(),
        Some(TOKEN),
    )
    .await;
    assert_eq!(saved.0, StatusCode::OK);
    assert_eq!(saved.1["revision"], 1);
    assert_eq!(
        request(&app, "PUT", "/api/household/profiles", write, Some(TOKEN))
            .await
            .0,
        StatusCode::CONFLICT
    );
    let evaluation = json!({"member_ids":["child","parent"],"ingredient_ids":["Haferkorn","chia"]});
    let result = request(
        &app,
        "POST",
        "/api/household/evaluate",
        evaluation,
        Some(TOKEN),
    )
    .await
    .1;
    assert_eq!(result["status"], "blocked");
    assert_eq!(result["excluded_ingredients"], json!(["oats"]));
    assert_eq!(result["disliked_ingredients"], json!(["chia"]));
    let result = request(
        &app,
        "POST",
        "/api/household/evaluate",
        json!({"member_ids":["parent"],"ingredient_ids":["chia","unknown"]}),
        Some(TOKEN),
    )
    .await
    .1;
    assert_eq!(result["status"], "review_required");
    assert_eq!(result["unknown_ingredients"], json!(["unknown"]));
    assert!(mealshop_server::household_router(catalogue, &path, TOKEN.to_owned()).is_err());
    drop(app);
    let app = mealshop_server::household_router(catalogue, &path, TOKEN.to_owned()).unwrap();
    assert_eq!(
        request(
            &app,
            "GET",
            "/api/household/profiles",
            Value::Null,
            Some(TOKEN)
        )
        .await
        .1,
        saved.1
    );
    assert_eq!(
        request(
            &app,
            "PUT",
            "/api/household/profiles",
            json!({"expected_revision":1,"members":[]}),
            Some(TOKEN)
        )
        .await
        .0,
        StatusCode::OK
    );
    drop(app);
    let app = mealshop_server::household_router(catalogue, &path, TOKEN.to_owned()).unwrap();
    assert_eq!(
        request(
            &app,
            "GET",
            "/api/household/profiles",
            Value::Null,
            Some(TOKEN)
        )
        .await
        .1["members"],
        json!([])
    );
    drop(app);
    std::fs::remove_dir_all(path).unwrap();
}

#[tokio::test]
async fn invalid_profiles_and_interrupted_writes_fail_without_exposing_or_replacing_data() {
    let path =
        std::env::temp_dir().join(format!("mealshop-household-errors-{}", std::process::id()));
    std::fs::create_dir(&path).unwrap();
    let catalogue = include_str!("../data/catalogue.json");
    assert!(mealshop_server::household_router(catalogue, &path, "short".to_owned()).is_err());
    let app = mealshop_server::household_router(catalogue, &path, TOKEN.to_owned()).unwrap();
    let invalid = json!({"expected_revision":0,"members":[{"id":"person","name":"Test","allergies":[],"excluded_ingredients":["not-in-catalogue"],"disliked_ingredients":[]}]});
    assert_eq!(
        request(&app, "PUT", "/api/household/profiles", invalid, Some(TOKEN))
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    std::fs::write(path.join(".pending"), "interrupted").unwrap();
    for _ in 0..2 {
        assert_eq!(
            request(
                &app,
                "PUT",
                "/api/household/profiles",
                json!({"expected_revision":0,"members":[]}),
                Some(TOKEN)
            )
            .await
            .0,
            StatusCode::SERVICE_UNAVAILABLE
        );
    }
    assert!(!path.join("profiles.json").exists());
    drop(app);
    assert!(mealshop_server::household_router(catalogue, &path, TOKEN.to_owned()).is_err());
    assert!(!path.join(".household.lock").exists());
    std::fs::remove_file(path.join(".pending")).unwrap();
    std::fs::write(path.join("profiles.json"), "corrupt").unwrap();
    assert!(mealshop_server::household_router(catalogue, &path, TOKEN.to_owned()).is_err());
    std::fs::remove_dir_all(path).unwrap();
}
