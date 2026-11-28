#![allow(clippy::unwrap_used, clippy::too_many_lines)]
use super::tests::{SECRET, call};
use super::*;
use axum::body::Body;
use std::sync::Mutex as Lock;

#[derive(Default)]
struct Fake {
    status: &'static str,
    uploads: Vec<(String, usize)>,
    deleted: Vec<String>,
    scheduled: Vec<Value>,
    callers: Vec<String>,
}
type Seen = Arc<Lock<Fake>>;

fn caller(request: &Request) -> String {
    let header = request.headers()["authorization-app-api"].to_str().unwrap();
    let decoded = String::from_utf8(STANDARD.decode(header).unwrap()).unwrap();
    decoded.split_once(':').unwrap().0.to_owned()
}
async fn nextcloud(State(seen): State<Seen>, request: Request) -> Response {
    let path = request.uri().path().to_owned();
    let method = request.method().as_str().to_owned();
    let who = caller(&request);
    seen.lock().unwrap().callers.push(who);
    let ok =
        |data: Value| Json(json!({"ocs":{"meta":{"statuscode":200},"data":data}})).into_response();
    match (method.as_str(), path.as_str()) {
        ("GET", "/ocs/v2.php/taskprocessing/tasktypes") => {
            ok(json!({"types":{"core:audio2text":{},"core:image2text:ocr":{},"core:text2text":{}}}))
        }
        ("POST", "/ocs/v2.php/taskprocessing/schedule") => {
            let body = axum::body::to_bytes(request.into_body(), 65_536)
                .await
                .unwrap();
            seen.lock()
                .unwrap()
                .scheduled
                .push(serde_json::from_slice(&body).unwrap());
            ok(json!({"task":{"id":7,"status":"STATUS_SCHEDULED"}}))
        }
        ("GET", "/ocs/v2.php/taskprocessing/task/7") => {
            let status = seen.lock().unwrap().status;
            ok(json!({"task":{"id":7,"status":status,"progress":0.5,
                "output":{"output":"Synthetic bowl\n100 g Oats"}}}))
        }
        ("MKCOL", _) => StatusCode::CREATED.into_response(),
        ("PUT", _) => {
            let body = axum::body::to_bytes(request.into_body(), 1_048_576)
                .await
                .unwrap();
            seen.lock().unwrap().uploads.push((path, body.len()));
            (StatusCode::CREATED, [("OC-FileId", "00000042ocsynthetic")]).into_response()
        }
        ("DELETE", _) => {
            seen.lock().unwrap().deleted.push(path);
            StatusCode::NO_CONTENT.into_response()
        }
        _ => ok(json!(false)),
    }
}
async fn upload(
    app: &Router,
    user: &str,
    kind: &str,
    mime: &str,
    body: &[u8],
) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/assist/transcriptions?kind={kind}"))
                .header("content-type", mime)
                .header("x-mealshop-request", "1")
                .header("ex-app-id", "mealshop")
                .header(
                    "authorization-app-api",
                    STANDARD.encode(format!("{user}:{SECRET}")),
                )
                .body(Body::from(body.to_vec()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), 65_536)
        .await
        .unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}

