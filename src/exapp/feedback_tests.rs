use super::{
    library_tests::{Fixture, home, join, recipe},
    tests::call,
};
use axum::http::StatusCode;
use serde_json::{Value, json};
#[tokio::test]
async fn history_and_learning_respect_current_roles_and_do_not_leak_through_recipe_grants() {
    let fixture = Fixture::new().await;
    let app = fixture.app().await;
    let source = home(&app, "alice").await;
    let root = format!("/api/households/{source}");
    let base = format!("{root}/domain/api/household");
    join(&app, &root, "alice", "bob", "editor").await;
    join(&app, &root, "alice", "carol", "reader").await;
    assert_eq!(call(&app,Some("alice"),"PUT",&format!("{base}/profiles"),json!({"expected_revision":0,"members":[{"id":"person-a","name":"Synthetic attendee","allergies":[],"intolerances":[],"excluded_ingredients":[],"disliked_ingredients":[]}]})).await.0,StatusCode::OK);
    let saved = call(
        &app,
        Some("alice"),
        "POST",
        &format!("{root}/domain/api/recipes"),
        recipe(),
    )
    .await
    .1;
    let input = json!({"expected_revision":0,"expected_profile_revision":1,"confirmed_completed":true,"source":{"kind":"manual","recipe_id":saved["id"],"revision":1,"servings":2},"input":{"date":"2026-11-01","timezone":"Europe/Berlin","attendees":["person-a"],"active_seconds":null,"household_rating":5,"ratings":{"person-a":4}}});
    let history = format!("{base}/history");
    let learning = format!("{base}/learning");
    assert_eq!(
        call(&app, Some("carol"), "POST", &history, input.clone())
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let (status, entry) = call(&app, Some("bob"), "POST", &history, input).await;
    assert_eq!(status, StatusCode::CREATED, "{entry}");
    let read = call(&app, Some("carol"), "GET", &history, json!({}))
        .await
        .1;
    assert_eq!(read["entries"][0]["recorded_by"], "bob");
    let policy = json!({"expected_revision":1,"expected_profile_revision":1,"policy":{"enabled":true,"member_ids":["person-a"]}});
    assert_eq!(
        call(&app, Some("bob"), "PUT", &learning, policy.clone())
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(&app, Some("alice"), "PUT", &learning, policy).await.0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &app,
            Some("bob"),
            "DELETE",
            &history,
            json!({"expected_revision":2,"confirmed":true})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let granted=call(&app,Some("alice"),"POST",&format!("{root}/shares"),json!({"expected_revision":0,"resource":{"kind":"recipe","recipe_id":saved["id"],"revision":1},"recipient":{"kind":"account","user":"eve"}})).await.1;
    assert_eq!(
        call(
            &app,
            Some("eve"),
            "GET",
            &format!("/api/shared/{source}/{}", granted["id"].as_str().unwrap()),
            Value::Null
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, Some("eve"), "GET", &history, Value::Null)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(&app, Some("eve"), "GET", &learning, Value::Null)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "DELETE",
            &format!("{root}/members/bob"),
            Value::Null
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, Some("bob"), "GET", &history, Value::Null)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
}
