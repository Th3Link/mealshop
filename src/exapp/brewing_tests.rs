#![allow(clippy::unwrap_used)]
use super::{
    Config,
    library_tests::join,
    router,
    tests::{SECRET, call},
};
use axum::{Json, Router, body::Bytes, extract::Request, http::StatusCode};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
struct Fixture {
    root: PathBuf,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.task.abort();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
async fn setup() -> (Fixture, Router, Arc<Mutex<Vec<Value>>>) {
    let root =
        std::env::temp_dir().join(format!("mealshop-brewing-delivery-{}", std::process::id()));
    std::fs::create_dir_all(root.join("assets")).unwrap();
    for file in ["mealshop.js", "mealshop.css"] {
        std::fs::write(root.join("assets").join(file), "Synthetic test asset").unwrap();
    }
    let seen = Arc::new(Mutex::new(vec![]));
    let captured = seen.clone();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new().fallback(axum::routing::any(move |request: Request| {
                let captured = captured.clone();
                async move {
                    let path = request.uri().path().to_owned();
                    let data = if path.ends_with("/notification") {
                        let bytes: Bytes = axum::body::to_bytes(request.into_body(), 65536)
                            .await
                            .unwrap();
                        captured
                            .lock()
                            .unwrap()
                            .push(serde_json::from_slice::<Value>(&bytes).unwrap());
                        json!(true)
                    } else if path.ends_with("/users") {
                        json!(["alice", "bob"])
                    } else {
                        json!(false)
                    };
                    Json(json!({"ocs":{"meta":{"statuscode":100},"data":data}}))
                }
            })),
        )
        .await
        .unwrap();
    });
    let app = router(Config {
        secret: SECRET.to_owned(),
        nextcloud_url: url,
        public_url: Some("https://cloud.example.test".into()),
        directory: root.join("data"),
        assets: root.join("assets"),
    })
    .await
    .unwrap();
    assert_eq!(
        call(&app, Some(""), "PUT", "/enabled?enabled=1", json!({}))
            .await
            .0,
        StatusCode::OK
    );
    (Fixture { root, task }, app, seen)
}
async fn change(app: &Router, root: &str, command: Value, key: &str) -> Value {
    let (_, state) = call(app, Some("alice"), "GET", root, json!({})).await;
    let (status,result)=call(app,Some("alice"),"POST",root,json!({"expected_revision":state["revision"],"idempotency_key":key,"reviewed":true,"command":command})).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    result
}
#[tokio::test]
async fn brewing_uses_one_durable_worker_with_private_personal_notices_and_native_batch_links() {
    let (_fixture, app, seen) = setup().await;
    let (_, home) = call(
        &app,
        Some("alice"),
        "POST",
        "/api/households",
        json!({"name":"Synthetic brewing delivery household"}),
    )
    .await;
    let id = home["id"].as_str().unwrap();
    join(
        &app,
        &format!("/api/households/{id}"),
        "alice",
        "bob",
        "reader",
    )
    .await;
    let root = format!("/api/households/{id}/domain/api/household/brewing");
    let (_, catalogue) = call(
        &app,
        Some("alice"),
        "GET",
        &format!("{root}/templates"),
        json!({}),
    )
    .await;
    let template = &catalogue["templates"][0];
    let sources = catalogue["sources"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| template["sources"].as_array().unwrap().contains(&s["id"]))
        .collect::<Vec<_>>();
    let recipe = json!({"beverage":"beer","title":"Synthetic worker recipe","language":"en","volume":{"amount":"1.000001","unit":"l"},"template":{"id":template["id"],"version":template["version"],"sources":sources},"method":{"strategy":"extract","layout":"kettle","heat":"direct","sparge":"none","mash":"none"},"ingredients":[],"phases":[{"kind":"water","timing_known":true,"temperature":null,"volume":null,"stage":{"id":"water","origin":{"kind":"custom","note":"Synthetic delivery check"},"name":"Synthetic brewing action","instruction":"Synthetic action, no production instruction","dependencies":[],"active_seconds":0,"passive_seconds":1,"passive_min_seconds":1,"passive_max_seconds":1,"equipment":[],"lead_seconds":0,"repeat":1,"wait_between_seconds":0,"uncertainty":"Synthetic only"}}],"notes":"Synthetic notification check"});
    let saved=change(&app,&root,json!({"operation":"save_recipe","id":null,"expected_recipe_revision":null,"recipe":recipe}),"synthetic-worker-recipe-0001").await;
    let recipe_id = &saved["brewing"]["recipes"][0]["record"]["id"];
    let local = jiff::Timestamp::from_second(jiff::Timestamp::now().as_second() + 2)
        .unwrap()
        .to_zoned(jiff::tz::TimeZone::UTC)
        .datetime()
        .to_string();
    let saved=change(&app,&root,json!({"operation":"create_batch","draft":{"name":"Synthetic worker batch","recipe_id":recipe_id,"recipe_revision":1,"brew_day":{"local":local,"timezone":"UTC","fold":"reject"},"yeast_preparation":{"choice":"skip"},"equipment":{}}}),"synthetic-worker-batch-0001").await;
    let run_id = saved["brewing"]["batches"][0]["run"]["id"]
        .as_str()
        .unwrap();
    change(&app,&root,json!({"operation":"subscribe","run_id":run_id,"enabled":true,"language":"en","permission":"denied"}),"synthetic-worker-opt-in-0001").await;
    for _ in 0..60 {
        if !seen.lock().unwrap().is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    let messages = seen.lock().unwrap().clone();
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0]["params"]["subject_params"]["link"],
        format!(
            "https://cloud.example.test/index.php/apps/app_api/embedded/mealshop/main/h/{id}/brewing/batches/{run_id}"
        )
    );
    let (_, detail) = call(
        &app,
        Some("alice"),
        "GET",
        &format!("{root}/batches/{run_id}"),
        json!({}),
    )
    .await;
    assert_eq!(detail["recipe"]["actor"], "alice");
    assert_eq!(detail["notices"][0]["delivery"]["state"], "accepted");
    assert!(detail["notices"][0].get("user").is_none());
    assert!(detail["run"].get("subscriptions").is_none());
    let (_, reader) = call(
        &app,
        Some("bob"),
        "GET",
        &format!("{root}/batches/{run_id}"),
        json!({}),
    )
    .await;
    assert_eq!(reader["notices"], json!([]));
    assert!(reader["run"]["subscription"].is_null());
    assert_eq!(
        call(&app, Some("bob"), "POST", &root, json!({})).await.0,
        StatusCode::FORBIDDEN
    );
}
