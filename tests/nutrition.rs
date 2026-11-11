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
    let bytes = to_bytes(response.into_body(), 2_097_152).await.unwrap();
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
    let (status, search) = request("GET", "/api/ingredients?q=haferkorn", Value::Null).await;
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
        json!("1,5"),
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
    assert!(mealshop_server::router_from_catalogue_json(&" ".repeat(2_097_153)).is_err());
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
            serde_json::from_slice(&to_bytes(response.into_body(), 2_097_152).await.unwrap())
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

#[tokio::test]
async fn expanded_catalogue_has_english_base_names_and_searchable_german_labels() {
    let (status, body) = request("GET", "/api/ingredients?q=Weizen%20405", Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["language"], "en");
    assert_eq!(body["ingredients"].as_array().unwrap().len(), 1);
    let flour = &body["ingredients"][0];
    assert_eq!(flour["id"], "wheat-flour-type-405");
    assert_eq!(flour["name"], "Wheat flour, Type 405");
    assert_eq!(flour["translations"]["de"]["name"], "Weizen Mehl, Type 405");
    assert_eq!(flour["source"]["record_id"], "C214100");
    assert_eq!(flour["source"]["license"], "CC-BY-4.0");
    assert!(flour["nutrients"]["available_carbohydrate"]["amount"].is_string());
    assert!(flour["nutrients"].get("carbohydrate").is_none());
    let (_, english) = request("GET", "/api/ingredients?q=wheat%20405", Value::Null).await;
    assert_eq!(body, english);
    for (query, id) in [
        ("Hafermilch", "oat-drink-unsweetened"),
        ("Magerquark", "quark-low-fat"),
        ("Erbsenprotein", "pea-protein-isolate-unspecified"),
        ("Reisbl%C3%A4tter", "rice-paper-unspecified"),
        ("Zucchine", "zucchini-raw"),
    ] {
        let (_, matches) =
            request("GET", &format!("/api/ingredients?q={query}"), Value::Null).await;
        assert!(
            matches["ingredients"]
                .as_array()
                .unwrap()
                .iter()
                .any(|i| i["id"] == id),
            "missing {id}"
        );
    }
}

#[tokio::test]
async fn requested_variants_keep_exact_source_identity_and_do_not_reuse_neighbouring_products() {
    for (id, source) in [
        ("rye-flour-type-997", "C223200"),
        ("spelt-flour-type-630", "C234000"),
        ("wheat-wholemeal-flour", "C211000"),
        ("milk-1-5-pasteurised", "M111200"),
        ("milk-1-5-lactose-free-pasteurised", "M114200"),
        ("quark-20-fat-dry-matter", "M713300"),
        ("skyr-plain", "M710100"),
        ("edamame-raw", "G750100"),
        ("lentil-flour", "H730400"),
        ("whey-protein-powder-usda-reference", "173180"),
    ] {
        let (status, ingredient) =
            request("GET", &format!("/api/ingredients/{id}"), Value::Null).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(ingredient["source"]["record_id"], source);
    }
    let (_, quark) = request(
        "GET",
        "/api/ingredients/quark-20-fat-dry-matter",
        Value::Null,
    )
    .await;
    assert_ne!(quark["nutrients"]["fat"]["amount"], "20"); // Dry-matter grade is not composition per 100 g.
    for id in [
        "pasta-lentil-dry-unspecified",
        "pea-protein-isolate-unspecified",
        "erythritol",
        "quark-double-cream-unspecified",
        "whey-protein-isolate-unspecified",
    ] {
        let (_, ingredient) = request("GET", &format!("/api/ingredients/{id}"), Value::Null).await;
        assert_eq!(
            ingredient["source"]["name"],
            "Mealshop identity-only record"
        );
        assert!(
            ingredient["nutrients"]
                .as_object()
                .unwrap()
                .values()
                .all(|v| v["amount"].is_null())
        );
        let (_, result) = request(
            "POST",
            "/api/nutrition/calculate",
            recipe(id, json!("100"), 1),
        )
        .await;
        assert_eq!(nutrient(&result, "energy_kcal")["status"], "unknown");
        assert_eq!(nutrient(&result, "energy_kcal")["total"], Value::Null);
    }
}

