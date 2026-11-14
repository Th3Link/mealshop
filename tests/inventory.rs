//! F15 transactions use only isolated synthetic households and fixed-point quantities.
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
const OWNER: &str = "inventory_owner_test_credential_1234567";
const READER: &str = "inventory_reader_test_credential_123456";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "mealshop-inventory-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&p).unwrap();
        Self(p)
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
    body: Value,
) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json");
    if let Some(token) = token {
        req = req.header("authorization", format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(req.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), 8_388_608).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}
async fn ok(app: &axum::Router, method: &str, path: &str, body: Value) -> Value {
    let (status, value) = call(app, Some(OWNER), method, path, body).await;
    assert!(status.is_success(), "{path}: {status} {value}");
    value
}
async fn inventory(app: &axum::Router) -> Value {
    ok(
        app,
        "GET",
        "/api/household/inventory?limit=500",
        Value::Null,
    )
    .await
}
async fn shopping(app: &axum::Router, command: Value) -> Value {
    let revision = ok(app, "GET", "/api/household/shopping", Value::Null).await["revision"].clone();
    ok(
        app,
        "POST",
        "/api/household/shopping",
        json!({"expected_revision":revision,"command":command}),
    )
    .await
}
fn metadata() -> Value {
    json!({"location":"Acceptance pantry","batch":"synthetic batch","expiry":null})
}
fn lot(quantity: &str) -> Value {
    json!({"operation":"create_lot","target":{"kind":"ingredient","ingredient_id":"oats"},"quantity":{"amount":quantity,"unit":"g"},"metadata":metadata()})
}
async fn request(app: &axum::Router, command: Value) -> Value {
    let revision = inventory(app).await["revision"].clone();
    json!({"expected_revision":revision,"idempotency_key":format!("inventory-request-{:020}",NEXT.fetch_add(1,Ordering::Relaxed)),"reviewed":true,"command":command})
}
async fn apply(app: &axum::Router, command: Value) -> Value {
    let input = request(app, command).await;
    let before = inventory(app).await;
    let preview = ok(
        app,
        "POST",
        "/api/household/inventory/preview",
        input.clone(),
    )
    .await;
    assert_eq!(preview["expected_revision"], before["revision"]);
    assert_eq!(
        inventory(app).await,
        before,
        "Preview cannot mutate stock or shopping"
    );
    ok(app, "POST", "/api/household/inventory", input).await
}
fn row<'a>(view: &'a Value, id: &str) -> &'a Value {
    view["lots"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["id"] == id)
        .unwrap()
}
#[tokio::test]
async fn exact_lots_review_authorization_corrections_and_archive() {
    let dir = Directory::new();
    let app = dir.app().unwrap();
    assert_eq!(
        call(&app, None, "GET", "/api/household/inventory", Value::Null)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let input = request(&app, lot("100.500000")).await;
    for path in [
        "/api/household/inventory",
        "/api/household/inventory/preview",
    ] {
        assert_eq!(
            call(&app, Some(READER), "POST", path, input.clone())
                .await
                .0,
            StatusCode::FORBIDDEN
        );
    }
    let mut unreviewed = input.clone();
    unreviewed["reviewed"] = json!(false);
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/inventory",
            unreviewed
        )
        .await
        .1["code"],
        "inventory_review_required"
    );
    let created = apply(&app, lot("100.500000")).await;
    let id = created["inventory"]["lots"][0]["id"].as_str().unwrap();
    assert_eq!(
        created["inventory"]["lots"][0]["original_quantity"]["amount"],
        "100.500000"
    );
    let invalid=request(&app,json!({"operation":"adjust_lot","lot_id":id,"quantity":{"amount":"5","unit":"ml"},"reason":"Synthetic count"})).await;
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/inventory",
            invalid
        )
        .await
        .1["code"],
        "inventory_dimension_mismatch"
    );
    apply(&app,json!({"operation":"update_lot","lot_id":id,"metadata":{"location":"Acceptance fridge","batch":"corrected batch","expiry":"2026-12-31"}})).await;
    let corrected=apply(&app,json!({"operation":"adjust_lot","lot_id":id,"quantity":{"amount":"0","unit":"g"},"reason":"Empty container"})).await;
    assert_eq!(row(&corrected["inventory"], id)["remaining"], "0.000000");
    let reversed=apply(&app,json!({"operation":"reverse","movement_id":corrected["operation_id"],"reason":"Count corrected"})).await;
    assert_eq!(row(&reversed["inventory"], id)["remaining"], "100.500000");
    let before = inventory(&app).await;
    let nonempty = request(&app, json!({"operation":"archive_lot","lot_id":id})).await;
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/inventory",
            nonempty
        )
        .await
        .1["code"],
        "inventory_not_empty"
    );
    assert_eq!(inventory(&app).await, before);
    let counted=apply(&app,json!({"operation":"adjust_lot","lot_id":id,"quantity":{"amount":"0","unit":"g"},"reason":"Consumed outside a recorded cooking session"})).await;
    let archived = apply(&app, json!({"operation":"archive_lot","lot_id":id})).await;
    assert_eq!(row(&archived["inventory"], id)["archived"], true);
    let restored=apply(&app,json!({"operation":"reverse","movement_id":counted["operation_id"],"reason":"Correct an archived count"})).await;
    assert_eq!(row(&restored["inventory"], id)["remaining"], "100.500000");
    assert_eq!(row(&restored["inventory"], id)["archived"], false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(dir.0.join("household/shopping.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
#[tokio::test]
async fn reservations_reduce_only_missing_purchases_and_receipt_undo_requires_reviewed_reversal() {
    let dir = Directory::new();
    let app = dir.app().unwrap();
    apply(&app, lot("120")).await;
    for n in ["100", "80"] {
        shopping(&app,json!({"operation":"add_manual","label":"Acceptance oats","ingredient_id":"oats","quantity":{"amount":n,"unit":"g"}})).await;
    }
    let list = ok(&app, "GET", "/api/household/shopping", Value::Null).await;
    let ids = list["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["id"].clone())
        .collect::<Vec<_>>();
    apply(
        &app,
        json!({"operation":"reserve","source_ids":ids,"as_of":"2026-11-12"}),
    )
    .await;
    let list = ok(&app, "GET", "/api/household/shopping", Value::Null).await;
    assert_eq!(list["items"][0]["remaining"], "60.000000");
    assert_eq!(list["items"][0]["reserved"], "120.000000");
    assert_eq!(list["items"][0]["purchased"], "0.000000");
    let before = inventory(&app).await;
    assert_eq!(before["lots"][0]["remaining"], "120.000000");
    assert_eq!(before["lots"][0]["available"], "0.000000");
    let rejected=request(&app,json!({"operation":"adjust_lot","lot_id":before["lots"][0]["id"],"quantity":{"amount":"119","unit":"g"},"reason":"Observed quantity"})).await;
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/inventory",
            rejected
        )
        .await
        .1["code"],
        "inventory_reserved"
    );
    shopping(
        &app,
        json!({"operation":"check","key":"ingredient:oats:g","checked":true}),
    )
    .await;
    let view = inventory(&app).await;
    assert_eq!(view["purchases"][0]["amount"], "60.000000");
    let received=apply(&app,json!({"operation":"receive","purchase_ids":[view["purchases"][0]["id"]],"metadata":metadata()})).await;
    assert_eq!(received["inventory"]["lots"][1]["remaining"], "60.000000");
    let revision = received["inventory"]["revision"].clone();
    let undo=call(&app,Some(OWNER),"POST","/api/household/shopping",json!({"expected_revision":revision,"command":{"operation":"check","key":"ingredient:oats:g","checked":false}})).await;
    assert_eq!(undo.1["code"], "inventory_receipt_booked");
    apply(&app,json!({"operation":"reverse","movement_id":received["operation_id"],"reason":"Purchase corrected"})).await;
    let undone = shopping(
        &app,
        json!({"operation":"check","key":"ingredient:oats:g","checked":false}),
    )
    .await;
    assert_eq!(undone["items"][0]["remaining"], "60.000000");
    let id = undone["sources"][0]["id"].clone();
    let changed=shopping(&app,json!({"operation":"add_manual","source_id":id,"label":"Acceptance oats","ingredient_id":"oats","quantity":{"amount":"90","unit":"g"}})).await;
    assert_eq!(changed["items"][0]["reserved"], "20.000000");
    assert_eq!(changed["items"][0]["remaining"], "150.000000");
}
#[tokio::test]
async fn concurrent_retry_restart_and_payload_conflict_never_duplicate_physical_stock() {
    let dir = Directory::new();
    let app = dir.app().unwrap();
    let input = request(&app, lot("100.500000")).await;
    let (one, two) = tokio::join!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/inventory",
            input.clone()
        ),
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/inventory",
            input.clone()
        )
    );
    assert_eq!(one.0, StatusCode::OK);
    assert_eq!(two.1["operation_id"], one.1["operation_id"]);
    let mut altered = input.clone();
    altered["command"]["quantity"]["amount"] = json!("200");
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/inventory",
            altered
        )
        .await
        .1["code"],
        "inventory_request_conflict"
    );
    drop(app);
    let app = dir.app().unwrap();
    let replay = ok(&app, "POST", "/api/household/inventory", input.clone()).await;
    assert_eq!(replay["operation_id"], one.1["operation_id"]);
    assert_eq!(replay["inventory"]["lots"].as_array().unwrap().len(), 1);
    assert_eq!(
        replay["inventory"]["movements"].as_array().unwrap().len(),
        1
    );
    assert!(
        !replay
            .to_string()
            .contains(input["idempotency_key"].as_str().unwrap())
    );
    let mut stale = input.clone();
    stale["idempotency_key"] = json!("inventory-distinct-intent-0001");
    assert_eq!(
        call(&app, Some(OWNER), "POST", "/api/household/inventory", stale)
            .await
            .0,
        StatusCode::CONFLICT
    );
}
#[tokio::test]
async fn manual_offers_keep_snapshot_validity_and_current_hard_exclusions() {
    let dir = Directory::new();
    let app = dir.app().unwrap();
    let list = shopping(
        &app,
        json!({"operation":"save_shop","name":"Acceptance shop"}),
    )
    .await;
    let shop = list["shops"]["entries"][0]["id"].clone();
    let offer=apply(&app,json!({"operation":"save_offer","offer":{"id":null,"shop_id":shop,"target":{"kind":"ingredient","ingredient_id":"oats"},"quantity":{"amount":"0.5","unit":"kg"},"price":"1.99","currency":"EUR","valid_from":"2026-11-01","valid_until":"2026-11-30","source":"Synthetic printed offer","source_url":"https://example.org/acceptance-offer"}})).await;
    let id = offer["inventory"]["offers"][0]["offer"]["id"].clone();
    ok(&app,"PUT","/api/household/profiles",json!({"expected_revision":0,"members":[{"id":"person-a","name":"Acceptance attendee","allergies":[],"intolerances":[],"excluded_ingredients":["oats"],"disliked_ingredients":[]}]})).await;
    let command = json!({"operation":"add_offer","offer_id":id,"packages":2,"as_of":"2026-11-12","member_ids":["person-a"]});
    let blocked = request(&app, command.clone()).await;
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/inventory",
            blocked
        )
        .await
        .1["code"],
        "inventory_offer_blocked"
    );
    ok(&app,"PUT","/api/household/profiles",json!({"expected_revision":1,"members":[{"id":"person-a","name":"Acceptance attendee","allergies":[],"intolerances":[],"excluded_ingredients":[],"disliked_ingredients":[]}]})).await;
    let reviewed = request(&app, command.clone()).await;
    ok(
        &app,
        "POST",
        "/api/household/inventory/preview",
        reviewed.clone(),
    )
    .await;
    ok(&app,"PUT","/api/household/profiles",json!({"expected_revision":2,"members":[{"id":"person-a","name":"Acceptance attendee","allergies":[],"intolerances":[],"excluded_ingredients":["oats"],"disliked_ingredients":[]}]})).await;
    let changed = call(
        &app,
        Some(OWNER),
        "POST",
        "/api/household/inventory",
        reviewed,
    )
    .await;
    assert_eq!(changed.0, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(changed.1["code"], "inventory_offer_blocked");
    assert_eq!(
        ok(&app, "GET", "/api/household/shopping", Value::Null).await["sources"],
        json!([])
    );
    ok(&app,"PUT","/api/household/profiles",json!({"expected_revision":3,"members":[{"id":"person-a","name":"Acceptance attendee","allergies":[],"intolerances":[],"excluded_ingredients":[],"disliked_ingredients":[]}]})).await;
    let mut invalid = command.clone();
    invalid["as_of"] = json!("2026-12-01");
    let invalid = request(&app, invalid).await;
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/inventory",
            invalid
        )
        .await
        .1["code"],
        "inventory_offer_inactive"
    );
    apply(&app, command).await;
    let list = ok(&app, "GET", "/api/household/shopping", Value::Null).await;
    assert_eq!(list["items"][0]["remaining"], "1000.000000");
    assert_eq!(list["sources"][0]["origin"]["offer"]["price"], "1.99");
    assert_eq!(
        list["sources"][0]["origin"]["offer"]["source"],
        "Synthetic printed offer"
    );
    apply(&app, json!({"operation":"archive_offer","offer_id":id})).await;
    assert_eq!(
        ok(&app, "GET", "/api/household/shopping", Value::Null).await["sources"][0]["origin"]["offer"]
            ["archived"],
        false
    );
}
#[tokio::test]
async fn failed_publication_keeps_original_balance_and_blocks_further_writes_until_recovery() {
    let dir = Directory::new();
    let app = dir.app().unwrap();
    let opening = request(&app, lot("100")).await;
    ok(&app, "POST", "/api/household/inventory", opening.clone()).await;
    let before = inventory(&app).await;
    let original = std::fs::read(dir.0.join("household/shopping.json")).unwrap();
    let input = request(&app, lot("20")).await;
    std::fs::write(
        dir.0.join("household/.shopping.pending"),
        b"synthetic interrupted write",
    )
    .unwrap();
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/inventory",
            input.clone()
        )
        .await
        .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "GET",
            "/api/household/inventory",
            Value::Null
        )
        .await
        .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/inventory",
            opening
        )
        .await
        .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        std::fs::read(dir.0.join("household/shopping.json")).unwrap(),
        original
    );
    assert_eq!(
        call(&app, Some(OWNER), "POST", "/api/household/inventory", input)
            .await
            .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    drop(app);
    assert!(dir.app().is_err());
    std::fs::remove_file(dir.0.join("household/.shopping.pending")).unwrap();
    let app = dir.app().unwrap();
    assert_eq!(inventory(&app).await, before);
}