#[tokio::test]
async fn transcription_runs_as_the_user_and_removes_its_temporary_file() {
    let root = std::env::temp_dir().join(format!("mealshop-assist-{}", std::process::id()));
    std::fs::create_dir_all(root.join("assets")).unwrap();
    for file in ["mealshop.js", "mealshop.css"] {
        std::fs::write(root.join("assets").join(file), "test").unwrap();
    }
    let seen = Seen::default();
    seen.lock().unwrap().status = "STATUS_SCHEDULED";
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let fake = Router::new().fallback(nextcloud).with_state(seen.clone());
    tokio::spawn(async move { axum::serve(listener, fake).await.unwrap() });
    let app = router(Config {
        secret: SECRET.to_owned(),
        nextcloud_url: url,
        public_url: None,
        directory: root.join("data"),
        assets: root.join("assets"),
    })
    .await
    .unwrap();
    call(&app, Some(""), "PUT", "/enabled?enabled=1", json!({})).await;

    assert_eq!(
        call(&app, None, "GET", "/api/assist", json!({})).await.0,
        StatusCode::UNAUTHORIZED
    );
    // Only task types with a provider are offered; a text model alone offers nothing here.
    assert_eq!(
        call(&app, Some("alice"), "GET", "/api/assist", json!({}))
            .await
            .1,
        json!({"speech":true,"picture":true,"handwriting":false})
    );
    assert_eq!(
        upload(&app, "alice", "speech", "text/plain", b"x").await,
        (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            json!({"code":"assist_media_unsupported"})
        )
    );
    assert_eq!(
        upload(&app, "alice", "handwriting", "image/png", b"x").await,
        (StatusCode::CONFLICT, json!({"code":"assist_unavailable"}))
    );
    assert_eq!(seen.lock().unwrap().uploads.len(), 0);

    let started = upload(
        &app,
        "alice",
        "speech",
        "audio/webm;codecs=opus",
        b"synthetic audio",
    )
    .await;
    assert_eq!(
        started,
        (
            StatusCode::ACCEPTED,
            json!({"id":7,"kind":"speech","status":"waiting"})
        )
    );
    let file = {
        let seen = seen.lock().unwrap();
        let (path, bytes) = seen.uploads[0].clone();
        assert!(path.starts_with("/remote.php/dav/files/alice/.mealshop-capture/"));
        assert_eq!(path.rsplit_once('.').unwrap().1, "webm");
        assert_eq!(bytes, 15);
        assert_eq!(seen.scheduled[0]["type"], "core:audio2text");
        assert_eq!(seen.scheduled[0]["appId"], "mealshop");
        assert_eq!(seen.scheduled[0]["input"], json!({"input":42}));
        // Every call to Nextcloud is made as the requesting account.
        // (the app's own enable callback is the only call without one).
        assert!(
            seen.callers
                .iter()
                .all(|who| who == "alice" || who.is_empty())
        );
        assert!(seen.callers.iter().filter(|who| *who == "alice").count() >= 5);
        path
    };
    let path = "/api/assist/transcriptions/7";
    // Another account cannot read or end the task.
    assert_eq!(
        call(&app, Some("bob"), "GET", path, json!({})).await,
        (StatusCode::NOT_FOUND, json!({"code":"assist_not_found"}))
    );
    assert_eq!(
        call(&app, Some("alice"), "GET", path, json!({})).await.1,
        json!({"id":7,"kind":"speech","status":"waiting"})
    );
    seen.lock().unwrap().status = "STATUS_RUNNING";
    assert_eq!(
        call(&app, Some("alice"), "GET", path, json!({})).await.1["status"],
        "running"
    );
    assert_eq!(seen.lock().unwrap().deleted.len(), 0);
    seen.lock().unwrap().status = "STATUS_SUCCESSFUL";
    assert_eq!(
        call(&app, Some("alice"), "GET", path, json!({})).await.1,
        json!({"id":7,"kind":"speech","status":"done","text":"Synthetic bowl\n100 g Oats"})
    );
    assert_eq!(seen.lock().unwrap().deleted, vec![file]);
    assert_eq!(
        call(&app, Some("alice"), "GET", path, json!({})).await.0,
        StatusCode::NOT_FOUND
    );

    // A failed task also removes its upload; a picture is scheduled as a list of files.
    seen.lock().unwrap().status = "STATUS_FAILED";
    assert_eq!(
        upload(&app, "alice", "picture", "image/png", b"png")
            .await
            .0,
        StatusCode::ACCEPTED
    );
    assert_eq!(
        seen.lock().unwrap().scheduled[1]["input"],
        json!({"input":[42]})
    );
    assert_eq!(
        call(&app, Some("alice"), "GET", path, json!({})).await.1["status"],
        "failed"
    );
    assert_eq!(seen.lock().unwrap().deleted.len(), 2);
    std::fs::remove_dir_all(root).unwrap();
}
