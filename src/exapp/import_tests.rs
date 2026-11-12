use super::{
    library_tests::{Fixture, home, join, recipe},
    tests::call,
};
use axum::http::StatusCode;
use serde_json::{Value, json};
#[tokio::test]
async fn import_previews_follow_current_accounts_roles_and_household_boundaries() {
    let fixture = Fixture::new().await;
    let app = fixture.app().await;
    let home = home(&app, "alice").await;
    let root = format!("/api/households/{home}");
    let imports = format!("{root}/domain/api/household/imports");
    join(&app, &root, "alice", "bob", "editor").await;
    let preview = call(
        &app,
        Some("bob"),
        "POST",
        &format!("{imports}/preview"),
        json!({"mode":"manual","url":"https://example.org/import-original"}),
    )
    .await;
    assert_eq!(preview.0, StatusCode::OK);
    let mut reviewed = recipe();
    reviewed["origin"] =
        json!({"kind":"imported","source_url":"https://example.org/import-original"});
    let payload = json!({"preview_id":preview.1["preview_id"],"candidate":0,"idempotency_key":"synthetic-account-import-0001","reviewed":true,"recipe":reviewed});
    assert_eq!(
        call(&app, Some("alice"), "POST", &imports, payload.clone())
            .await
            .1["code"],
        "import_preview_unavailable"
    );
    assert_eq!(
        call(&app, Some("eve"), "POST", &imports, payload.clone())
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "PUT",
            &format!("{root}/members/bob"),
            json!({"role":"reader"})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, Some("bob"), "POST", &imports, payload.clone())
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "PUT",
            &format!("{root}/members/bob"),
            json!({"role":"editor"})
        )
        .await
        .0,
        StatusCode::OK
    );
    let saved = call(&app, Some("bob"), "POST", &imports, payload).await;
    assert_eq!(saved.0, StatusCode::CREATED);
    let shared=call(&app,Some("alice"),"POST",&format!("{root}/shares"),json!({"expected_revision":0,"resource":{"kind":"recipe","recipe_id":saved.1["id"],"revision":1},"recipient":{"kind":"account","user":"eve"}})).await;
    assert_eq!(shared.0, StatusCode::OK);
    let granted = call(
        &app,
        Some("eve"),
        "GET",
        &format!("/api/shared/{home}/{}", shared.1["id"].as_str().unwrap()),
        Value::Null,
    )
    .await;
    assert_eq!(granted.0, StatusCode::OK);
    assert_eq!(
        granted.1["content"],
        json!({"kind":"source_link","source_url":"https://example.org/import-original"})
    );
    assert_eq!(
        std::fs::read_dir(fixture.root.join("data").join(&home).join("recipes"))
            .unwrap()
            .filter_map(Result::ok)
            .filter(|f| f.path().extension().is_some_and(|e| e == "json"))
            .count(),
        1
    );
}