#[tokio::test]
async fn completed_cooking_consumes_holds_once_without_reopening_purchase_need_and_reverses_explicitly()
 {
    let dir = Directory::new();
    let app = dir.app().unwrap();
    let recipe=ok(&app,"POST","/api/recipes",json!({"title":"Inventory acceptance recipe","language":"en","servings":2,"preparation":"as_recorded","lines":[{"ingredient_id":"oats","grams":"100.500000"}],"steps":[{"instruction":"Prepare synthetic stock ingredients","active_seconds":0,"passive_seconds":0}]})).await;
    apply(&app, lot("80.250000")).await;
    let list=shopping(&app,json!({"operation":"add_recipe","recipe":{"recipe_id":recipe["id"],"revision":1,"servings":2}})).await;
    let source = list["sources"][0]["id"].clone();
    apply(
        &app,
        json!({"operation":"reserve","source_ids":[source],"as_of":"2026-11-12"}),
    )
    .await;
    let template=ok(&app,"GET",&format!("/api/household/cooking/template?recipe_id={}&revision=1&servings=2&timezone=Europe%2FBerlin",recipe["id"].as_str().unwrap()),Value::Null).await;
    let preview = ok(
        &app,
        "POST",
        "/api/household/cooking/preview",
        json!({"expected_revision":0,"proposal":{"kind":"create","draft":template["draft"]}}),
    )
    .await;
    let saved=ok(&app,"POST","/api/household/cooking",json!({"expected_revision":0,"command":{"operation":"apply","preview_id":preview["preview_id"],"reviewed":true}})).await;
    let run = saved["run_id"].clone();
    let command = json!({"operation":"consume","run_id":run,"source_ids":[source],"include_unreserved":true,"as_of":"2026-11-12"});
    let premature = request(&app, command.clone()).await;
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/inventory",
            premature
        )
        .await
        .1["code"],
        "inventory_cooking_incomplete"
    );
    let detail = ok(
        &app,
        "GET",
        &format!("/api/household/cooking/{}", run.as_str().unwrap()),
        Value::Null,
    )
    .await;
    let step = detail["run"]["steps"][0]["stage"]["id"].clone();
    ok(&app,"POST","/api/household/cooking",json!({"expected_revision":1,"command":{"operation":"timer","run_id":run,"step_id":step,"action":"complete"}})).await;
    let selection = ok(
        &app,
        "GET",
        &format!("/api/household/inventory?run_id={}", run.as_str().unwrap()),
        Value::Null,
    )
    .await;
    assert_eq!(
        selection["cooking_selection"]["requirements"][0]["amount"],
        "100.500000"
    );
    assert_eq!(
        selection["cooking_selection"]["source_ids"],
        json!([source])
    );
    let consumed = apply(&app, command.clone()).await;
    assert_eq!(consumed["inventory"]["lots"][0]["remaining"], "0.000000");
    assert_eq!(consumed["inventory"]["lots"][0]["reserved"], "0.000000");
    let list = ok(&app, "GET", "/api/household/shopping", Value::Null).await;
    assert_eq!(list["items"][0]["used"], "80.250000");
    assert_eq!(list["items"][0]["remaining"], "20.250000");
    assert_eq!(
        consumed["inventory"]["movements"][0]["kind"]["unbooked"][0]["amount"],
        "20.250000"
    );
    let duplicate = request(&app, command).await;
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/inventory",
            duplicate
        )
        .await
        .1["code"],
        "inventory_cooking_booked"
    );
    let reversed=apply(&app,json!({"operation":"reverse","movement_id":consumed["operation_id"],"reason":"Actual consumption corrected"})).await;
    assert_eq!(reversed["inventory"]["lots"][0]["remaining"], "80.250000");
    assert_eq!(
        ok(&app, "GET", "/api/household/shopping", Value::Null).await["items"][0]["remaining"],
        "100.500000"
    );
    drop(app);
    let app = dir.app().unwrap();
    assert_eq!(inventory(&app).await, reversed["inventory"]);
}
#[tokio::test]
async fn passed_dates_and_cross_dimension_lots_do_not_supply_a_proposal() {
    let dir = Directory::new();
    let app = dir.app().unwrap();
    let mut expired = lot("100");
    expired["metadata"]["expiry"] = json!("2026-11-11");
    apply(&app, expired).await;
    let mut volume = lot("200");
    volume["quantity"]["unit"] = json!("ml");
    apply(&app, volume).await;
    let list=shopping(&app,json!({"operation":"add_manual","label":"Acceptance oats","ingredient_id":"oats","quantity":{"amount":"100","unit":"g"}})).await;
    let body = request(
        &app,
        json!({"operation":"reserve","source_ids":[list["sources"][0]["id"]],"as_of":"2026-11-12"}),
    )
    .await;
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/inventory/preview",
            body
        )
        .await
        .1["code"],
        "inventory_no_stock"
    );
    let mut fresh = lot("60");
    fresh["metadata"]["expiry"] = json!("2026-11-20");
    apply(&app, fresh).await;
    let mut later = lot("60");
    later["metadata"]["expiry"] = json!("2026-12-01");
    apply(&app, later).await;
    let result = apply(
        &app,
        json!({"operation":"reserve","source_ids":[list["sources"][0]["id"]],"as_of":"2026-11-12"}),
    )
    .await;
    assert_eq!(result["inventory"]["lots"][0]["reserved"], "0.000000");
    assert_eq!(result["inventory"]["lots"][1]["reserved"], "0.000000");
    assert_eq!(result["inventory"]["lots"][2]["reserved"], "60.000000");
    assert_eq!(result["inventory"]["lots"][3]["reserved"], "40.000000");
}
#[tokio::test]
async fn confirmed_product_bindings_do_not_substitute_a_different_barcode() {
    let dir = Directory::new();
    let app = dir.app().unwrap();
    for barcode in ["12345670", "034000470693"] {
        shopping(&app,json!({"operation":"save_product","product":{"barcode":barcode,"name":"Acceptance oats product","brand":"Synthetic brand","label_quantity":"100 g","quantity":{"amount":"100","unit":"g"},"ingredient_id":"oats","match_confirmed":true,"candidate_id":null}})).await;
    }
    let mut stock = lot("100");
    stock["target"] = json!({"kind":"product","barcode":"12345670"});
    apply(&app, stock).await;
    let list = shopping(
        &app,
        json!({"operation":"add_product","barcode":"034000470693","packages":1}),
    )
    .await;
    let body = request(
        &app,
        json!({"operation":"reserve","source_ids":[list["sources"][0]["id"]],"as_of":"2026-11-12"}),
    )
    .await;
    assert_eq!(
        call(&app, Some(OWNER), "POST", "/api/household/inventory", body)
            .await
            .1["code"],
        "inventory_no_stock"
    );
    let list=shopping(&app,json!({"operation":"add_manual","label":"Acceptance oats","ingredient_id":"oats","quantity":{"amount":"50","unit":"g"}})).await;
    let result = apply(
        &app,
        json!({"operation":"reserve","source_ids":[list["sources"][1]["id"]],"as_of":"2026-11-12"}),
    )
    .await;
    assert_eq!(result["inventory"]["lots"][0]["reserved"], "50.000000");
}

