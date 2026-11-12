//! MCP protocol, privacy, creation and retry behavior through the protected family boundary.
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
const OWNER: &str = "mcp_owner_test_credential_1234567890";
const READER: &str = "mcp_reader_test_credential_123456789";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "mealshop-mcp-api-{}-{}",
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
    if path.ends_with("/mcp") {
        r = r
            .header("accept", "application/json, text/event-stream")
            .header("mcp-protocol-version", "2026-07-28");
        if let Some(method) = body["method"].as_str() {
            r = r.header("mcp-method", method);
        }
        if let Some(name) = body["params"]["name"].as_str() {
            r = r.header("mcp-name", name);
        }
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

fn request(id: u64, method: &str, mut params: Value) -> Value {
    params["_meta"] = json!({"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}});
    json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})
}
async fn rpc(
    app: &axum::Router,
    token: Option<&str>,
    name: &str,
    args: Value,
) -> (StatusCode, Value) {
    call(
        app,
        token,
        "POST",
        "/api/household/mcp",
        request(1, "tools/call", json!({"name":name,"arguments":args})),
    )
    .await
}
async fn enable(app: &axum::Router) {
    let (status, value) = call(
        app,
        Some(OWNER),
        "PUT",
        "/api/household/mcp/settings",
        json!({"expected_revision":0,"grants":{"local":"create","local-reader":"read"}}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
}
fn recipe() -> Value {
    json!({"title":"Synthetic MCP acceptance recipe","language":"en","servings":2,"preparation":"as_recorded","lines":[{"ingredient_id":"oats","quantity":{"amount":"0.100500","unit":"kg"}}],"steps":[{"instruction":"Prepare synthetic fixture","active_seconds":60,"passive_seconds":0}]})
}
async fn preview(app: &axum::Router, input: Value) -> Value {
    let (status, value) = rpc(
        app,
        Some(OWNER),
        "recipe_preview_create",
        json!({"recipe":input}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["result"]["isError"], false, "{value}");
    value["result"]["structuredContent"].clone()
}
fn insert(preview: &Value) -> Value {
    json!({"preview_id":preview["preview_id"],"idempotency_key":"synthetic-mcp-key-0001","confirmed":true})
}
#[tokio::test]
async fn mcp_is_disabled_by_default_and_intersects_explicit_grants_with_current_authority() {
    let dir = Directory::new();
    let app = dir.app();
    assert_eq!(
        rpc(&app, None, "recipe_search", json!({})).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        rpc(&app, Some(OWNER), "recipe_search", json!({})).await.0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &app,
            Some(READER),
            "PUT",
            "/api/household/mcp/settings",
            json!({"expected_revision":0,"grants":{}})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "PUT",
            "/api/household/mcp/settings",
            json!({"expected_revision":0,"grants":{"local-reader":"create"}})
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    enable(&app).await;
    let (status, tools) = call(
        &app,
        Some(OWNER),
        "POST",
        "/api/household/mcp",
        request(1, "tools/list", json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 6);
    let tools = call(
        &app,
        Some(READER),
        "POST",
        "/api/household/mcp",
        request(1, "tools/list", json!({})),
    )
    .await
    .1;
    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 5);
    let draft = preview(&app, recipe()).await;
    assert_eq!(
        rpc(&app, Some(READER), "recipe_create", insert(&draft))
            .await
            .1["result"]["structuredContent"]["code"],
        "mcp_read_only"
    );
    assert_eq!(
        rpc(
            &app,
            Some(READER),
            "recipe_search",
            json!({"household":"another-home"})
        )
        .await
        .1["result"]["isError"],
        true
    );
    let (status, _) = call(
        &app,
        Some(OWNER),
        "PUT",
        "/api/household/mcp/settings",
        json!({"expected_revision":1,"grants":{}}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        rpc(
            &app,
            Some(OWNER),
            "recipe_get",
            json!({"id":"recipe-00000001"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
}
#[tokio::test]
async fn canonical_preview_confirmation_and_invalid_quantities_never_silently_insert() {
    let dir = Directory::new();
    let app = dir.app();
    enable(&app).await;
    let mut invalid = recipe();
    invalid["lines"][0] = json!({"ingredient_id":"oats","grams":0.1});
    let result = rpc(
        &app,
        Some(OWNER),
        "recipe_preview_create",
        json!({"recipe":invalid}),
    )
    .await
    .1;
    assert_eq!(result["result"]["isError"], true);
    assert_eq!(
        result["result"]["structuredContent"]["code"],
        "invalid_recipe"
    );
    let mut invalid = recipe();
    invalid["lines"][0]["ingredient_id"] = json!("unknown-food");
    assert_eq!(
        rpc(
            &app,
            Some(OWNER),
            "recipe_preview_create",
            json!({"recipe":invalid})
        )
        .await
        .1["result"]["isError"],
        true
    );
    let draft = preview(&app, recipe()).await;
    assert_eq!(draft["saved"], false);
    assert_eq!(
        draft["content"]["recipe"]["lines"][0]["grams"],
        "100.500000"
    );
    assert_eq!(
        call(&app, None, "GET", "/api/recipes", Value::Null).await.1["recipes"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let mut body = insert(&draft);
    body["confirmed"] = json!(false);
    assert_eq!(
        rpc(&app, Some(OWNER), "recipe_create", body).await.1["result"]["structuredContent"]["code"],
        "mcp_confirmation_required"
    );
    let created = rpc(&app, Some(OWNER), "recipe_create", insert(&draft))
        .await
        .1;
    assert_eq!(created["result"]["isError"], false, "{created}");
    let id = created["result"]["structuredContent"]["record"]["id"]
        .as_str()
        .unwrap();
    let regular = call(
        &app,
        None,
        "GET",
        &format!("/api/recipes/{id}"),
        Value::Null,
    )
    .await
    .1;
    assert_eq!(regular["recipe"]["lines"][0]["grams"], "100.500000");
    assert!(regular.get("creation").is_none());
    let activity = call(
        &app,
        Some(READER),
        "GET",
        "/api/household/mcp/activity",
        Value::Null,
    )
    .await
    .1;
    assert_eq!(activity["total"], 1);
    assert_eq!(activity["entries"][0]["actor"], "local");
    assert!(activity.to_string().find("synthetic-mcp-key").is_none());
}
#[tokio::test]
async fn concurrent_insert_retries_survive_restart_and_reject_conflicting_key_reuse() {
    let dir = Directory::new();
    let app = dir.app();
    enable(&app).await;
    let draft = preview(&app, recipe()).await;
    let body = insert(&draft);
    let first = rpc(&app, Some(OWNER), "recipe_create", body.clone());
    let second = rpc(&app, Some(OWNER), "recipe_create", body.clone());
    let (first, second) = tokio::join!(first, second);
    assert_eq!(first.1["result"]["isError"], false);
    assert_eq!(second.1["result"]["isError"], false);
    assert_eq!(
        first.1["result"]["structuredContent"]["record"]["id"],
        second.1["result"]["structuredContent"]["record"]["id"]
    );
    drop(app);
    let app = dir.app();
    let retry = rpc(&app, Some(OWNER), "recipe_create", body).await.1;
    assert_eq!(retry["result"]["structuredContent"]["replayed"], true);
    let another = preview(&app, recipe()).await;
    let conflict = rpc(&app, Some(OWNER), "recipe_create", insert(&another))
        .await
        .1;
    assert_eq!(
        conflict["result"]["structuredContent"]["code"],
        "mcp_idempotency_conflict"
    );
    assert_eq!(
        call(&app, None, "GET", "/api/recipes", Value::Null).await.1["recipes"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
#[tokio::test]
async fn imported_content_and_retroactively_restricted_revisions_are_source_only() {
    let dir = Directory::new();
    let app = dir.app();
    enable(&app).await;
    let draft = preview(&app, recipe()).await;
    let created = rpc(&app, Some(OWNER), "recipe_create", insert(&draft))
        .await
        .1;
    let id = created["result"]["structuredContent"]["record"]["id"]
        .as_str()
        .unwrap();
    let mut imported = recipe();
    imported["origin"] =
        json!({"kind":"imported","source_url":"https://example.org/original-mcp-recipe"});
    assert_eq!(
        call(
            &app,
            None,
            "PUT",
            &format!("/api/recipes/{id}"),
            json!({"expected_revision":1,"recipe":imported})
        )
        .await
        .0,
        StatusCode::OK
    );
    let read = rpc(
        &app,
        Some(OWNER),
        "recipe_get",
        json!({"id":id,"revision":1}),
    )
    .await
    .1;
    assert_eq!(
        read["result"]["structuredContent"]["content"],
        json!({"kind":"source_link","source_url":"https://example.org/original-mcp-recipe"})
    );
    assert!(!read.to_string().contains("Synthetic MCP acceptance recipe"));
    let search = rpc(
        &app,
        Some(OWNER),
        "recipe_search",
        json!({"query":"Synthetic MCP acceptance recipe"}),
    )
    .await
    .1;
    assert_eq!(search["result"]["structuredContent"]["total"], 0);
    let imported = preview(&app, imported).await;
    assert_eq!(imported["content"]["kind"], "source_link");
    assert!(imported["content"].get("recipe").is_none());
}
async fn wire(app: &axum::Router, body: &str, headers: &[(&str, &str)]) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method("POST")
        .uri("/api/household/mcp")
        .header("authorization", format!("Bearer {OWNER}"));
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(body.to_owned())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), 4_194_304).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}
#[tokio::test]
async fn modern_metadata_and_mirrored_headers_cannot_disagree_or_bypass_request_boundaries() {
    let dir = Directory::new();
    let app = dir.app();
    enable(&app).await;
    let base = [
        ("content-type", "application/json"),
        ("accept", "application/json, text/event-stream"),
        ("mcp-protocol-version", "2026-07-28"),
        ("mcp-method", "server/discover"),
    ];
    let discover = request(1, "server/discover", json!({}));
    let result = wire(&app, &discover.to_string(), &base).await;
    assert_eq!(result.0, StatusCode::OK);
    assert_eq!(result.1["result"]["resultType"], "complete");
    let mut wrong = base;
    wrong[3].1 = "tools/list";
    assert_eq!(
        wire(&app, &discover.to_string(), &wrong).await.1["error"]["code"],
        -32020
    );
    let mut wrong = discover.clone();
    wrong["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"] = json!("2025-11-25");
    assert_eq!(
        wire(&app, &wrong.to_string(), &base).await.1["error"]["code"],
        -32020
    );
    let mut missing = discover.clone();
    missing["params"]["_meta"]
        .as_object_mut()
        .unwrap()
        .remove("io.modelcontextprotocol/clientCapabilities");
    assert_eq!(
        wire(&app, &missing.to_string(), &base).await.1["error"]["code"],
        -32602
    );
    let mut unsupported = discover.clone();
    unsupported["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"] = json!("1900-01-01");
    assert_eq!(
        wire(&app, &unsupported.to_string(), &base).await.1["error"]["code"],
        -32022
    );
    let mut origin = base.to_vec();
    origin.push(("origin", "https://example.org"));
    assert_eq!(
        wire(&app, &discover.to_string(), &origin).await.0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(wire(&app, "{", &base).await.1["error"]["code"], -32700);
    assert_eq!(wire(&app, "[]", &base).await.1["error"]["code"], -32600);
    let mut no_id = discover.clone();
    no_id.as_object_mut().unwrap().remove("id");
    assert_eq!(
        wire(&app, &no_id.to_string(), &base).await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        wire(&app, &discover.to_string(), &base[..1]).await.0,
        StatusCode::NOT_ACCEPTABLE
    );
    assert_eq!(
        wire(&app, &"x".repeat(131_073), &base).await.0,
        StatusCode::PAYLOAD_TOO_LARGE
    );
}
#[tokio::test]
async fn legacy_clients_initialize_then_use_the_same_scoped_tools_and_empty_notifications() {
    let dir = Directory::new();
    let app = dir.app();
    enable(&app).await;
    let base = [
        ("content-type", "application/json"),
        ("accept", "application/json, text/event-stream"),
    ];
    let init = json!({"jsonrpc":"2.0","id":"init","method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"synthetic-client","version":"1"}}});
    let (status, value) = wire(&app, &init.to_string(), &base).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["result"]["protocolVersion"], "2025-11-25");
    assert!(value["result"].get("resultType").is_none());
    let mut headers = base.to_vec();
    headers.push(("mcp-protocol-version", "2025-11-25"));
    assert_eq!(
        wire(
            &app,
            &json!({"jsonrpc":"2.0","method":"notifications/initialized"}).to_string(),
            &headers
        )
        .await
        .0,
        StatusCode::ACCEPTED
    );
    let listed = wire(
        &app,
        &json!({"jsonrpc":"2.0","id":8,"method":"tools/list"}).to_string(),
        &headers,
    )
    .await;
    assert_eq!(listed.0, StatusCode::OK);
    assert_eq!(listed.1["result"]["tools"].as_array().unwrap().len(), 6);
    let read = json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"ingredient_search","arguments":{"query":"Haferkorn"}}});
    let result = wire(&app, &read.to_string(), &headers).await;
    assert_eq!(result.0, StatusCode::OK);
    assert_eq!(result.1["result"]["isError"], false);
    assert!(
        result.1["result"]["structuredContent"]["total"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert_eq!(
        call(&app, Some(OWNER), "GET", "/api/household/mcp", Value::Null)
            .await
            .0,
        StatusCode::METHOD_NOT_ALLOWED
    );
}
#[tokio::test]
async fn changed_ingredients_and_revoked_scopes_require_an_explicit_new_preview() {
    let dir = Directory::new();
    let app = dir.app();
    enable(&app).await;
    let draft = preview(&app, recipe()).await;
    let ingredient = json!({"expected_revision":0,"ingredient":{"name":"Synthetic MCP ingredient","language":"en","form":"As declared","recorded_on":"2026-11-01"}});
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/ingredients",
            ingredient
        )
        .await
        .0,
        StatusCode::CREATED
    );
    assert_eq!(
        rpc(&app, Some(OWNER), "recipe_create", insert(&draft))
            .await
            .1["result"]["structuredContent"]["code"],
        "mcp_ingredients_changed"
    );
    let fresh = preview(&app, recipe()).await;
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "PUT",
            "/api/household/mcp/settings",
            json!({"expected_revision":1,"grants":{"local":"read","local-reader":"read"}})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        rpc(&app, Some(OWNER), "recipe_create", insert(&fresh))
            .await
            .1["result"]["structuredContent"]["code"],
        "mcp_read_only"
    );
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "PUT",
            "/api/household/mcp/settings",
            json!({"expected_revision":2,"grants":{"local":"create","local-reader":"read"}})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        rpc(&app, Some(OWNER), "recipe_create", insert(&fresh))
            .await
            .1["result"]["structuredContent"]["code"],
        "mcp_preview_expired"
    );
    let fresh = preview(&app, recipe()).await;
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "PUT",
            "/api/household/mcp/settings",
            json!({"expected_revision":3,"grants":{"local":"create","local-reader":"read"}})
        )
        .await
        .1["revision"],
        3
    );
    assert_eq!(
        rpc(&app, Some(OWNER), "recipe_create", insert(&fresh))
            .await
            .1["result"]["isError"],
        false
    );
}

#[tokio::test]
async fn failed_scope_revocation_blocks_all_tools_instead_of_using_stale_grants() {
    let directory = Directory::new();
    let app = directory.app();
    enable(&app).await;
    let draft = preview(&app, recipe()).await;
    std::fs::write(directory.0.join("household/.mcp.pending"), b"interrupted").unwrap();
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "PUT",
            "/api/household/mcp/settings",
            json!({"expected_revision":1,"grants":{}})
        )
        .await
        .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        rpc(&app, Some(OWNER), "recipe_search", json!({})).await.0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        rpc(&app, Some(OWNER), "recipe_create", insert(&draft))
            .await
            .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "GET",
            "/api/household/mcp/settings",
            Value::Null
        )
        .await
        .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        call(&app, Some(OWNER), "GET", "/api/recipes", Value::Null)
            .await
            .1,
        json!({"recipes":[]})
    );
}
