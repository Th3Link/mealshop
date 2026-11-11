//! Verify canonical recipient IDs against the documented `AppAPI` user utility, without retention.
use super::{Failure, Shared, failure};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Deserialize;

#[derive(Deserialize)]
struct Envelope {
    ocs: Users,
}
#[derive(Deserialize)]
struct Users {
    meta: Meta,
    data: Vec<String>,
}
#[derive(Deserialize)]
struct Meta {
    statuscode: u16,
}

pub(super) async fn verify(app: Shared, actor: &str, recipient: &str) -> Result<(), Failure> {
    if !crate::membership::valid_user(recipient) {
        return Err(failure("invalid_share_recipient"));
    }
    let mut reply = app
        .client
        .get(format!(
            "{}/ocs/v1.php/apps/app_api/api/v1/users?format=json",
            app.config.nextcloud_url.trim_end_matches('/')
        ))
        .header("EX-APP-ID", "mealshop")
        .header("EX-APP-VERSION", env!("CARGO_PKG_VERSION"))
        .header(
            "AUTHORIZATION-APP-API",
            STANDARD.encode(format!("{actor}:{}", app.config.secret)),
        )
        .header("OCS-APIRequest", "true")
        .send()
        .await
        .map_err(|_| failure("share_user_lookup_failed"))?;
    if !reply.status().is_success() {
        return Err(failure("share_user_lookup_failed"));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = reply
        .chunk()
        .await
        .map_err(|_| failure("share_user_lookup_failed"))?
    {
        if bytes.len() + chunk.len() > 2_097_152 {
            return Err(failure("share_user_lookup_failed"));
        }
        bytes.extend_from_slice(&chunk);
    }
    let result: Envelope =
        serde_json::from_slice(&bytes).map_err(|_| failure("share_user_lookup_failed"))?;
    if !matches!(result.ocs.meta.statuscode, 100 | 200) {
        return Err(failure("share_user_lookup_failed"));
    }
    if result.ocs.data.iter().any(|id| id == recipient) {
        Ok(())
    } else {
        Err(failure("share_recipient_not_found"))
    }
}