#[tokio::test]
async fn source_specific_carbohydrate_definitions_and_warning_codes_remain_separate() {
    let (_, result) = request("POST", "/api/nutrition/calculate", json!({
        "lines":[{"ingredient_id":"oats","grams":"100"},{"ingredient_id":"milk-1-5-pasteurised","grams":"100"}],
        "servings":2,"preparation":"cooking_unmodelled"
    })).await;
    assert_eq!(nutrient(&result, "carbohydrate")["status"], "partial");
    assert_eq!(
        nutrient(&result, "available_carbohydrate")["status"],
        "partial"
    );
    assert_eq!(nutrient(&result, "protein")["status"], "complete");
    assert_eq!(
        result["warning_codes"],
        json!([
            "composition_estimate",
            "cooking_unmodelled",
            "carbohydrate_definitions"
        ])
    );
    assert_eq!(
        result["warnings"].as_array().unwrap().len(),
        result["warning_codes"].as_array().unwrap().len()
    );
    assert_eq!(result["ingredients"][0]["name"], "Oats");
    assert_eq!(
        result["ingredients"][0]["translations"]["de"]["name"],
        "Haferkorn"
    );
    let (status, error) = request(
        "POST",
        "/api/nutrition/calculate",
        recipe("unknown", json!("1"), 1),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error["code"], "invalid_recipe");
    assert!(error["error"].is_string());
}

#[test]
fn translations_are_optional_but_cannot_override_the_english_base_or_use_invalid_keys() {
    let original: Value = serde_json::from_str(include_str!("fixtures/catalogue.json")).unwrap();
    assert!(mealshop_server::router_from_catalogue_json(&original.to_string()).is_ok());
    for mutate in 0..4 {
        let mut fixture = original.clone();
        match mutate {
            0 => fixture["language"] = json!("de"),
            1 => {
                fixture["ingredients"][0]["translations"] =
                    json!({"en":{"name":"Override","aliases":[]}});
            }
            2 => fixture["ingredients"][0]["translations"] = json!({"de":{"name":"","aliases":[]}}),
            _ => {
                fixture["ingredients"][0]["translations"] =
                    json!({"../de":{"name":"Fixture","aliases":[]}});
            }
        }
        assert!(mealshop_server::router_from_catalogue_json(&fixture.to_string()).is_err());
    }
}

#[tokio::test]
async fn declared_nutrition_ranges_are_summed_without_omitting_inputs_and_round_outwards() {
    let mut catalogue: Value =
        serde_json::from_str(include_str!("../data/catalogue-base.json")).unwrap();
    catalogue["ingredients"][0]["nutrients"]["energy_kcal"] = json!({"amount":null,"range":{"lower":"100.000001","upper":"120.000009"},"uncertainty":"Synthetic declared interval"});
    let app = mealshop_server::router_from_catalogue_json(&catalogue.to_string()).unwrap();
    let (_,body)=request_on(app.clone(),"POST","/api/nutrition/calculate",json!({"lines":[{"ingredient_id":"oats","grams":"100"},{"ingredient_id":"chia","grams":"100"}],"servings":2,"preparation":"as_recorded"})).await;
    let energy = nutrient(&body, "energy_kcal");
    assert_eq!(energy["total"], Value::Null);
    assert_eq!(energy["known_total"], "486.00");
    assert_eq!(energy["range"]["status"], "complete");
    assert_eq!(
        energy["range"]["total"],
        json!({"lower":"586.00","upper":"606.01"})
    );
    assert_eq!(
        energy["range"]["per_serving"],
        json!({"lower":"293.00","upper":"303.01"})
    );
    let (_, tiny) = request_on(
        app,
        "POST",
        "/api/nutrition/calculate",
        recipe("oats", json!("0.000001"), 1),
    )
    .await;
    assert_eq!(
        nutrient(&tiny, "energy_kcal")["range"]["total"],
        json!({"lower":"0.00","upper":"0.01"})
    );
    for reading in [
        json!({"amount":null,"range":{"lower":"2","upper":"1"},"uncertainty":"Invalid reversed range"}),
        json!({"amount":"1","range":{"lower":"1","upper":"2"},"uncertainty":"Ambiguous value and range"}),
    ] {
        catalogue["ingredients"][0]["nutrients"]["energy_kcal"] = reading;
        assert!(mealshop_server::router_from_catalogue_json(&catalogue.to_string()).is_err());
    }
}

