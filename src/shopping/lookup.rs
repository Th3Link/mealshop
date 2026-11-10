use super::products::{Candidate, Evidence, evidence_valid, gtin};
use crate::household::{self, Shared};
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
static LAST_LOOKUP: AtomicU64 = AtomicU64::new(0);
pub(crate) fn now() -> Result<u64, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|_| "shopping_lookup_unavailable")
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    expected_revision: u64,
    barcode: String,
    consent: bool,
}
fn failure(code: &str) -> Response {
    let status = match code {
        "shopping_lookup_throttled" => StatusCode::TOO_MANY_REQUESTS,
        "shopping_revision_conflict" => StatusCode::CONFLICT,
        "shopping_storage_failed" => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::UNPROCESSABLE_ENTITY,
    };
    household::reply(status, json!({"code":code}))
}
pub(crate) async fn lookup(
    State(state): State<Shared>,
    headers: HeaderMap,
    body: Result<Json<Input>, JsonRejection>,
) -> Response {
    if !household::can_write(&state, &headers) {
        return household::reply(
            StatusCode::FORBIDDEN,
            json!({"code":"household_write_required"}),
        );
    }
    let Ok(Json(input)) = body else {
        return failure("shopping_request_invalid");
    };
    if !input.consent {
        return failure("shopping_lookup_consent_required");
    }
    if gtin(&input.barcode).is_err() {
        return failure("shopping_barcode_invalid");
    }
    match perform(state, input).await {
        Ok(value) => household::reply(StatusCode::OK, value),
        Err(code) => failure(code),
    }
}
async fn perform(state: Shared, input: Input) -> Result<Value, &'static str> {
    let time = now()?;
    let previous = LAST_LOOKUP.load(Ordering::Relaxed);
    if previous > 0 && time.saturating_sub(previous) < 2 {
        return Err("shopping_lookup_throttled");
    }
    LAST_LOOKUP
        .compare_exchange(previous, time, Ordering::Relaxed, Ordering::Relaxed)
        .map_err(|_| "shopping_lookup_throttled")?;
    let reserve = state.clone();
    tokio::task::spawn_blocking(move || {
        let mut store = reserve
            .shopping
            .lock()
            .map_err(|_| "shopping_storage_failed")?;
        store.change(input.expected_revision, |data| {
            if data.products.last_lookup > 0 && time.saturating_sub(data.products.last_lookup) < 2 {
                return Err("shopping_lookup_throttled");
            }
            data.products.last_lookup = time;
            Ok(())
        })
    })
    .await
    .map_err(|_| "shopping_storage_failed")??;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .timeout(Duration::from_secs(8))
        .connect_timeout(Duration::from_secs(3))
        .user_agent("Mealshop/0.1.0 (https://github.com/Th3Link/mealshop)")
        .build()
        .map_err(|_| "shopping_lookup_unavailable")?;
    let url = format!(
        "https://world.openfoodfacts.org/api/v2/product/{}?fields=code,product_name,brands,quantity,last_modified_t",
        input.barcode
    );
    let mut response = client
        .get(url)
        .send()
        .await
        .map_err(|_| "shopping_lookup_unavailable")?;
    if response.status() != reqwest::StatusCode::OK
        && response.status() != reqwest::StatusCode::NOT_FOUND
    {
        return Err("shopping_lookup_unavailable");
    }
    if response.content_length().is_some_and(|n| n > 65_536) {
        return Err("shopping_lookup_invalid");
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "shopping_lookup_unavailable")?
    {
        if bytes.len() + chunk.len() > 65_536 {
            return Err("shopping_lookup_invalid");
        }
        bytes.extend_from_slice(&chunk);
    }
    let parsed: Value = serde_json::from_slice(&bytes).map_err(|_| "shopping_lookup_invalid")?;
    let evidence = parse(&parsed, &input.barcode, time)?;
    tokio::task::spawn_blocking(move || {
        let mut store = state
            .shopping
            .lock()
            .map_err(|_| "shopping_storage_failed")?;
        let mut candidate = None;
        if let Some(evidence) = evidence {
            let revision = store.data.revision;
            store.change(revision, |data| {
                data.products
                    .candidates
                    .retain(|c| time.saturating_sub(c.evidence.retrieved_at) <= 86400);
                if data.products.candidates.len() >= 20 {
                    data.products.candidates.remove(0);
                }
                let value = Candidate {
                    id: data.allocate("lookup")?,
                    barcode: input.barcode,
                    evidence,
                };
                data.products.candidates.push(value.clone());
                candidate = Some(value);
                Ok(())
            })?;
        }
        Ok(json!({"shopping":store.view(),"candidate":candidate}))
    })
    .await
    .map_err(|_| "shopping_storage_failed")?
}
fn parse(value: &Value, barcode: &str, time: u64) -> Result<Option<Evidence>, &'static str> {
    if value["status"] == 0 {
        return Ok(None);
    }
    if value["status"] != 1 {
        return Err("shopping_lookup_invalid");
    }
    let product = &value["product"];
    let code = product["code"].as_str().ok_or("shopping_lookup_invalid")?;
    if gtin(code).map_err(|_| "shopping_lookup_invalid")? != gtin(barcode)? {
        return Err("shopping_lookup_invalid");
    }
    let field = |key: &str| -> Result<String, &'static str> {
        match &product[key] {
            Value::Null => Ok(String::new()),
            Value::String(s) if s.len() <= 200 => Ok(s.clone()),
            _ => Err("shopping_lookup_invalid"),
        }
    };
    let modified_at = match &product["last_modified_t"] {
        Value::Null => None,
        value => Some(value.as_u64().ok_or("shopping_lookup_invalid")?),
    };
    let evidence = Evidence {
        source_url: format!("https://world.openfoodfacts.org/product/{barcode}"),
        licence: "ODbL-1.0".to_owned(),
        retrieved_at: time,
        modified_at,
        original_name: field("product_name")?,
        original_brand: field("brands")?,
        original_quantity: field("quantity")?,
    };
    if !evidence_valid(&evidence, barcode) {
        return Err("shopping_lookup_invalid");
    }
    Ok(Some(evidence))
}
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn provider_data_is_bounded_and_does_not_become_nutrition_or_a_match() {
        let value = json!({"status":1,"product":{"code":"0034000470693","product_name":"Test product","brands":"Test brand","quantity":"2 x 50 g","last_modified_t":100,"nutriments":{"salt":5}}});
        let evidence = parse(&value, "034000470693", 200).unwrap().unwrap();
        assert_eq!(evidence.original_quantity, "2 x 50 g");
        assert_eq!(evidence.licence, "ODbL-1.0");
        assert!(
            parse(&json!({"status":0}), "034000470693", 200)
                .unwrap()
                .is_none()
        );
        for field in [
            json!({"status":1,"product":{"code":"12345670","product_name":"Wrong code"}}),
            json!({"status":1,"product":{"code":"034000470693","product_name":123}}),
            json!({"status":1,"product":{"code":"034000470693","product_name":"x".repeat(201)}}),
        ] {
            assert!(parse(&field, "034000470693", 200).is_err());
        }
    }
    #[test]
    fn gtin_preserves_identity_and_rejects_bad_codes() {
        assert_eq!(
            gtin("034000470693").unwrap(),
            gtin("0034000470693").unwrap()
        );
        for code in [
            "00000000",
            "12345671",
            "123",
            "https://localhost",
            "123456789012345",
        ] {
            assert!(gtin(code).is_err());
        }
    }
}
