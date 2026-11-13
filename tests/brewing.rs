//! F14 uses only isolated synthetic households; no retained personal records are fixtures.
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
const OWNER: &str = "brewing_owner_synthetic_credential_123456789";
const READER: &str = "brewing_reader_synthetic_credential_12345678";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "mealshop-brewing-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn app(&self) -> Result<axum::Router, String> {
        mealshop_server::family_router(
            include_str!("../data/catalogue.json"),
            self.0.join("recipes"),
            self.0.join("household"),
            OWNER.to_owned(),
            Some(READER.to_owned()),
        )
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
    value: Value,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json");
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(value.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), 8_388_608).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}
#[tokio::test]
async fn protected_method_catalogue_keeps_partial_mash_biab_and_named_systems_distinct() {
    let directory = Directory::new();
    let app = directory.app().unwrap();
    let path = "/api/household/brewing/templates";
    assert_eq!(
        call(&app, None, "GET", path, Value::Null).await.0,
        StatusCode::UNAUTHORIZED
    );
    let (status, catalogue) = call(&app, Some(READER), "GET", path, Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(catalogue["templates"].as_array().unwrap().len(), 12);
    let template = |id| {
        catalogue["templates"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["id"] == id)
            .unwrap()
    };
    assert_eq!(template(json!("biab"))["strategy"], "all_grain");
    assert_eq!(template(json!("partial-mash"))["strategy"], "partial_mash");
    assert_ne!(
        template(json!("grainfather"))["layout"],
        template(json!("braumeister"))["layout"]
    );
    assert!(
        catalogue["sources"]
            .as_array()
            .unwrap()
            .iter()
            .all(|s| s["url"].as_str().unwrap().starts_with("https://"))
    );
}

const ROOT: &str = "/api/household/brewing";
async fn ok(app: &axum::Router, method: &str, path: &str, value: Value) -> Value {
    let (status, body) = call(app, Some(OWNER), method, path, value).await;
    assert!(status.is_success(), "{path}: {status} {body}");
    body
}
async fn input(app: &axum::Router, command: Value) -> Value {
    let current = ok(app, "GET", ROOT, Value::Null).await;
    json!({"expected_revision":current["revision"],"idempotency_key":format!("synthetic-brew-request-{:020}",NEXT.fetch_add(1,Ordering::Relaxed)),"command":command,"reviewed":true})
}
async fn apply(app: &axum::Router, command: Value) -> Value {
    let request = input(app, command).await;
    ok(app, "POST", ROOT, request).await
}
async fn recipe(app: &axum::Router) -> Value {
    let c = ok(app, "GET", "/api/household/brewing/templates", Value::Null).await;
    let t = c["templates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == "biab")
        .unwrap();
    let sources = c["sources"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| t["sources"].as_array().unwrap().contains(&s["id"]))
        .cloned()
        .collect::<Vec<_>>();
    let phases = t["phases"].as_array().unwrap().iter().enumerate().map(|(i,key)| {
        let p=&c["phases"][key.as_str().unwrap()];
        json!({"kind":p["kind"],"timing_known":false,"temperature":null,"volume":null,"stage":{"start_after_work":false,"id":format!("p{}",i+1),"origin":{"kind":"custom","note":"Synthetic reviewed brew phase"},"name":p["name"],"instruction":p["instruction"],"dependencies":if i==0{vec![]}else{vec![format!("p{i}")]},"active_seconds":0,"passive_seconds":0,"passive_min_seconds":0,"passive_max_seconds":0,"equipment":[],"lead_seconds":0,"repeat":1,"wait_between_seconds":0,"uncertainty":"Enter actual recipe timing"}})
    }).collect::<Vec<_>>();
    json!({"beverage":"beer","title":"Brewing acceptance recipe","language":"en","volume":{"amount":"20.000001","unit":"l"},"template":{"id":"biab","version":1,"sources":sources},"method":{"strategy":"all_grain","layout":"bag","heat":"manual","sparge":"none","mash":"infusion"},"ingredients":[{"id":"line-1","name":"Synthetic oat ingredient","ingredient_id":"oats","quantity":{"amount":"100.000001","unit":"g"},"phase_id":"p2","note":"Synthetic test, not a suggested beer recipe"}],"phases":phases,"notes":"Synthetic process acceptance fixture"})
}
#[tokio::test]
async fn reviewed_recipe_versions_retain_exact_facts_and_retry_after_restart_without_duplication() {
    let directory = Directory::new();
    let app = directory.app().unwrap();
    let draft = recipe(&app).await;
    let request = input(
        &app,
        json!({"operation":"save_recipe","id":null,"expected_recipe_revision":null,"recipe":draft}),
    )
    .await;
    let mut preview = request.clone();
    preview["reviewed"] = json!(false);
    let shown = ok(&app, "POST", &format!("{ROOT}/preview"), preview.clone()).await;
    assert_eq!(shown["after"]["recipes"].as_array().unwrap().len(), 1);
    assert_eq!(
        ok(&app, "GET", ROOT, Value::Null).await["recipes"],
        json!([])
    );
    assert!(!directory.0.join("household/cooking.json").exists());
    assert_eq!(
        call(&app, Some(OWNER), "POST", ROOT, preview).await.0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(&app, Some(READER), "POST", ROOT, request.clone())
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let created = ok(&app, "POST", ROOT, request.clone()).await;
    let id = created["brewing"]["recipes"][0]["record"]["id"]
        .as_str()
        .unwrap();
    let mut revised = draft.clone();
    revised["title"] = json!("Revised brewing acceptance recipe");
    let updated = apply(
        &app,
        json!({"operation":"save_recipe","id":id,"expected_recipe_revision":1,"recipe":revised}),
    )
    .await;
    assert_eq!(updated["brewing"]["recipes"][0]["record"]["revision"], 2);
    let old = ok(
        &app,
        "GET",
        &format!("{ROOT}/recipes/{id}?revision=1"),
        Value::Null,
    )
    .await;
    assert_eq!(old["recipe"], draft);
    drop(app);
    let reopened = directory.app().unwrap();
    let replay = ok(&reopened, "POST", ROOT, request.clone()).await;
    assert_eq!(replay["operation_id"], created["operation_id"]);
    assert_eq!(replay["replayed"], true);
    assert_eq!(replay["brewing"]["recipes"][0]["record"]["revision"], 2);
    let mut changed = request;
    changed["command"]["recipe"]["title"] = json!("Changed key reuse");
    assert_eq!(
        call(&reopened, Some(OWNER), "POST", ROOT, changed).await.0,
        StatusCode::CONFLICT
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(directory.0.join("household/cooking.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
#[tokio::test]
async fn exact_equipment_capacity_and_recipe_sources_reject_invalid_dimensions_and_fabricated_evidence()
 {
    let directory = Directory::new();
    let app = directory.app().unwrap();
    let equipment = json!({"name":"Synthetic kettle","kind":"kettle","model":"Declared test model","capacity":{"amount":"25.000001","unit":"l"},"notes":"Synthetic equipment only"});
    let created = apply(
        &app,
        json!({"operation":"save_equipment","id":null,"equipment":equipment}),
    )
    .await;
    assert_eq!(
        created["brewing"]["equipment"][0]["details"]["capacity"]["amount"],
        "25.000001"
    );
    let id = created["brewing"]["equipment"][0]["id"].clone();
    let archived = apply(
        &app,
        json!({"operation":"archive_equipment","id":id,"archived":true}),
    )
    .await;
    assert_eq!(archived["brewing"]["equipment"][0]["archived"], true);
    let mut bad = equipment.clone();
    bad["capacity"]["unit"] = json!("g");
    let invalid = input(
        &app,
        json!({"operation":"save_equipment","id":null,"equipment":bad}),
    )
    .await;
    assert_eq!(
        call(&app, Some(OWNER), "POST", ROOT, invalid).await.1["code"],
        "brewing_equipment_invalid"
    );
    let mut draft = recipe(&app).await;
    draft["template"]["sources"][0]["url"] = json!("https://example.org/forged-source");
    let invalid = input(
        &app,
        json!({"operation":"save_recipe","id":null,"expected_recipe_revision":null,"recipe":draft}),
    )
    .await;
    assert_eq!(
        call(&app, Some(OWNER), "POST", ROOT, invalid).await.1["code"],
        "brewing_source_invalid"
    );
    assert_eq!(ok(&app, "GET", ROOT, Value::Null).await["revision"], 2);
}