#[tokio::test]
async fn unknown_inputs_make_the_range_a_covered_subtotal_instead_of_false_total() {
    let mut catalogue: Value =
        serde_json::from_str(include_str!("../data/catalogue-base.json")).unwrap();
    catalogue["ingredients"][0]["nutrients"]["protein"] = json!({"amount":null,"range":{"lower":"10","upper":"20"},"uncertainty":"Synthetic declared interval"});
    catalogue["ingredients"][2]["source"]["version"] = json!("Synthetic unbounded test record");
    let app = mealshop_server::router_from_catalogue_json(&catalogue.to_string()).unwrap();
    let (_,body)=request_on(app,"POST","/api/nutrition/calculate",json!({"lines":[{"ingredient_id":"oats","grams":"100"},{"ingredient_id":"psyllium-husks","grams":"100"}],"servings":1,"preparation":"as_recorded"})).await;
    let row = nutrient(&body, "protein");
    assert_eq!(row["range"]["status"], "partial");
    assert_eq!(
        row["range"]["known_total"],
        json!({"lower":"10.00","upper":"20.00"})
    );
    assert_eq!(
        row["range"]["missing_ingredients"],
        json!(["psyllium-husks"])
    );
    assert!(row["range"].get("total").is_none());
}

#[tokio::test]
async fn every_bundled_food_has_core_values_and_psyllium_is_a_source_backed_recipe_range() {
    let (_, list) = request("GET", "/api/ingredients", Value::Null).await;
    let items = list["ingredients"].as_array().unwrap();
    assert_eq!(items.len(), 313);
    for item in items {
        let covered = |id: &str| {
            item["nutrients"][id]["amount"].is_string()
                || item["nutrition_estimates"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|p| p["values"].get(id).is_some())
        };
        for id in ["energy_kcal", "protein", "fat"] {
            assert!(covered(id), "{} missing {id}", item["id"]);
        }
        assert!(covered("available_carbohydrate") || covered("carbohydrate"));
    }
    let (_, result) = request("POST", "/api/nutrition/calculate", json!({"lines":[{"ingredient_id":"oats","grams":"100"},{"ingredient_id":"psyllium-husks","grams":"100"}],"servings":2,"preparation":"as_recorded"})).await;
    let energy = nutrient(&result, "energy_kcal");
    assert_eq!(energy["total"], Value::Null);
    assert_eq!(energy["range"]["status"], "complete");
    assert_eq!(
        energy["range"]["total"],
        json!({"lower":"580.00","upper":"597.00"})
    );
    assert_eq!(
        energy["range"]["per_serving"],
        json!({"lower":"290.00","upper":"298.50"})
    );
    assert_eq!(
        energy["range"]["estimated_ingredients"],
        json!(["psyllium-husks"])
    );
    assert_eq!(
        result["ingredients"][1]["nutrition_estimates"][0]["references"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(nutrient(&result, "beta_glucan")["total"], Value::Null);
}

#[test]
fn comparison_profiles_reject_fabricated_bounds_sources_and_overlapping_evidence() {
    let original: Value =
        serde_json::from_str(include_str!("../data/catalogue-base.json")).unwrap();
    let registry: Value =
        serde_json::from_str(include_str!("../data/nutrition-estimates-v1.json")).unwrap();
    let profile = registry["profiles"]["psyllium-husks"]["profiles"][0].clone();
    for variant in 0..6 {
        let mut p = profile.clone();
        match variant {
            0 => p["values"]["energy_kcal"]["upper"] = json!("999"),
            1 => p["references"][0]["source"]["url"] = json!("http://example.org"),
            2 => p["references"][0]["source"]["license"] = json!(""),
            3 => p["references"][0]["values"]["energy_kcal"] = json!({"lower":"3","upper":"2"}),
            4 => p["values"]["beta_glucan"] = json!({"lower":"0","upper":"0"}),
            _ => p["method"] = json!("source_supplement"),
        }
        let mut catalogue = original.clone();
        catalogue["ingredients"][2]["nutrition_estimates"] = json!([p]);
        assert!(mealshop_server::router_from_catalogue_json(&catalogue.to_string()).is_err());
    }
    let mut catalogue = original;
    catalogue["ingredients"][2]["nutrition_estimates"] = json!([profile.clone(), profile]);
    assert!(mealshop_server::router_from_catalogue_json(&catalogue.to_string()).is_err());
}
