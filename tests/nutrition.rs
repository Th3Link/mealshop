//! Black-box acceptance tests for F02.
#![allow(clippy::unwrap_used)]

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn request(method: &str, path: &str, body: Value) -> (StatusCode, Value) {
    request_on(mealshop_server::router().unwrap(), method, path, body).await
}

async fn request_on(
    router: axum::Router,
    method: &str,
    path: &str,
    body: Value,
) -> (StatusCode, Value) {
    let response = router
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("content-type", "application/json")
                .body(if method == "GET" {
                    Body::empty()
                } else {
                    Body::from(body.to_string())
                })
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1_000_000).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

fn recipe(ingredient: &str, grams: Value, servings: u32) -> Value {
    let mut line = json!({"ingredient_id": ingredient});
    line["grams"] = grams;
    json!({"lines":[line],"servings":servings,"preparation":"as_recorded"})
}

fn nutrient<'a>(body: &'a Value, id: &str) -> &'a Value {
    body["nutrients"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == id)
        .unwrap()
}

#[tokio::test]
async fn catalogue_search_and_detail_retain_provenance_and_evidence() {
    let (status, search) = request("GET", "/api/ingredients?q=hafer", Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(search["ingredients"].as_array().unwrap().len(), 1);
    assert_eq!(search["ingredients"][0]["id"], "oats");
    let (status, oats) = request("GET", "/api/ingredients/oats", Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(oats["basis"], "per_100g_edible_portion");
    assert_eq!(oats["source"]["record_id"], "169705");
    assert_eq!(oats["source"]["version"], "SR Legacy / 2019-04-01");
    assert_eq!(oats["source"]["license"], "CC0-1.0");
    assert_ne!(oats["uncertainty"].as_str().unwrap(), "");
    assert!(
        oats["education"][0]["conditions"]
            .as_str()
            .unwrap()
            .contains("3 g")
    );
    assert!(
        oats["education"][0]["limitations"]
            .as_str()
            .unwrap()
            .contains("not measured")
    );
    assert_eq!(oats["nutrients"]["beta_glucan"]["amount"], Value::Null);
    let (status, _) = request("GET", "/api/ingredients/not-present", Value::Null).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn recipe_and_serving_values_are_scaled_from_source_values() {
    let (status, result) = request(
        "POST",
        "/api/nutrition/calculate",
        json!({
            "lines":[{"ingredient_id":"oats","grams":"100"},{"ingredient_id":"chia","grams":"25"}],
            "servings":2,"preparation":"as_recorded"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let energy = nutrient(&result, "energy_kcal");
    assert_eq!(energy["total"], "510.50");
    assert_eq!(energy["per_serving"], "255.25");
    assert_eq!(energy["unit"], "kcal");
    assert_eq!(nutrient(&result, "protein")["total"], "21.03");
    assert_eq!(nutrient(&result, "protein")["per_serving"], "10.51");
    assert_eq!(nutrient(&result, "iron")["unit"], "mg");
    assert_eq!(nutrient(&result, "folate")["unit"], "µg");
    assert_eq!(result["ingredients"][0]["grams"], "100");
    assert_eq!(result["ingredients"][0]["source"]["record_id"], "169705");
}

#[tokio::test]
async fn portion_rounding_does_not_use_rounded_recipe_total() {
    let (_, result) = request(
        "POST",
        "/api/nutrition/calculate",
        recipe("oats", json!("0.1"), 2),
    )
    .await;
    assert_eq!(nutrient(&result, "energy_kcal")["total"], "0.39");
    assert_eq!(nutrient(&result, "energy_kcal")["per_serving"], "0.19");
    let (_, result) = request(
        "POST",
        "/api/nutrition/calculate",
        recipe("oats", json!("0.5"), 1),
    )
    .await;
    assert_eq!(nutrient(&result, "energy_kcal")["total"], "1.95");
}

#[tokio::test]
async fn unknown_and_partial_values_are_never_reported_as_complete_zero() {
    let (_, result) = request(
        "POST",
        "/api/nutrition/calculate",
        recipe("oats", json!("100"), 1),
    )
    .await;
    assert_eq!(nutrient(&result, "vitamin_c")["total"], "0.00");
    assert_eq!(nutrient(&result, "vitamin_c")["status"], "complete");
    assert_eq!(nutrient(&result, "beta_glucan")["total"], Value::Null);
    assert_eq!(nutrient(&result, "beta_glucan")["status"], "unknown");
    let (_, result) = request("POST", "/api/nutrition/calculate", json!({
        "lines":[{"ingredient_id":"oats","grams":"100"},{"ingredient_id":"psyllium-husks","grams":"5"}],
        "servings":2,"preparation":"cooking_unmodelled"
    })).await;
    let energy = nutrient(&result, "energy_kcal");
    assert_eq!(energy["status"], "partial");
    assert_eq!(energy["total"], Value::Null);
    assert_eq!(energy["per_serving"], Value::Null);
    assert_eq!(energy["known_total"], "389.00");
    assert_eq!(energy["missing_ingredients"], json!(["psyllium-husks"]));
    assert!(
        result["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w.as_str().unwrap().contains("retention"))
    );
}

#[tokio::test]
async fn invalid_requests_fail_without_silent_coercion() {
    for grams in [
        json!("0"),
        json!("-1"),
        json!("NaN"),
        json!("1e2"),
        json!("0.0000001"),
        json!("1000000001"),
        json!(2),
    ] {
        let (status, _) = request(
            "POST",
            "/api/nutrition/calculate",
            recipe("oats", grams.clone(), 1),
        )
        .await;
        assert!(status.is_client_error(), "accepted invalid amount {grams}");
    }
    for value in [
        recipe("oats", json!("1"), 0),
        recipe("unknown", json!("1"), 1),
        json!({"lines":[],"servings":1,"preparation":"as_recorded"}),
        json!({"lines":[{"ingredient_id":"oats","grams":"1","unit":"ml"}],"servings":1,"preparation":"as_recorded"}),
    ] {
        let (status, _) = request("POST", "/api/nutrition/calculate", value).await;
        assert!(status.is_client_error());
    }
}

#[tokio::test]
async fn custom_catalogue_isolated_and_fractional_results_remain_exact() {
    let router =
        mealshop_server::router_from_catalogue_json(include_str!("fixtures/catalogue.json"))
            .unwrap();
    let (_, list) = request_on(router.clone(), "GET", "/api/ingredients", Value::Null).await;
    assert_eq!(list["ingredients"].as_array().unwrap().len(), 2);
    let (_, result) = request_on(router, "POST", "/api/nutrition/calculate", json!({
        "lines":[{"ingredient_id":"fixture-a","grams":"50.000000"},{"ingredient_id":"fixture-b","grams":"100"}],
        "servings":2,"preparation":"as_recorded"
    })).await;
    assert_eq!(nutrient(&result, "energy_kcal")["total"], "0.03");
    assert_eq!(nutrient(&result, "energy_kcal")["per_serving"], "0.01");
    assert_eq!(result["ingredients"][0]["grams"], "50.000000");
    assert_eq!(nutrient(&result, "protein")["status"], "unknown");
}

#[tokio::test]
async fn bounds_and_repeated_ingredients_do_not_overflow_or_drop_unknowns() {
    let (_, result) = request(
        "POST",
        "/api/nutrition/calculate",
        json!({
            "lines":[{"ingredient_id":"oats","grams":"100"},{"ingredient_id":"oats","grams":"100"}],
            "servings":1,"preparation":"as_recorded"
        }),
    )
    .await;
    assert_eq!(nutrient(&result, "energy_kcal")["total"], "778.00");
    assert_eq!(
        nutrient(&result, "beta_glucan")["missing_ingredients"],
        json!(["oats"])
    );
    let mut fixture: Value = serde_json::from_str(include_str!("fixtures/catalogue.json")).unwrap();
    fixture["ingredients"][0]["nutrients"]["energy_kcal"]["amount"] = json!("1000000000");
    let router = mealshop_server::router_from_catalogue_json(&fixture.to_string()).unwrap();
    let (_, result) = request_on(
        router.clone(),
        "POST",
        "/api/nutrition/calculate",
        json!({
            "lines":vec![json!({"ingredient_id":"fixture-a","grams":"1000000000"});100],
            "servings":1_000_000,"preparation":"as_recorded"
        }),
    )
    .await;
    assert_eq!(
        nutrient(&result, "energy_kcal")["total"],
        "1000000000000000000.00"
    );
    assert_eq!(
        nutrient(&result, "energy_kcal")["per_serving"],
        "1000000000000.00"
    );
    for (count, servings) in [(101, 1), (1, 1_000_001)] {
        let (status, _) = request_on(
            router.clone(),
            "POST",
            "/api/nutrition/calculate",
            json!({
                "lines":vec![json!({"ingredient_id":"fixture-a","grams":"1"});count],
                "servings":servings,"preparation":"as_recorded"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[test]
fn catalogue_validation_rejects_incompatible_or_ambiguous_records() {
    let original: Value = serde_json::from_str(include_str!("fixtures/catalogue.json")).unwrap();
    for mutate in 0..8 {
        let mut fixture = original.clone();
        match mutate {
            0 => fixture["schema_version"] = json!(2),
            1 => fixture["ingredients"][1]["id"] = json!("fixture-a"),
            2 => fixture["ingredients"][0]["source"]["version"] = json!(""),
            3 => fixture["ingredients"][0]["source"]["url"] = json!("javascript:alert(1)"),
            4 => fixture["ingredients"][0]["basis"] = json!("per_ml"),
            5 => fixture["ingredients"][0]["nutrients"]["energy_kcal"]["amount"] = json!(-1),
            6 => fixture["ingredients"][0]["nutrients"]["energy_kcal"]["amount"] = json!("-1"),
            _ => {
                fixture["ingredients"][0]["nutrients"]["unknown_nutrient"] =
                    json!({"amount":"1","uncertainty":"test"});
            }
        }
        assert!(
            mealshop_server::router_from_catalogue_json(&fixture.to_string()).is_err(),
            "accepted mutation {mutate}"
        );
    }
    assert!(mealshop_server::router_from_catalogue_json("{").is_err());
    assert!(mealshop_server::router_from_catalogue_json(&" ".repeat(1_048_577)).is_err());
    assert!(
        mealshop_server::router_from_catalogue_json(
            r#"{"schema_version":1,"version":"empty","ingredients":[]}"#
        )
        .is_ok()
    );
}

#[tokio::test]
async fn oversized_and_malformed_bodies_return_structured_errors() {
    for body in ["not-json".to_owned(), " ".repeat(131_073)] {
        let response = mealshop_server::router()
            .unwrap()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/nutrition/calculate")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.status().is_client_error());
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 1_000_000).await.unwrap())
                .unwrap();
        assert!(value["error"].is_string());
    }
}

#[test]
fn duplicate_nutrient_keys_cannot_silently_replace_source_values() {
    let fixture = include_str!("fixtures/catalogue.json").replacen(
        "\"energy_kcal\": {",
        "\"energy_kcal\": {\"amount\":\"999\",\"uncertainty\":\"duplicate\"}, \"energy_kcal\": {",
        1,
    );
    assert!(mealshop_server::router_from_catalogue_json(&fixture).is_err());
}
