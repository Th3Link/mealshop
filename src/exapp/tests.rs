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
        public_url: None,
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

#[tokio::test]
async fn cooking_notifications_use_only_the_opted_in_actor_and_hide_other_subscriptions() {
    let root = std::env::temp_dir().join(format!("mealshop-cooking-exapp-{}", std::process::id()));
    std::fs::create_dir_all(root.join("assets")).unwrap();
    for file in ["mealshop.js", "mealshop.css"] {
        std::fs::write(root.join("assets").join(file), "test").unwrap();
    }
    let notices = Arc::new(Mutex::new(Vec::<Value>::new()));
    let collected = notices.clone();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let ocs = tokio::spawn(async move {
        axum::serve(listener,Router::new().fallback(axum::routing::any(move |headers: axum::http::HeaderMap,body: axum::body::Bytes|{let notices=collected.clone();async move {if let Ok(value)=serde_json::from_slice::<Value>(&body) {if value.pointer("/params/object")==Some(&json!("mealshop_cooking")){notices.lock().unwrap().push(json!({"authorization":headers.get("authorization-app-api").unwrap().to_str().unwrap(),"value":value}));}}Json(json!({"ocs":{"meta":{"statuscode":100},"data":true}}))}}))).await.unwrap();
    });
    let app = router(Config {
        secret: SECRET.to_owned(),
        nextcloud_url: url,
        public_url: Some("https://cloud.example.test".to_owned()),
        directory: root.join("data"),
        assets: root.join("assets"),
    })
    .await
    .unwrap();
    let (status, home) = call(
        &app,
        Some("alice"),
        "POST",
        "/api/households",
        json!({"name":"Cooking fixture"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{home}");
    let id = home["id"].as_str().unwrap();
    let prefix = format!("/api/households/{id}/domain");
    let (status,recipe)=call(&app,Some("alice"),"POST",&format!("{prefix}/api/recipes"),json!({"title":"Test recipe","language":"en","servings":1,"preparation":"as_recorded","lines":[{"ingredient_id":"oats","grams":"1"}],"steps":[{"instruction":"Check test dough","active_seconds":0,"passive_seconds":1}]})).await;
    assert_eq!(status, StatusCode::CREATED, "{recipe}");
    let (_,template)=call(&app,Some("alice"),"GET",&format!("{prefix}/api/household/cooking/template?recipe_id={}&revision=1&servings=1&timezone=UTC",recipe["id"].as_str().unwrap()),json!({})).await;
    let (_, review) = call(
        &app,
        Some("alice"),
        "POST",
        &format!("{prefix}/api/household/cooking/preview"),
        json!({"expected_revision":0,"proposal":{"kind":"create","draft":template["draft"]}}),
    )
    .await;
    let (_,applied)=call(&app,Some("alice"),"POST",&format!("{prefix}/api/household/cooking"),json!({"expected_revision":0,"command":{"operation":"apply","preview_id":review["preview_id"],"reviewed":true}})).await;
    let run = applied["run_id"].as_str().unwrap();
    for (revision, command) in [
        (
            1,
            json!({"operation":"timer","run_id":run,"step_id":"s1-1","action":"start"}),
        ),
        (
            2,
            json!({"operation":"subscribe","run_id":run,"enabled":true,"language":"en","permission":"denied"}),
        ),
    ] {
        let (status, value) = call(
            &app,
            Some("alice"),
            "POST",
            &format!("{prefix}/api/household/cooking"),
            json!({"expected_revision":revision,"command":command}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{value}");
    }
    for _ in 0..30 {
        if !notices.lock().unwrap().is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    {
        let notices = notices.lock().unwrap();
        assert_eq!(notices.len(), 1);
        assert_eq!(
            notices[0]["authorization"],
            STANDARD.encode(format!("alice:{SECRET}"))
        );
        assert!(
            notices[0]["value"]["params"]["subject_params"]["link"]
                .as_str()
                .unwrap()
                .starts_with(
                    "https://cloud.example.test/index.php/apps/app_api/embedded/mealshop/main/h/"
                )
        );
    }
    let (_, detail) = call(
        &app,
        Some("alice"),
        "GET",
        &format!("{prefix}/api/household/cooking/{run}"),
        json!({}),
    )
    .await;
    assert!(detail["run"].get("subscriptions").is_none());
    assert_eq!(detail["notices"][0]["delivery"]["state"], "accepted");
    assert!(detail["notices"][0].get("user").is_none());
    assert_eq!(detail["run"]["subscription"]["permission"], "denied");
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "POST",
            &format!("{prefix}/api/household/cooking/internal"),
            json!({"operation":"claim"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    drop(app);
    ocs.abort();
    std::fs::remove_dir_all(root).unwrap();
}
