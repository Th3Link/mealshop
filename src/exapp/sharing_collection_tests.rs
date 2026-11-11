use super::*;
use super::{
    library_tests::{Fixture, home, join, recipe},
    sharing_tests::{grant, saved_recipe},
    tests::{SECRET, call},
};
use axum::{
    body::{Body, to_bytes},
    http::HeaderMap,
};
fn png() -> Vec<u8> {
    STANDARD.decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aL1sAAAAASUVORK5CYII=").unwrap()
}
async fn image(app: &Router, actor: &str, path: &str) -> (StatusCode, HeaderMap, Vec<u8>) {
    let request = Request::builder()
        .uri(path)
        .header("ex-app-id", "mealshop")
        .header(
            "authorization-app-api",
            STANDARD.encode(format!("{actor}:{SECRET}")),
        )
        .body(Body::empty())
        .unwrap();
    let result = app.clone().oneshot(request).await.unwrap();
    let status = result.status();
    let headers = result.headers().clone();
    let bytes = to_bytes(result.into_body(), 4_194_304)
        .await
        .unwrap()
        .to_vec();
    (status, headers, bytes)
}
async fn upload(app: &Router, home: &str) -> String {
    let request = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/households/{home}/domain/api/household/images"
        ))
        .header("ex-app-id", "mealshop")
        .header(
            "authorization-app-api",
            STANDARD.encode(format!("alice:{SECRET}")),
        )
        .header("x-mealshop-request", "1")
        .header("content-type", "image/png")
        .body(Body::from(png()))
        .unwrap();
    let result = app.clone().oneshot(request).await.unwrap();
    assert_eq!(result.status(), StatusCode::CREATED);
    let value: Value =
        serde_json::from_slice(&to_bytes(result.into_body(), 10000).await.unwrap()).unwrap();
    value["id"].as_str().unwrap().to_owned()
}
#[tokio::test]
async fn shared_images_are_pinned_no_store_and_revoked_independently_of_source_access() {
    let fixture = Fixture::new().await;
    let app = fixture.app().await;
    let home = home(&app, "alice").await;
    let id = upload(&app, &home).await;
    let mut draft = recipe();
    draft["image_id"] = json!(id);
    let source = call(
        &app,
        Some("alice"),
        "POST",
        &format!("/api/households/{home}/domain/api/recipes"),
        draft,
    )
    .await
    .1;
    let created = grant(
        &app,
        &home,
        0,
        json!({"kind":"recipe","recipe_id":source["id"],"revision":1}),
        json!({"kind":"account","user":"bob"}),
    )
    .await;
    let shared = format!("/api/shared/{home}/{}", created["id"].as_str().unwrap());
    let (status, headers, bytes) = image(&app, "bob", &format!("{shared}/image")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bytes, png());
    assert_eq!(headers["content-type"], "image/png");
    assert_eq!(headers["cache-control"], "no-store");
    assert_eq!(headers["x-content-type-options"], "nosniff");
    assert_eq!(
        image(&app, "mallory", &format!("{shared}/image")).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        image(
            &app,
            "bob",
            &format!("/api/households/{home}/domain/api/household/images/{id}")
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "DELETE",
            &format!(
                "/api/households/{home}/shares/{}",
                created["id"].as_str().unwrap()
            ),
            json!({"expected_revision":1})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        image(&app, "bob", &format!("{shared}/image")).await.0,
        StatusCode::NOT_FOUND
    );
}
struct SharedCollection {
    app: Router,
    home: String,
    shared: String,
    detail: String,
    original: Value,
    other: Value,
    draft: Value,
    _fixture: Fixture,
}
async fn shared_collection() -> SharedCollection {
    let fixture = Fixture::new().await;
    let app = fixture.app().await;
    let home = home(&app, "alice").await;
    let original = saved_recipe(&app, &home, "alice").await;
    let mut imported = recipe();
    imported["title"] = json!("Imported cached title");
    imported["author"] = json!("Imported cached author");
    imported["steps"] = json!([{"instruction":"Imported cached instruction","active_seconds":1,"passive_seconds":0}]);
    imported["image_id"] = json!(upload(&app, &home).await);
    imported["origin"] =
        json!({"kind":"imported","source_url":"https://example.org/original-recipe"});
    let other = call(
        &app,
        Some("alice"),
        "POST",
        &format!("/api/households/{home}/domain/api/recipes"),
        imported,
    )
    .await
    .1;
    let entries =
        json!([{"recipe_id":original["id"],"revision":1},{"recipe_id":other["id"],"revision":1}]);
    let draft = json!({"name":"Shared synthetic collection","entries":entries});
    let path = format!("/api/households/{home}/collections");
    let collection = call(
        &app,
        Some("alice"),
        "POST",
        &path,
        json!({"expected_revision":0,"collection":draft}),
    )
    .await
    .1;
    let created = grant(
        &app,
        &home,
        1,
        json!({"kind":"collection","collection_id":collection["id"]}),
        json!({"kind":"account","user":"bob"}),
    )
    .await;
    let shared = format!("/api/shared/{home}/{}", created["id"].as_str().unwrap());
    let detail = format!("{path}/{}", collection["id"].as_str().unwrap());
    SharedCollection {
        app,
        home,
        shared,
        detail,
        original,
        other,
        draft,
        _fixture: fixture,
    }
}
#[tokio::test]
async fn shared_collections_emit_only_source_links_for_imported_content() {
    let data = shared_collection().await;
    let app = data.app.clone();
    let shared = data.shared.clone();
    let original = &data.original;
    let other = &data.other;
    let result = call(&app, Some("bob"), "GET", &shared, json!({})).await.1;
    let link = json!({"kind":"source_link","source_url":"https://example.org/original-recipe"});
    assert_eq!(result["content"]["items"][1]["content"], link);
    assert!(!result.to_string().contains("Imported cached"));
    let imported_path = format!("{shared}/recipes/{}", other["id"].as_str().unwrap());
    assert_eq!(
        call(
            &app,
            Some("bob"),
            "GET",
            &format!("{imported_path}?revision=1&servings=3"),
            json!({})
        )
        .await
        .1["content"],
        link
    );
    assert_eq!(
        image(&app, "bob", &format!("{imported_path}/image?revision=1"))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    let owned_path = format!("{shared}/recipes/{}", original["id"].as_str().unwrap());
    assert_eq!(
        call(
            &app,
            Some("bob"),
            "GET",
            &format!("{owned_path}?revision=2"),
            json!({})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
}
#[tokio::test]
async fn only_owners_change_shared_collections_and_removed_entries_lose_access() {
    let data = shared_collection().await;
    let app = data.app.clone();
    let home = &data.home;
    let shared = data.shared.clone();
    let draft = &data.draft;
    let other = &data.other;
    let owned_path = format!("{shared}/recipes/{}", data.original["id"].as_str().unwrap());
    let base = format!("/api/households/{home}");
    join(&app, &base, "alice", "eve", "editor").await;
    let detail = data.detail.clone();
    assert_eq!(
        call(&app, Some("eve"), "GET", &detail, json!({})).await.1["editable"],
        false
    );
    assert_eq!(
        call(
            &app,
            Some("eve"),
            "PUT",
            &detail,
            json!({"expected_revision":2,"collection":draft})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let changed = json!({"name":"Shared synthetic collection","entries":[{"recipe_id":other["id"],"revision":1}]});
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "PUT",
            &detail,
            json!({"expected_revision":2,"collection":changed})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, Some("bob"), "GET", &owned_path, json!({}))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "DELETE",
            &detail,
            json!({"expected_revision":3})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, Some("bob"), "GET", &shared, json!({})).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(&app, Some("bob"), "GET", "/api/shared", json!({}))
            .await
            .1["total"],
        0
    );
}