#[tokio::test]
async fn corrupted_opening_receipt_or_lot_reference_fails_closed_on_restart() {
    for corruption in ["opening_quantity", "receipt_key", "lot_reference"] {
        let dir = Directory::new();
        let app = dir.app().unwrap();
        apply(&app, lot("100")).await;
        drop(app);
        let path = dir.0.join("household/shopping.json");
        let mut data: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        match corruption {
            "opening_quantity" => {
                data["inventory"]["movements"][0]["entries"][0]["amount"] = json!("50");
            }
            "receipt_key" => data["inventory"]["operations"][0]["key"] = json!("short"),
            _ => data["inventory"]["movements"][0]["entries"][0]["lot_id"] = json!("lot-999999"),
        }
        std::fs::write(path, serde_json::to_vec(&data).unwrap()).unwrap();
        assert!(dir.app().is_err(), "{corruption}");
    }
}

#[tokio::test]
async fn stock_history_pages_preserve_order_and_expose_no_operation_secrets() {
    let directory = Directory::new();
    let app = directory.app().unwrap();
    for amount in ["1", "2", "3"] {
        apply(&app, lot(amount)).await;
    }
    let all = inventory(&app).await;
    let page = ok(
        &app,
        "GET",
        "/api/household/inventory?limit=1&offset=1&history_limit=1&history_offset=1",
        Value::Null,
    )
    .await;
    assert_eq!(page["total"], 3);
    assert_eq!(page["movement_total"], 3);
    assert_eq!(page["movement_offset"], 1);
    assert_eq!(page["movement_limit"], 1);
    assert_eq!(page["lots"].as_array().unwrap().len(), 1);
    assert_eq!(page["movements"].as_array().unwrap().len(), 1);
    assert_eq!(page["movements"][0], all["movements"][1]);
    assert_eq!(page["lots"][0], all["lots"][1]);
    assert!(!page.to_string().contains("idempotency_key"));
    let beyond = ok(
        &app,
        "GET",
        "/api/household/inventory?history_offset=9999",
        Value::Null,
    )
    .await;
    assert_eq!(beyond["movements"], json!([]));
}

