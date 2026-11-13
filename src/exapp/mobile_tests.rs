#![allow(clippy::unwrap_used)]
use super::{
    library_tests::{Fixture, home, join},
    tests::call,
};
use axum::http::StatusCode;
use serde_json::json;

fn intent(revision: u64, operation: &str, command: serde_json::Value) -> serde_json::Value {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut request =
        json!({"expected_revision":revision,"operation_id":operation,"issued_at":now});
    request["command"] = command;
    request
}
#[tokio::test]
async fn mobile_groups_project_only_used_ingredient_labels_without_changing_legacy_view() {
    let fixture = Fixture::new().await;
    let app = fixture.app().await;
    let home = home(&app, "alice").await;
    let legacy = format!("/api/households/{home}/domain/api/household/shopping");
    let mobile = format!("{legacy}/mobile");
    let applied = call(&app, Some("alice"), "POST", &mobile, intent(0, "intent-groups-00001", json!({"operation":"add_manual","label":"Flour","ingredient_id":"wheat-flour-type-550","quantity":{"amount":"30","unit":"g"}}))).await;
    assert_eq!(applied.0, StatusCode::OK);
    assert_eq!(applied.1["ingredient_groups"].as_object().unwrap().len(), 1);
    assert_eq!(
        applied.1["ingredient_groups"]["wheat-flour-type-550"]["id"],
        "custom-group-wheat-flour"
    );
    assert_eq!(
        applied.1["ingredient_groups"]["wheat-flour-type-550"]["translations"]["de"],
        "Weizenmehl"
    );
    let read = call(&app, Some("alice"), "GET", &mobile, json!({})).await;
    assert_eq!(read.1["ingredient_groups"], applied.1["ingredient_groups"]);
    let ordinary = call(&app, Some("alice"), "GET", &legacy, json!({})).await;
    assert_eq!(ordinary.1, applied.1["shopping"]);
    assert!(ordinary.1.get("ingredient_groups").is_none());
}
#[tokio::test]
async fn mobile_shopping_replays_after_restart_and_revalidates_authority() {
    let fixture = Fixture::new().await;
    let mut app = fixture.app().await;
    let home = home(&app, "alice").await;
    let base = format!("/api/households/{home}");
    let mobile = format!("{base}/domain/api/household/shopping/mobile");
    let legacy = format!("{base}/domain/api/household/shopping");
    join(&app, &base, "alice", "bob", "editor").await;
    join(&app, &base, "alice", "carol", "reader").await;
    let capability = call(&app, Some("carol"), "GET", &mobile, json!({})).await;
    assert_eq!(capability.0, StatusCode::OK);
    assert_eq!(capability.1["version"], 1);
    assert_eq!(capability.1["role"], "reader");
    let add = intent(
        0,
        "intent-add-00000001",
        json!({"operation":"add_manual","label":"Synthetic oats","ingredient_id":"oats","quantity":{"amount":"200.000001","unit":"g"}}),
    );
    assert_eq!(
        call(&app, Some("carol"), "POST", &mobile, add.clone())
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let applied = call(&app, Some("alice"), "POST", &mobile, add.clone()).await;
    assert_eq!(applied.0, StatusCode::OK);
    assert_eq!(applied.1["applied_revision"], 1);
    // Simulate response loss: discard response, close all server state, reopen persisted files.
    drop(app);
    app = fixture.app().await;
    let replayed = call(&app, Some("alice"), "POST", &mobile, add.clone()).await;
    assert_eq!(replayed.0, StatusCode::OK);
    assert_eq!(replayed.1["replayed"], true);
    assert_eq!(
        replayed.1["shopping"]["sources"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        replayed.1["shopping"]["items"][0]["remaining"],
        "200.000001"
    );
    let mut changed = add.clone();
    changed["command"]["label"] = json!("Changed request");
    assert_eq!(
        call(&app, Some("alice"), "POST", &mobile, changed).await.1["code"],
        "shopping_operation_reused"
    );
    let check = intent(
        1,
        "intent-check-000001",
        json!({"operation":"check","key":"ingredient:oats:g","checked":true}),
    );
    let extra = intent(
        1,
        "intent-add-00000002",
        json!({"operation":"add_manual","label":"New oats","ingredient_id":"oats","quantity":{"amount":"10","unit":"g"}}),
    );
    assert_eq!(
        call(&app, Some("bob"), "POST", &mobile, extra).await.0,
        StatusCode::OK
    );
    let conflict = call(&app, Some("alice"), "POST", &mobile, check).await;
    assert_eq!(conflict.0, StatusCode::CONFLICT);
    assert_eq!(conflict.1["code"], "shopping_revision_conflict");
    let old = call(&app, Some("alice"), "GET", &legacy, json!({})).await;
    assert_eq!(old.1["revision"], 2);
    assert_eq!(old.1["items"][0]["purchased"], "0.000000");
    assert!(old.1.get("mobile_receipts").is_none());
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
        call(&app, Some("bob"), "POST", &mobile, add.clone())
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let mut expired = add;
    expired["issued_at"] = json!(1);
    assert_eq!(
        call(&app, Some("alice"), "POST", &mobile, expired).await.1["code"],
        "shopping_operation_expired"
    );
    drop(app);
}

#[tokio::test]
async fn mobile_duplicate_checks_commit_once_even_with_concurrent_replays() {
    let fixture = Fixture::new().await;
    let app = fixture.app().await;
    let home = home(&app, "alice").await;
    let path = format!("/api/households/{home}/domain/api/household/shopping/mobile");
    let add = intent(
        0,
        "parallel-add-000001",
        json!({"operation":"add_manual","label":"Synthetic milk","ingredient_id":null,"quantity":{"amount":"0.000001","unit":"l"}}),
    );
    let first = call(&app, Some("alice"), "POST", &path, add).await;
    let key = first.1["shopping"]["items"][0]["key"].clone();
    let check = intent(
        1,
        "parallel-check-00001",
        json!({"operation":"check","key":key,"checked":true}),
    );
    let (a, b) = tokio::join!(
        call(&app, Some("alice"), "POST", &path, check.clone()),
        call(&app, Some("alice"), "POST", &path, check)
    );
    assert_eq!(a.0, StatusCode::OK);
    assert_eq!(b.0, StatusCode::OK);
    assert_ne!(a.1["replayed"], b.1["replayed"]);
    let read = call(&app, Some("alice"), "GET", &path, json!({})).await;
    assert_eq!(read.1["shopping"]["revision"], 2);
    assert_eq!(read.1["shopping"]["items"][0]["purchased"], "0.001000");
    assert_eq!(
        read.1["shopping"]["items"][0]["contributions"][0]["receipts"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    drop(app);
}

#[tokio::test]
async fn mobile_receipt_capacity_preserves_live_receipts_and_prunes_only_expired_intent() {
    let fixture = Fixture::new().await;
    let mut app = fixture.app().await;
    let home = home(&app, "alice").await;
    let path = format!("/api/households/{home}/domain/api/household/shopping/mobile");
    let first = intent(
        0,
        "capacity-seed-00001",
        json!({"operation":"learning","enabled":false}),
    );
    assert_eq!(
        call(&app, Some("alice"), "POST", &path, first.clone())
            .await
            .0,
        StatusCode::OK
    );
    drop(app);
    let file = fixture
        .root
        .join("data")
        .join(&home)
        .join("household/shopping.json");
    let mut persisted: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    let now = first["issued_at"].as_u64().unwrap();
    persisted["revision"] = json!(1024);
    persisted["mobile_receipts"]=json!((0..1024).map(|i|json!({"actor":"alice","operation_id":format!("capacity-intent-{i:08}"),"issued_at":now,"recorded_at":now,"applied_revision":i+1,"request":{"expected_revision":i,"command":{"operation":"learning","enabled":false}}})).collect::<Vec<_>>());
    std::fs::write(&file, serde_json::to_vec(&persisted).unwrap()).unwrap();
    app = fixture.app().await;
    let next = intent(
        1024,
        "capacity-next-00001",
        json!({"operation":"learning","enabled":true}),
    );
    let full = call(&app, Some("alice"), "POST", &path, next.clone()).await;
    assert_eq!(full.1["code"], "shopping_receipt_capacity");
    assert_eq!(
        call(&app, Some("alice"), "GET", &path, json!({})).await.1["shopping"]["revision"],
        1024
    );
    drop(app);
    persisted["mobile_receipts"][0]["issued_at"] = json!(now - 8 * 86400 - 1);
    persisted["mobile_receipts"][0]["recorded_at"] = json!(now - 8 * 86400 - 1);
    std::fs::write(&file, serde_json::to_vec(&persisted).unwrap()).unwrap();
    app = fixture.app().await;
    assert_eq!(
        call(&app, Some("alice"), "POST", &path, next).await.0,
        StatusCode::OK
    );
    let mut expired = first;
    expired["operation_id"] = json!("capacity-intent-00000000");
    expired["issued_at"] = json!(now - 8 * 86400 - 1);
    assert_eq!(
        call(&app, Some("alice"), "POST", &path, expired).await.1["code"],
        "shopping_operation_expired"
    );
    drop(app);
    let mut corrupt: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    corrupt["mobile_receipts"][0]["applied_revision"] = json!(0);
    std::fs::write(&file, serde_json::to_vec(&corrupt).unwrap()).unwrap();
    app = fixture.app().await;
    assert_eq!(
        call(&app, Some("alice"), "GET", &path, json!({})).await.0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    drop(app);
}

#[tokio::test]
async fn locally_purchased_addition_never_checks_other_contributions() {
    let fixture = Fixture::new().await;
    let app = fixture.app().await;
    let home = home(&app, "alice").await;
    let path = format!("/api/households/{home}/domain/api/household/shopping/mobile");
    let old = intent(
        0,
        "existing-demand-0001",
        json!({"operation":"add_manual","label":"Existing oats","ingredient_id":"oats","quantity":{"amount":"500","unit":"g"}}),
    );
    assert_eq!(
        call(&app, Some("alice"), "POST", &path, old).await.0,
        StatusCode::OK
    );
    let bought = intent(
        1,
        "bought-offline-00001",
        json!({"operation":"add_manual","label":"Bought offline","ingredient_id":"oats","quantity":{"amount":"200.000001","unit":"g"},"purchased":true}),
    );
    let result = call(&app, Some("alice"), "POST", &path, bought.clone()).await;
    assert_eq!(result.0, StatusCode::OK);
    assert_eq!(result.1["shopping"]["items"][0]["remaining"], "500.000000");
    assert_eq!(result.1["shopping"]["items"][0]["purchased"], "200.000001");
    let replay = call(&app, Some("alice"), "POST", &path, bought).await;
    assert_eq!(replay.1["replayed"], true);
    assert_eq!(replay.1["shopping"]["revision"], 2);
    assert_eq!(replay.1["shopping"]["items"][0]["purchased"], "200.000001");
    drop(app);
}
