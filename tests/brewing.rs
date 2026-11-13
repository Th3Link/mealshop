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
    assert_eq!(catalogue["templates"].as_array().unwrap().len(), 16);
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
    recipe_for(app, "biab").await
}
async fn recipe_for(app: &axum::Router, template_id: &str) -> Value {
    let c = ok(app, "GET", "/api/household/brewing/templates", Value::Null).await;
    let t = c["templates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == template_id)
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
    let mut recipe = json!({"beverage":"beer","title":"Brewing acceptance recipe","language":"en","volume":{"amount":"20.000001","unit":"l"},"template":{"id":"biab","version":1,"sources":sources},"method":{"strategy":"all_grain","layout":"bag","heat":"manual","sparge":"none","mash":"infusion"},"ingredients":[{"id":"line-1","name":"Synthetic oat ingredient","ingredient_id":"oats","quantity":{"amount":"100.000001","unit":"g"},"phase_id":"p2","note":"Synthetic test, not a suggested beer recipe"}],"phases":phases,"notes":"Synthetic process acceptance fixture"});
    recipe["beverage"] = t["beverage"].clone();
    recipe["template"]["id"] = t["id"].clone();
    if t["beverage"] != "beer" {
        recipe["method"] = json!({"process":t["process"]});
    }
    recipe
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

fn utc(seconds: i64) -> Value {
    json!({"local":jiff::Timestamp::from_second(seconds).unwrap().to_zoned(jiff::tz::TimeZone::UTC).datetime().to_string(),"timezone":"UTC","fold":"reject"})
}
async fn save(app: &axum::Router, recipe: Value) -> String {
    apply(app,json!({"operation":"save_recipe","id":null,"expected_recipe_revision":null,"recipe":recipe})).await["brewing"]["recipes"][0]["record"]["id"].as_str().unwrap().to_owned()
}
async fn kettle(app: &axum::Router, name: &str) -> String {
    let result = apply(app,json!({"operation":"save_equipment","id":null,"equipment":{"name":name,"kind":"kettle","model":"Synthetic test vessel","capacity":{"amount":"30.000001","unit":"l"},"notes":"Isolated fixture"}})).await;
    result["brewing"]["equipment"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned()
}
fn batch(recipe_id: &str, at: &Value, equipment: &Value) -> Value {
    json!({"operation":"create_batch","draft":{"name":"Independent synthetic batch","recipe_id":recipe_id,"recipe_revision":1,"brew_day":at,"yeast_preparation":{"choice":"skip"},"equipment":equipment}})
}
#[tokio::test]
async fn batches_reserve_exact_equipment_atomically_and_keep_calendrical_recipe_reminders() {
    let directory = Directory::new();
    let app = directory.app().unwrap();
    let mut r = recipe(&app).await;
    r["yeast_preparation"] = json!({"days_before":1,"local_time":"08:00","fold":"reject","instruction":"Synthetic product-specific yeast reminder"});
    r["phases"][0]["timing_known"] = json!(true);
    for field in [
        "passive_seconds",
        "passive_min_seconds",
        "passive_max_seconds",
    ] {
        r["phases"][0]["stage"][field] = json!(3_456_000);
    }
    r["phases"][0]["volume"] = json!({"amount":"20000.000001","unit":"ml"});
    let id = save(&app, r).await;
    let e = kettle(&app, "Primary synthetic vessel").await;
    let other = kettle(&app, "Independent synthetic vessel").await;
    let day = json!({"local":"2027-03-28T08:00","timezone":"Europe/Berlin","fold":"reject"});
    let mut command = batch(&id, &day, &json!({"p1":[e]}));
    command["draft"]["yeast_preparation"] = json!({"choice":"recipe"});
    let request = input(&app, command.clone()).await;
    let preview = ok(&app, "POST", &format!("{ROOT}/preview"), request.clone()).await;
    assert_eq!(preview["after"]["batches"].as_array().unwrap().len(), 1);
    assert_eq!(
        ok(&app, "GET", ROOT, Value::Null).await["batches"],
        json!([])
    );
    let created = ok(&app, "POST", ROOT, request.clone()).await;
    let run = &created["brewing"]["batches"][0]["run"];
    assert!(run.get("servings").is_none() && run.get("recipe").is_none());
    let schedule = run["schedule"].as_array().unwrap();
    let prep = schedule
        .iter()
        .find(|s| s["id"] == "yeast-preparation-1")
        .unwrap()["start"]
        .as_i64()
        .unwrap();
    let start = schedule.iter().find(|s| s["id"] == "p1-1").unwrap()["start"]
        .as_i64()
        .unwrap();
    assert_eq!(start - prep, 23 * 3600);
    assert_eq!(run["brew_recipe"]["volume"]["amount"], "20.000001");
    let collision = input(&app, command.clone()).await;
    assert_eq!(
        call(&app, Some(OWNER), "POST", ROOT, collision).await.1["code"],
        "brewing_equipment_overlap"
    );
    command["draft"]["equipment"]["p1"] = json!([other]);
    let second = apply(&app, command).await;
    assert_eq!(second["brewing"]["batches"].as_array().unwrap().len(), 2);
    let run_id = run["id"].as_str().unwrap();
    assert!(
        !call(
            &app,
            Some(OWNER),
            "GET",
            &format!("/api/household/cooking/{run_id}"),
            Value::Null
        )
        .await
        .0
        .is_success()
    );
    drop(app);
    let reopened = directory.app().unwrap();
    let replay = ok(&reopened, "POST", ROOT, request).await;
    assert_eq!(replay["operation_id"], created["operation_id"]);
    assert_eq!(replay["brewing"]["batches"].as_array().unwrap().len(), 2);
}
#[tokio::test]
async fn manual_phases_keep_physical_locks_until_explicit_completion_and_readings_append_corrections()
 {
    let directory = Directory::new();
    let app = directory.app().unwrap();
    let id = save(&app, recipe(&app).await).await;
    let e = kettle(&app, "Shared synthetic vessel").await;
    let at = jiff::Timestamp::now().as_second() - 60;
    let created = apply(&app, batch(&id, &utc(at), &json!({"p1":[e],"p2":[e]}))).await;
    let run_id = created["brewing"]["batches"][0]["run"]["id"]
        .as_str()
        .unwrap();
    let started = apply(
        &app,
        json!({"operation":"timer","run_id":run_id,"step_id":"p1-1","action":"start"}),
    )
    .await;
    assert_eq!(
        started["brewing"]["batches"][0]["run"]["steps"][0]["timer"]["state"],
        "running"
    );
    let blocked = input(
        &app,
        json!({"operation":"timer","run_id":run_id,"step_id":"p2-1","action":"start"}),
    )
    .await;
    assert_eq!(
        call(&app, Some(OWNER), "POST", ROOT, blocked).await.1["code"],
        "brewing_equipment_busy"
    );
    let mut later = batch(&id, &utc(at + 3600), &json!({"p1":[e]}));
    // A held vessel remains unavailable, even when the new phase has no declared duration.
    later["draft"]["name"] = json!("Second manual batch");
    let blocked = input(&app, later).await;
    assert_eq!(
        call(&app, Some(OWNER), "POST", ROOT, blocked).await.1["code"],
        "brewing_equipment_overlap"
    );
    let reading = json!({"phase_id":"p1-1","measurement":{"value":"1.050000","unit":"sg"},"measured_at":utc(at),"source":{"kind":"instrument","device_id":"synthetic-hydrometer","name":"Synthetic hydrometer","acquisition":"transcribed"},"calibration":{"kind":"declared","details":"Original calibration declaration","calibrated_at":utc(at-3600),"reference_temperature":{"value":"20.000000","unit":"celsius"},"source_url":"https://www.ispindel.de/docs/Calibration_en.html"},"notes":"Exact transcription; no automatic correction"});
    let request = input(&app,json!({"operation":"add_reading","run_id":run_id,"reading":reading,"corrects":null,"reason":null})).await;
    ok(&app, "POST", ROOT, request.clone()).await;
    let detail_path = format!("{ROOT}/batches/{run_id}");
    let detail = ok(&app, "GET", &detail_path, Value::Null).await;
    let original = detail["measurements"]["readings"][0]["id"].clone();
    assert_eq!(detail["measurements"]["readings"][0]["input"], reading);
    let mut corrected = reading;
    corrected["measurement"]["value"] = json!("1.050001");
    apply(&app,json!({"operation":"add_reading","run_id":run_id,"reading":corrected,"corrects":original,"reason":"Corrected synthetic transcription"})).await;
    apply(
        &app,
        json!({"operation":"timer","run_id":run_id,"step_id":"p1-1","action":"complete"}),
    )
    .await;
    let next = apply(
        &app,
        json!({"operation":"timer","run_id":run_id,"step_id":"p2-1","action":"start"}),
    )
    .await;
    assert_eq!(
        next["brewing"]["batches"][0]["run"]["steps"][1]["timer"]["state"],
        "running"
    );
    drop(app);
    let reopened = directory.app().unwrap();
    assert_eq!(ok(&reopened, "POST", ROOT, request).await["replayed"], true);
    let detail = ok(&reopened, "GET", &detail_path, Value::Null).await;
    assert_eq!(detail["measurements"]["total"], 2);
    assert_eq!(
        detail["measurements"]["readings"][1]["input"]["measurement"]["value"],
        "1.050000"
    );
    let reader = input(&reopened, json!({"operation":"cancel","run_id":run_id})).await;
    assert_eq!(
        call(&reopened, Some(READER), "POST", ROOT, reader).await.0,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn invalid_dimensions_mismatched_beverages_and_uncertain_publication_never_publish_partial_changes()
 {
    let directory = Directory::new();
    let app = directory.app().unwrap();
    let mut r = recipe(&app).await;
    r["beverage"] = json!("cider");
    let invalid = input(
        &app,
        json!({"operation":"save_recipe","id":null,"expected_recipe_revision":null,"recipe":r}),
    )
    .await;
    assert_eq!(
        call(&app, Some(OWNER), "POST", ROOT, invalid).await.1["code"],
        "brewing_method_invalid"
    );
    let mut r = recipe(&app).await;
    r["phases"][0]["volume"] = json!({"amount":"30.000002","unit":"l"});
    let id = save(&app, r).await;
    let e = kettle(&app, "Capacity acceptance vessel").await;
    let invalid = input(
        &app,
        batch(
            &id,
            &utc(jiff::Timestamp::now().as_second() + 3600),
            &json!({"p1":[e]}),
        ),
    )
    .await;
    assert_eq!(
        call(&app, Some(OWNER), "POST", ROOT, invalid).await.1["code"],
        "brewing_equipment_capacity"
    );
    assert_eq!(
        ok(&app, "GET", ROOT, Value::Null).await["batches"],
        json!([])
    );
    let snapshot = std::fs::read(directory.0.join("household/cooking.json")).unwrap();
    std::fs::write(
        directory.0.join("household/.cooking.pending"),
        b"synthetic interrupted write",
    )
    .unwrap();
    let request = input(
        &app,
        json!({"operation":"archive_recipe","id":id,"archived":true}),
    )
    .await;
    assert_eq!(
        call(&app, Some(OWNER), "POST", ROOT, request.clone())
            .await
            .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        call(&app, Some(OWNER), "GET", ROOT, Value::Null).await.0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        call(&app, Some(OWNER), "POST", ROOT, request).await.0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        std::fs::read(directory.0.join("household/cooking.json")).unwrap(),
        snapshot
    );
    drop(app);
    assert!(directory.app().is_err());
    std::fs::remove_file(directory.0.join("household/.cooking.pending")).unwrap();
    let app = directory.app().unwrap();
    assert_eq!(
        ok(&app, "GET", ROOT, Value::Null).await["recipes"][0]["archived"],
        false
    );
}

#[tokio::test]
async fn equipment_rebase_cannot_overwrite_a_concurrent_edit_and_retains_frozen_batch_details() {
    let directory = Directory::new();
    let app = directory.app().unwrap();
    let id = kettle(&app, "Original synthetic equipment").await;
    let state = ok(&app, "GET", ROOT, Value::Null).await;
    let original = state["equipment"][0]["details"].clone();
    let recipe_id = save(&app, recipe(&app).await).await;
    apply(
        &app,
        batch(
            &recipe_id,
            &utc(jiff::Timestamp::now().as_second() + 3600),
            &json!({"p1":[id]}),
        ),
    )
    .await;
    let mut edited = original.clone();
    edited["name"] = json!("Edited synthetic equipment");
    let request=input(&app,json!({"operation":"save_equipment","id":id,"expected_equipment":original,"equipment":edited})).await;
    let applied = ok(&app, "POST", ROOT, request.clone()).await;
    let conflicting=input(&app,json!({"operation":"save_equipment","id":id,"expected_equipment":original,"equipment":original})).await;
    assert_eq!(
        call(&app, Some(OWNER), "POST", ROOT, conflicting).await.1["code"],
        "brewing_equipment_conflict"
    );
    drop(app);
    let app = directory.app().unwrap();
    assert_eq!(
        ok(&app, "POST", ROOT, request).await["operation_id"],
        applied["operation_id"]
    );
    let state = ok(&app, "GET", ROOT, Value::Null).await;
    assert_eq!(state["equipment"][0]["details"], edited);
    assert_eq!(
        state["batches"][0]["batch"]["equipment"][0]["details"],
        original
    );
}

#[tokio::test]
async fn restart_rejects_two_physical_holds_for_one_equipment_record() {
    let directory = Directory::new();
    let app = directory.app().unwrap();
    let recipe_id = save(&app, recipe(&app).await).await;
    let equipment_id = kettle(&app, "Synthetic restart hold vessel").await;
    let created = apply(
        &app,
        batch(
            &recipe_id,
            &utc(jiff::Timestamp::now().as_second() - 60),
            &json!({"p1":[equipment_id],"p2":[equipment_id]}),
        ),
    )
    .await;
    let run_id = created["brewing"]["batches"][0]["run"]["id"]
        .as_str()
        .unwrap();
    apply(
        &app,
        json!({"operation":"timer","run_id":run_id,"step_id":"p1-1","action":"start"}),
    )
    .await;
    drop(app);
    let path = directory.0.join("household/cooking.json");
    let mut stored: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    stored["runs"][0]["steps"][1]["timer"] = stored["runs"][0]["steps"][0]["timer"].clone();
    std::fs::write(path, serde_json::to_vec(&stored).unwrap()).unwrap();
    assert!(directory.app().is_err());
}

#[tokio::test]
async fn independently_validated_cider_wine_and_mead_survive_restart_without_food_or_beer_assumptions()
 {
    let directory = Directory::new();
    let app = directory.app().unwrap();
    let mut receipts = Vec::new();
    for (template, beverage) in [
        ("cider-juice", "cider"),
        ("white-wine", "wine"),
        ("red-wine", "wine"),
        ("traditional-mead", "mead"),
    ] {
        let draft = recipe_for(&app, template).await;
        assert!(draft["method"].get("strategy").is_none());
        let saved = apply(&app, json!({"operation":"save_recipe","id":null,"expected_recipe_revision":null,"recipe":draft})).await;
        let recipe_id = saved["brewing"]["recipes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["record"]["recipe"]["template"]["id"] == template)
            .unwrap()["record"]["id"]
            .clone();
        let request = input(&app, json!({"operation":"create_batch","draft":{"name":format!("Independent brewing {template}"),"recipe_id":recipe_id,"recipe_revision":1,"brew_day":{"local":"2030-01-15T08:00:00","timezone":"UTC","fold":"reject"},"yeast_preparation":{"choice":"skip"},"equipment":{}}})).await;
        let result = ok(&app, "POST", ROOT, request.clone()).await;
        let run = result["brewing"]["batches"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["run"]["brew_recipe"]["id"] == recipe_id)
            .unwrap()["run"]
            .clone();
        assert_eq!(run["kind"], beverage);
        assert_eq!(run["brew_recipe"]["beverage"], beverage);
        assert!(run.get("recipe").is_none() && run.get("servings").is_none());
        assert_eq!(run["brew_recipe"]["volume"]["amount"], "20.000001");
        let run_id = run["id"].as_str().unwrap();
        let before = ok(&app, "GET", ROOT, Value::Null).await;
        assert!(
            call(
                &app,
                Some(OWNER),
                "POST",
                "/api/household/cooking",
                json!({"expected_revision":before["revision"],"command":{"operation":"timer","run_id":run_id,"step_id":"p1-1","action":"start"}})
            )
            .await
            .0
            .is_client_error()
        );
        let reading = json!({"phase_id":null,"measurement":{"value":"12.345678","unit":"brix"},"measured_at":{"local":"2026-11-13T08:00:00","timezone":"UTC","fold":"reject"},"source":{"kind":"manual"},"calibration":{"kind":"unknown"},"notes":"Unconverted original reading"});
        apply(&app,json!({"operation":"add_reading","run_id":run_id,"reading":reading,"corrects":null,"reason":null})).await;
        let shown = ok(
            &app,
            "GET",
            &format!("{ROOT}/batches/{run_id}"),
            Value::Null,
        )
        .await;
        assert_eq!(shown["measurements"]["readings"][0]["input"], reading);
        assert_eq!(shown["recipe"]["recipe"], draft);
        let mut wrong = draft.clone();
        wrong["method"] = recipe(&app).await["method"].clone();
        let invalid = input(&app,json!({"operation":"save_recipe","id":null,"expected_recipe_revision":null,"recipe":wrong})).await;
        assert!(
            call(&app, Some(OWNER), "POST", ROOT, invalid)
                .await
                .0
                .is_client_error()
        );
        let mut wrong_template = draft.clone();
        wrong_template["template"]["id"] = json!("biab");
        let invalid = input(&app,json!({"operation":"save_recipe","id":null,"expected_recipe_revision":null,"recipe":wrong_template})).await;
        assert!(
            call(&app, Some(OWNER), "POST", ROOT, invalid)
                .await
                .0
                .is_client_error()
        );
        receipts.push((request, result["operation_id"].clone()));
    }
    drop(app);
    let app = directory.app().unwrap();
    assert_eq!(
        ok(&app, "GET", ROOT, Value::Null).await["batches"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    for (request, operation) in receipts {
        let result = ok(&app, "POST", ROOT, request.clone()).await;
        assert_eq!(result["replayed"], true);
        assert_eq!(result["operation_id"], operation);
        assert_eq!(
            call(&app, Some(READER), "POST", ROOT, request).await.0,
            StatusCode::FORBIDDEN
        );
    }
}

#[tokio::test]
async fn fermentation_reservations_block_beer_and_manual_completion_releases_the_same_physical_resource()
 {
    let directory = Directory::new();
    let app = directory.app().unwrap();
    let mut cider = recipe_for(&app, "cider-juice").await;
    cider["phases"] = json!([cider["phases"][0]]);
    cider["ingredients"] = json!([]);
    let cider_id = save(&app, cider).await;
    let mut beer = recipe(&app).await;
    beer["phases"] = json!([beer["phases"][0]]);
    beer["ingredients"] = json!([]);
    beer["phases"][0]["timing_known"] = json!(true);
    for field in [
        "passive_seconds",
        "passive_min_seconds",
        "passive_max_seconds",
    ] {
        beer["phases"][0]["stage"][field] = json!(1);
    }
    let beer_id = save(&app, beer).await;
    let equipment = kettle(&app, "Shared physical vessel").await;
    let day = utc(jiff::Timestamp::now().as_second() - 60);
    let created = apply(&app, batch(&cider_id, &day, &json!({"p1":[equipment]}))).await;
    let run = created["brewing"]["batches"][0]["run"]["id"]
        .as_str()
        .unwrap();
    apply(
        &app,
        json!({"operation":"timer","run_id":run,"step_id":"p1-1","action":"start"}),
    )
    .await;
    let started = ok(&app, "GET", &format!("{ROOT}/batches/{run}"), Value::Null).await;
    assert_eq!(started["run"]["steps"][0]["timer"]["state"], "running");
    let beer_day = utc(jiff::Timestamp::now().as_second() + 3600);
    let blocked = input(&app, batch(&beer_id, &beer_day, &json!({"p1":[equipment]}))).await;
    assert_eq!(
        call(&app, Some(OWNER), "POST", ROOT, blocked).await.1["code"],
        "brewing_equipment_overlap"
    );
    apply(
        &app,
        json!({"operation":"timer","run_id":run,"step_id":"p1-1","action":"complete"}),
    )
    .await;
    assert_eq!(
        apply(&app, batch(&beer_id, &beer_day, &json!({"p1":[equipment]}))).await["brewing"]["batches"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}