fn prepared_storage(condition: &str) -> Value {
    json!({"location":"Synthetic freezer","condition":condition,"declared_on":"2026-11-15","use_by":null})
}
#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn prepared_six_equals_four_allocated_and_two_left_without_inventing_ingredient_stock() {
    let dir = Directory::new();
    let app = dir.app().unwrap();
    ok(&app,"PUT","/api/household/profiles",json!({"expected_revision":0,"members":[
        {"id":"one","name":"Synthetic","allergies":[],"excluded_ingredients":[],"disliked_ingredients":[]}]})).await;
    let recipe = ok(
        &app,
        "POST",
        "/api/recipes",
        json!({"title":"Synthetic prepared food","language":"en","servings":2,
        "preparation":"as_recorded","lines":[{"ingredient_id":"oats","grams":"200.000001"}],
        "steps":[{"instruction":"Synthetic preparation","active_seconds":60,"passive_seconds":0}]}),
    )
    .await;
    let command = json!({"operation":"record_prepared","input":{"recipe_id":recipe["id"],"revision":1,"portions":6,
        "produced_on":"2026-11-15","storage":prepared_storage("frozen"),"source":null}});
    let body = request(&app, command).await;
    let preview = ok(
        &app,
        "POST",
        "/api/household/inventory/preview",
        body.clone(),
    )
    .await;
    assert_eq!(preview["prepared_after"][0]["available"], 6);
    assert_eq!(inventory(&app).await["prepared"], json!([]));
    let first = ok(&app, "POST", "/api/household/inventory", body.clone()).await;
    let id = first["operation_id"].as_str().unwrap().to_owned();
    assert_eq!(first["inventory"]["lots"], json!([]));
    for date in ["2026-11-16", "2026-11-17"] {
        apply(
            &app,
            json!({"operation":"allocate_prepared","batch_id":id,"date":date,"meal":"dinner",
            "portions":2,"member_ids":["one"]}),
        )
        .await;
    }
    let v = inventory(&app).await;
    let b = &v["prepared"][0];
    assert_eq!(b["held"], 4);
    assert_eq!(b["available"], 2);
    assert_eq!(b["used"], 0);
    let over = request(
        &app,
        json!({"operation":"allocate_prepared","batch_id":id,"date":"2026-11-18",
        "meal":"dinner","portions":3,"member_ids":["one"]}),
    )
    .await;
    assert_eq!(
        call(&app, Some(OWNER), "POST", "/api/household/inventory", over)
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let allocation = b["batch"]["allocations"][0]["id"].clone();
    apply(
        &app,
        json!({"operation":"use_prepared","batch_id":id,"allocation_id":allocation}),
    )
    .await;
    let v = inventory(&app).await;
    assert_eq!(v["prepared"][0]["used"], 2);
    assert_eq!(v["prepared"][0]["held"], 2);
    assert_eq!(v["prepared"][0]["available"], 2);
    let forbidden = request(
        &app,
        json!({"operation":"reverse_prepared","batch_id":id,"reason":"Synthetic correction"}),
    )
    .await;
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/inventory",
            forbidden
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    apply(
        &app,
        json!({"operation":"store_prepared","batch_id":id,"storage":prepared_storage("chilled")}),
    )
    .await;
    drop(app);
    let app = dir.app().unwrap();
    let replay = ok(&app, "POST", "/api/household/inventory", body).await;
    assert_eq!(replay["replayed"], true);
    assert_eq!(replay["inventory"]["prepared"].as_array().unwrap().len(), 1);
    assert_eq!(
        replay["inventory"]["prepared"][0]["batch"]["storage"]["condition"],
        "chilled"
    );
    apply(&app,json!({"operation":"undo_prepared_use","batch_id":id,"allocation_id":allocation,"reason":"Synthetic correction"})).await;
    for a in inventory(&app).await["prepared"][0]["batch"]["allocations"]
        .as_array()
        .unwrap()
    {
        apply(
            &app,
            json!({"operation":"release_prepared","batch_id":id,"allocation_id":a["id"]}),
        )
        .await;
    }
    assert_eq!(inventory(&app).await["prepared"][0]["available"], 6);
    apply(
        &app,
        json!({"operation":"reverse_prepared","batch_id":id,"reason":"Observed yield corrected"}),
    )
    .await;
    assert_eq!(inventory(&app).await["prepared"][0]["available"], 0);
    assert_eq!(inventory(&app).await["lots"], json!([]));
    let reader = request(&app, json!({"operation":"record_prepared","input":{}})).await;
    assert_eq!(
        call(
            &app,
            Some(READER),
            "POST",
            "/api/household/inventory",
            reader
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn prepared_output_locks_linked_production_and_blocks_duplicate_output_or_overlapping_meals()
{
    let dir = Directory::new();
    let app = dir.app().unwrap();
    ok(&app,"PUT","/api/household/profiles",json!({"expected_revision":0,"members":[{"id":"one","name":"Synthetic person","allergies":[],"excluded_ingredients":[],"disliked_ingredients":[]}]})).await;
    let recipe=ok(&app,"POST","/api/recipes",json!({"title":"Synthetic linked production","language":"en","servings":2,"preparation":"as_recorded",
        "lines":[{"ingredient_id":"oats","grams":"100"}],"steps":[{"instruction":"Prepare fixture","active_seconds":60,"passive_seconds":0}]})).await;
    let plan = json!({"date":"2026-11-15","meal":"preparation","recipe":{"id":recipe["id"],"revision":1},"servings":6,
        "attendees":["one"],"calendar":null,"active_seconds":600,"elapsed_seconds":3600,"advance_seconds":0,
        "allocations":[{"date":"2026-11-16","meal":"dinner","servings":2},{"date":"2026-11-17","meal":"dinner","servings":2}]});
    ok(
        &app,
        "POST",
        "/api/household/mealplan/apply",
        json!({"plan":plan,"expected_revision":0,"profile_revision":1,"reviewed":true}),
    )
    .await;
    let command = json!({"operation":"record_prepared","input":{"recipe_id":recipe["id"],"revision":1,"portions":6,"produced_on":"2026-11-15",
        "storage":prepared_storage("frozen"),"source":{"date":"2026-11-15","meal":"preparation"}}});
    let result = apply(&app, command.clone()).await;
    let id = result["operation_id"].clone();
    let duplicate = request(&app, command).await;
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/inventory",
            duplicate
        )
        .await
        .1["code"],
        "prepared_production_already_recorded"
    );
    let mut changed = plan.clone();
    changed["servings"] = json!(8);
    let review = ok(&app, "POST", "/api/household/mealplan/preview", changed).await;
    assert_eq!(review["feasible"], false);
    assert!(
        review["conflicts"]
            .as_array()
            .unwrap()
            .contains(&json!("prepared_production_frozen"))
    );
    apply(&app,json!({"operation":"allocate_prepared","batch_id":id,"date":"2026-11-16","meal":"dinner","portions":2,"member_ids":["one"]})).await;
    let overlap=request(&app,json!({"operation":"allocate_prepared","batch_id":id,"date":"2026-11-16","meal":"dinner","portions":1,"member_ids":["one"]})).await;
    assert_eq!(
        call(
            &app,
            Some(OWNER),
            "POST",
            "/api/household/inventory",
            overlap
        )
        .await
        .1["code"],
        "prepared_slot_conflict"
    );
    let refs = json!([{"date":"2026-11-16","meal":"dinner"},{"date":"2026-11-17","meal":"dinner"}]);
    let shopping = ok(
        &app,
        "POST",
        "/api/household/shopping/plan-preview",
        json!({"expected_revision":2,"plan_revision":1,"slots":refs}),
    )
    .await;
    assert_eq!(shopping["changes"][0]["after"]["remaining"], "300.000000");
    assert_eq!(inventory(&app).await["prepared"][0]["available"], 4);
}
