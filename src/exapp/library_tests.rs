use super::tests::{SECRET, call};
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub(super) struct Fixture {
    pub(super) root: PathBuf,
    url: String,
    server: tokio::task::JoinHandle<()>,
    directory_available: Arc<AtomicBool>,
}
impl Fixture {
    pub(super) fn stop_directory(&self) {
        self.directory_available.store(false, Ordering::Relaxed);
    }
    pub(super) async fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "mealshop-library-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(root.join("assets")).unwrap();
        for file in ["mealshop.js", "mealshop.css"] {
            std::fs::write(root.join("assets").join(file), "test").unwrap();
        }
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let directory_available = Arc::new(AtomicBool::new(true));
        let availability = directory_available.clone();
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                Router::new().route("/ocs/v1.php/apps/app_api/api/v1/users",axum::routing::get(move || {
                    let available = availability.clone();
                    async move {
                        if !available.load(Ordering::Relaxed) {
                            return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({})));
                        }
                        (StatusCode::OK, Json(json!({"ocs":{"meta":{"statuscode":100},"data":["alice","bob","carol","eve","mallory"]}})))
                    }
                })).fallback(axum::routing::any(|| async {
                    Json(json!({"ocs":{"meta":{"statuscode":100},"data":false}}))
                })),
            )
            .await
            .unwrap();
        });
        Self {
            root,
            url,
            server,
            directory_available,
        }
    }
    pub(super) async fn app(&self) -> Router {
        let app = router(Config {
            secret: SECRET.to_owned(),
            nextcloud_url: self.url.clone(),
            public_url: None,
            directory: self.root.join("data"),
            assets: self.root.join("assets"),
        })
        .await
        .unwrap();
        assert_eq!(
            call(&app, Some(""), "PUT", "/enabled?enabled=1", json!({}))
                .await
                .0,
            StatusCode::OK
        );
        app
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
pub(super) fn recipe() -> Value {
    json!({"title":"Original collection recipe","language":"en","servings":2,"preparation":"as_recorded","lines":[{"ingredient_id":"oats","grams":"100.500000"}],"steps":[]})
}
pub(super) async fn join(app: &Router, base: &str, owner: &str, actor: &str, role: &str) {
    assert_eq!(
        call(
            app,
            Some(owner),
            "POST",
            &format!("{base}/invitations"),
            json!({"user":actor,"role":role})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            app,
            Some(actor),
            "PUT",
            &format!("{base}/invitation"),
            json!({"accept":true})
        )
        .await
        .0,
        StatusCode::OK
    );
}
pub(super) async fn home(app: &Router, actor: &str) -> String {
    let (status, value) = call(
        app,
        Some(actor),
        "POST",
        "/api/households",
        json!({"name":"Synthetic library household"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    value["id"].as_str().unwrap().to_owned()
}
#[tokio::test]
async fn collections_pin_revisions_and_survive_restart() {
    let fixture = Fixture::new().await;
    let app = fixture.app().await;
    let home = home(&app, "alice").await;
    let base = format!("/api/households/{home}");
    let path = format!("{base}/collections");
    let recipes = format!("{base}/domain/api/recipes");
    let (status, saved) = call(&app, Some("alice"), "POST", &recipes, recipe()).await;
    assert_eq!(status, StatusCode::CREATED);
    let draft = json!({"name":"Weeknight recipes","description":"Synthetic test collection","entries":[{"recipe_id":saved["id"],"revision":1}]});
    let body = json!({"expected_revision":0,"collection":draft});
    assert_eq!(
        call(&app, Some("mallory"), "POST", &path, body.clone())
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let (status, created) = call(&app, Some("alice"), "POST", &path, body.clone()).await;
    assert_eq!(status, StatusCode::OK);
    let detail = format!("{path}/{}", created["id"].as_str().unwrap());
    assert_eq!(
        call(&app, Some("alice"), "POST", &path, body).await.0,
        StatusCode::CONFLICT
    );
    let mut revised = recipe();
    revised["title"] = json!("Changed source recipe");
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "PUT",
            &format!("{recipes}/{}", saved["id"].as_str().unwrap()),
            json!({"expected_revision":1,"recipe":revised})
        )
        .await
        .0,
        StatusCode::OK
    );
    let pinned = call(&app, Some("alice"), "GET", &detail, json!({})).await.1;
    assert_eq!(pinned["items"][0]["title"], saved["recipe"]["title"]);
    assert_eq!(pinned["items"][0]["revision"], 1);
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "GET",
            &format!("{path}?page=0"),
            json!({})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    drop(app);
    let reopened = fixture.app().await;
    assert_eq!(
        call(&reopened, Some("alice"), "GET", &detail, json!({}))
            .await
            .1,
        pinned
    );
    assert_eq!(
        call(
            &reopened,
            Some("alice"),
            "DELETE",
            &detail,
            json!({"expected_revision":1})
        )
        .await
        .0,
        StatusCode::OK
    );
    let next = call(
        &reopened,
        Some("alice"),
        "POST",
        &path,
        json!({"expected_revision":2,"collection":{"name":"Empty collection","entries":[]}}),
    )
    .await
    .1;
    assert_ne!(next["id"], created["id"]);
}
#[tokio::test]
async fn collection_permissions_follow_current_household_roles() {
    let fixture = Fixture::new().await;
    let app = fixture.app().await;
    let home = home(&app, "alice").await;
    let base = format!("/api/households/{home}");
    let path = format!("{base}/collections");
    let draft = json!({"name":"Synthetic collection","entries":[]});
    let created = call(
        &app,
        Some("alice"),
        "POST",
        &path,
        json!({"expected_revision":0,"collection":draft}),
    )
    .await
    .1;
    let detail = format!("{path}/{}", created["id"].as_str().unwrap());
    join(&app, &base, "alice", "bob", "reader").await;
    assert_eq!(
        call(&app, Some("bob"), "GET", &detail, json!({})).await.0,
        StatusCode::OK
    );
    for (method, target) in [("POST", &path), ("PUT", &detail), ("DELETE", &detail)] {
        let body = if method == "DELETE" {
            json!({"expected_revision":1})
        } else {
            json!({"expected_revision":1,"collection":draft})
        };
        assert_eq!(
            call(&app, Some("bob"), method, target, body).await.0,
            StatusCode::FORBIDDEN
        );
    }
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "PUT",
            &format!("{base}/members/bob"),
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
            &detail,
            json!({"expected_revision":1,"collection":draft})
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
            &format!("{base}/members/bob"),
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, Some("bob"), "GET", &detail, json!({})).await.0,
        StatusCode::FORBIDDEN
    );
}
#[tokio::test]
async fn collections_reject_foreign_missing_duplicate_and_stale_references_without_writes() {
    let fixture = Fixture::new().await;
    let app = fixture.app().await;
    let alice = home(&app, "alice").await;
    let bob = home(&app, "bob").await;
    let recipes = format!("/api/households/{bob}/domain/api/recipes");
    let saved = call(&app, Some("bob"), "POST", &recipes, recipe()).await.1;
    let path = format!("/api/households/{alice}/collections");
    for entries in [
        json!([{"recipe_id":saved["id"],"revision":1}]),
        json!([{"recipe_id":"../recipe-00000001","revision":1}]),
        json!([{"recipe_id":"recipe-00000001","revision":0}]),
        json!([{"recipe_id":"recipe-00000001","revision":1},{"recipe_id":"recipe-00000001","revision":2}]),
    ] {
        assert_eq!(call(&app,Some("alice"),"POST",&path,json!({"expected_revision":0,"collection":{"name":"Invalid collection","entries":entries}})).await.0,StatusCode::UNPROCESSABLE_ENTITY);
    }
    let list = call(&app, Some("alice"), "GET", &path, json!({})).await.1;
    assert_eq!(list["revision"], 0);
    assert_eq!(list["collections"], json!([]));
    assert!(!fixture.root.join("data/library.json").exists());
}

#[tokio::test]
async fn ambiguous_library_writes_fail_closed_until_restart() {
    let fixture = Fixture::new().await;
    let app = fixture.app().await;
    let home = home(&app, "alice").await;
    let path = format!("/api/households/{home}/collections");
    let draft = json!({"name":"Recovery fixture collection","entries":[]});
    let created = call(
        &app,
        Some("alice"),
        "POST",
        &path,
        json!({"expected_revision":0,"collection":draft}),
    )
    .await
    .1;
    let file = fixture.root.join("data/library.json");
    let original = std::fs::read(&file).unwrap();
    let marker = fixture.root.join("data/.library.pending");
    std::fs::write(&marker, b"synthetic interrupted write").unwrap();
    let detail = format!("{path}/{}", created["id"].as_str().unwrap());
    let edit = json!({"expected_revision":1,"collection":draft});
    assert_eq!(
        call(&app, Some("alice"), "PUT", &detail, edit.clone())
            .await
            .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    std::fs::remove_file(marker).unwrap();
    assert_eq!(
        call(&app, Some("alice"), "PUT", &detail, edit).await.0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(std::fs::read(&file).unwrap(), original);
    drop(app);
    let reopened = fixture.app().await;
    assert_eq!(
        call(&reopened, Some("alice"), "GET", &detail, json!({}))
            .await
            .0,
        StatusCode::OK
    );
}
