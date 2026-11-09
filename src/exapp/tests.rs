use super::*;
use axum::body::Body;
const SECRET: &str = "test-appapi-secret-with-at-least-32-characters";
async fn call(
    app: &Router,
    user: Option<&str>,
    method: &str,
    path: &str,
    body: Value,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .header("x-mealshop-request", "1");
    if let Some(user) = user {
        request = request.header("ex-app-id", "mealshop").header(
            "authorization-app-api",
            STANDARD.encode(format!("{user}:{SECRET}")),
        );
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), 2_097_152)
        .await
        .unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}
#[tokio::test]
async fn authenticated_households_enforce_invitation_roles_revocation_and_isolation() {
    let root = std::env::temp_dir().join(format!("mealshop-exapp-{}", std::process::id()));
    std::fs::create_dir_all(root.join("assets")).unwrap();
    for file in ["mealshop.js", "mealshop.css"] {
        std::fs::write(root.join("assets").join(file), "test").unwrap();
    }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let ocs = tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new().fallback(axum::routing::any(|| async {
                Json(json!({"ocs":{"meta":{"statuscode":100},"data":false}}))
            })),
        )
        .await
        .unwrap();
    });
    let app = router(Config {
        secret: SECRET.to_owned(),
        nextcloud_url: url,
        directory: root.join("data"),
        assets: root.join("assets"),
    })
    .await
    .unwrap();
    assert_eq!(
        call(&app, None, "GET", "/api/households", json!({}))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(&app, Some("alice"), "GET", "/api/households", json!({}))
            .await
            .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        call(&app, Some(""), "PUT", "/enabled?enabled=1", json!({}))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, Some(""), "GET", "/api/households", json!({}))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    for (marker, content_type, expected) in [
        (None, "application/json", StatusCode::FORBIDDEN),
        (Some("1"), "text/plain", StatusCode::UNSUPPORTED_MEDIA_TYPE),
    ] {
        let mut request = Request::builder()
            .method("POST")
            .uri("/api/households")
            .header("ex-app-id", "mealshop")
            .header(
                "authorization-app-api",
                STANDARD.encode(format!("alice:{SECRET}")),
            )
            .header("content-type", content_type);
        if let Some(marker) = marker {
            request = request.header("x-mealshop-request", marker);
        }
        let response = app
            .clone()
            .oneshot(
                request
                    .body(Body::from("{\"name\":\"CSRF must fail\"}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
    }
    let (_, created) = call(
        &app,
        Some("alice"),
        "POST",
        "/api/households",
        json!({"name":"Family"}),
    )
    .await;
    let id = created["id"].as_str().unwrap();
    let path = format!("/api/households/{id}");
    assert_eq!(
        call(&app, Some("bob"), "GET", &path, json!({})).await.0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &app,
            Some("bob"),
            "POST",
            &format!("{path}/invitations"),
            json!({"user":"bob","role":"editor"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "POST",
            &format!("{path}/invitations"),
            json!({"user":"bob","role":"reader"})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &app,
            Some("mallory"),
            "PUT",
            &format!("{path}/invitation"),
            json!({"accept":true})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &app,
            Some("bob"),
            "PUT",
            &format!("{path}/invitation"),
            json!({"accept":true})
        )
        .await
        .0,
        StatusCode::OK
    );
    let domain = format!("{path}/domain/api/household/profiles");
    assert_eq!(
        call(&app, Some("bob"), "GET", &domain, json!({})).await.0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &app,
            Some("bob"),
            "PUT",
            &domain,
            json!({"expected_revision":0,"members":[]})
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
            &format!("{path}/members/bob"),
            json!({"role":"editor"})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &app,
            Some("bob"),
            "PUT",
            &domain,
            json!({"expected_revision":0,"members":[]})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "DELETE",
            &format!("{path}/members/alice"),
            json!({})
        )
        .await
        .1["code"],
        "owner_transfer_required"
    );
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "DELETE",
            &format!("{path}/members/bob"),
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, Some("bob"), "GET", &domain, json!({})).await.0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(&app, Some("alice"), "GET", "/api/recipes", json!({}))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(&app, Some(""), "PUT", "/enabled?enabled=0", json!({}))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, Some("alice"), "GET", &domain, json!({})).await.0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    drop(app);
    ocs.abort();
    std::fs::remove_dir_all(root).unwrap();
}
