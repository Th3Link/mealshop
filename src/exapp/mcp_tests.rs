use super::{
    library_tests::{Fixture, home, join, recipe},
    tests::{SECRET, call},
};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use tower::ServiceExt;
async fn rpc(app: &Router, user: &str, path: &str, name: &str, args: Value) -> (StatusCode, Value) {
    let body = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":name,"arguments":args,"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}});
    let request = Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream")
        .header("x-mealshop-request", "1")
        .header("mcp-protocol-version", "2026-07-28")
        .header("mcp-method", "tools/call")
        .header("mcp-name", name)
        .header("ex-app-id", "mealshop")
        .header(
            "authorization-app-api",
            STANDARD.encode(format!("{user}:{SECRET}")),
        )
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), 4_194_304).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}
async fn configured() -> (Fixture, Router, String, String) {
    let fixture = Fixture::new().await;
    let app = fixture.app().await;
    let source = home(&app, "alice").await;
    let root = format!("/api/households/{source}");
    let base = format!("{root}/domain/api/household/mcp");
    join(&app, &root, "alice", "bob", "editor").await;
    join(&app, &root, "alice", "carol", "reader").await;
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "PUT",
            &format!("{base}/settings"),
            json!({"expected_revision":0,"grants":{"alice":"create","bob":"create","carol":"read"}})
        )
        .await
        .0,
        StatusCode::OK
    );
    (fixture, app, root, base)
}
#[tokio::test]
async fn mcp_grants_never_outlive_current_roles_or_convey_access_to_other_households() {
    let (_fixture, app, root, base) = configured().await;
    let other = home(&app, "eve").await;
    assert_eq!(
        call(
            &app,
            Some("bob"),
            "PUT",
            &format!("{base}/settings"),
            json!({"expected_revision":1,"grants":{}})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let preview = rpc(
        &app,
        "bob",
        &base,
        "recipe_preview_create",
        json!({"recipe":recipe()}),
    )
    .await
    .1;
    assert_eq!(preview["result"]["isError"], false);
    let insert = json!({"preview_id":preview["result"]["structuredContent"]["preview_id"],"idempotency_key":"synthetic-role-key-0001","confirmed":true});
    let created = rpc(&app, "bob", &base, "recipe_create", insert.clone())
        .await
        .1;
    assert_eq!(created["result"]["isError"], false);
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "GET",
            &format!("{base}/activity"),
            Value::Null
        )
        .await
        .1["entries"][0]["actor"],
        "bob"
    );
    assert_eq!(
        rpc(&app, "carol", &base, "recipe_create", insert.clone())
            .await
            .1["result"]["structuredContent"]["code"],
        "mcp_read_only"
    );
    assert_eq!(
        rpc(
            &app,
            "bob",
            &format!("/api/households/{other}/domain/api/household/mcp"),
            "recipe_search",
            json!({})
        )
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
        rpc(&app, "bob", &base, "recipe_create", insert).await.1["result"]["structuredContent"]["code"],
        "mcp_read_only"
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
        rpc(&app, "bob", &base, "recipe_search", json!({})).await.0,
        StatusCode::FORBIDDEN
    );
    let recipe_id = created["result"]["structuredContent"]["record"]["id"].clone();
    assert_eq!(call(&app,Some("alice"),"POST",&format!("{root}/shares"),json!({"expected_revision":0,"resource":{"kind":"recipe","recipe_id":recipe_id,"revision":1},"recipient":{"kind":"account","user":"eve"}})).await.0,StatusCode::OK);
    assert_eq!(
        rpc(&app, "eve", &base, "recipe_search", json!({})).await.0,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn preview_handles_are_bound_to_the_authenticated_account_and_ignore_forged_authority() {
    let (_fixture, app, _root, base) = configured().await;
    let preview = rpc(
        &app,
        "alice",
        &base,
        "recipe_preview_create",
        json!({"recipe":recipe()}),
    )
    .await
    .1;
    let stolen = json!({"preview_id":preview["result"]["structuredContent"]["preview_id"],"idempotency_key":"synthetic-stolen-key-001","confirmed":true});
    assert_eq!(
        rpc(&app, "bob", &base, "recipe_create", stolen).await.1["result"]["structuredContent"]["code"],
        "mcp_preview_unavailable"
    );
    assert_eq!(
        rpc(
            &app,
            "bob",
            &base,
            "recipe_preview_create",
            json!({"recipe":recipe(),"actor":"alice","household":"other"})
        )
        .await
        .1["result"]["structuredContent"]["code"],
        "invalid_recipe"
    );
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "GET",
            &format!("{base}/activity"),
            Value::Null
        )
        .await
        .1["total"],
        0
    );
}
