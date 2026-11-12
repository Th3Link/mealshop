use super::{
    model::{Candidate, Method, Receipt, Source},
    store::PreviewData,
};
use crate::{
    household::{self, Authority, Shared},
    recipe_origin::Origin,
    recipes::{Creation, Recipe},
};
use axum::{
    Extension, Json, Router,
    extract::{State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::post,
};
use serde::Deserialize;
use serde_json::json;
use std::time::Instant;
static FETCHES: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(4);
pub(crate) fn routes() -> Router<Shared> {
    Router::new()
        .route("/api/household/imports/preview", post(preview))
        .route("/api/household/imports", post(save))
}
fn failure(code: &'static str) -> Response {
    let status = match code {
        "import_write_required" | "import_review_required" => StatusCode::FORBIDDEN,
        "import_preview_expired" | "import_preview_unavailable" | "import_request_conflict" => {
            StatusCode::CONFLICT
        }
        "import_fetch_cooldown" | "import_fetch_busy" | "import_preview_capacity" => {
            StatusCode::TOO_MANY_REQUESTS
        }
        "import_storage_failed" => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::UNPROCESSABLE_ENTITY,
    };
    household::reply(status, json!({"code":code}))
}
fn now() -> Result<i64, &'static str> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|v| i64::try_from(v.as_secs()).ok())
        .ok_or("import_clock_invalid")
}
fn body<T>(value: Result<Json<T>, JsonRejection>) -> Result<T, StatusCode> {
    value.map(|Json(value)| value).map_err(|e| e.status())
}
fn writer(state: &crate::household::Household, headers: &HeaderMap, actor: &Authority) -> bool {
    household::can_write(state, headers) && actor.role != crate::membership::Role::Reader
}
#[derive(Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
enum Input {
    Url { url: String, consent: bool },
    Manual { url: String },
}
struct FetchGuard {
    state: Shared,
    actor: String,
}
impl Drop for FetchGuard {
    fn drop(&mut self) {
        if let Ok(mut store) = self.state.imports.lock() {
            store.finish(&self.actor);
        }
    }
}
async fn source(state: &Shared, actor: &str, input: Input) -> Result<PreviewData, &'static str> {
    match input {
        Input::Manual { url } => {
            Origin::Imported {
                source_url: url.clone(),
            }
            .validate()
            .map_err(|_| "import_url_invalid")?;
            Ok(PreviewData {
                requested_url: url,
                final_url: None,
                method: Method::Manual,
                retrieved_at: None,
                candidates: vec![Candidate::default()],
            })
        }
        Input::Url { url, consent } => {
            if !consent {
                return Err("import_consent_required");
            }
            super::network::url(&url)?;
            let permit = FETCHES.try_acquire().map_err(|_| "import_fetch_busy")?;
            state
                .imports
                .lock()
                .map_err(|_| "import_storage_failed")?
                .begin(actor)?;
            let _active = FetchGuard {
                state: state.clone(),
                actor: actor.to_owned(),
            };
            let (final_url, document) =
                super::network::fetch(&super::network::PublicNetwork, &url).await?;
            let processing = super::PROCESSING
                .try_acquire()
                .map_err(|_| "import_fetch_busy")?;
            let candidates = tokio::task::spawn_blocking(move || {
                let _permit = permit;
                let _processing = processing;
                super::parser::extract(&document)
            })
            .await
            .map_err(|_| "import_parser_failed")??;
            Ok(PreviewData {
                requested_url: url,
                final_url: Some(final_url),
                method: Method::JsonLd,
                retrieved_at: Some(now()?),
                candidates,
            })
        }
    }
}
async fn preview(
    State(state): State<Shared>,
    headers: HeaderMap,
    who: Option<Extension<Authority>>,
    input: Result<Json<Input>, JsonRejection>,
) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    let actor = household::authority(&state, &headers, who);
    if !writer(&state, &headers, &actor) {
        return failure("import_write_required");
    }
    let input = match body(input) {
        Ok(v) => v,
        Err(status) => return household::reply(status, json!({"code":"import_invalid_input"})),
    };
    super::maintain(&state);
    let result = async {
        let data = source(&state, &actor.user, input).await?;
        let expires_at = now()?.checked_add(1800).ok_or("import_clock_invalid")?;
        let id = state
            .imports
            .lock()
            .map_err(|_| "import_storage_failed")?
            .stage(actor.user, data.clone())?;
        let mut result = serde_json::to_value(data).map_err(|_| "import_storage_failed")?;
        result["preview_id"] = json!(id);
        result["expires_at"] = json!(expires_at);
        Ok::<_, &'static str>(result)
    }
    .await;
    match result {
        Ok(value) => household::reply(StatusCode::OK, value),
        Err(code) => failure(code),
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Save {
    preview_id: String,
    candidate: usize,
    idempotency_key: String,
    reviewed: bool,
    recipe: Recipe,
}
fn storage(error: &crate::recipes::Error) -> &'static str {
    match error {
        crate::recipes::Error::Invalid(_) => "import_recipe_invalid",
        crate::recipes::Error::Conflict => "import_request_conflict",
        _ => "import_storage_failed",
    }
}
async fn save(
    State(state): State<Shared>,
    headers: HeaderMap,
    who: Option<Extension<Authority>>,
    input: Result<Json<Save>, JsonRejection>,
) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    let actor = household::authority(&state, &headers, who);
    if !writer(&state, &headers, &actor) {
        return failure("import_write_required");
    }
    let input = match body(input) {
        Ok(v) => v,
        Err(status) => return household::reply(status, json!({"code":"import_invalid_input"})),
    };
    if !input.reviewed {
        return failure("import_review_required");
    }
    if !Creation::valid_key(&input.idempotency_key)
        || input.preview_id.len() != 71
        || !input.preview_id.starts_with("import-")
        || input.candidate >= 16
    {
        return failure("import_invalid_input");
    }
    let result = tokio::task::spawn_blocking(move || {
        let mut recipes = state
            .recipes
            .as_ref()
            .ok_or("import_storage_failed")?
            .lock()
            .map_err(|_| "import_storage_failed")?;
        if let Some(record) = recipes
            .import_result(
                &actor.user,
                &input.idempotency_key,
                &input.preview_id,
                input.candidate,
                &input.recipe,
            )
            .map_err(|e| storage(&e))?
        {
            return Ok(record);
        }
        let catalogue = state
            .catalogue
            .lock()
            .map_err(|_| "import_storage_failed")?;
        let mut imports = state.imports.lock().map_err(|_| "import_storage_failed")?;
        let preview = imports
            .previews
            .get(&input.preview_id)
            .ok_or("import_preview_expired")?;
        if preview.actor != actor.user {
            return Err("import_preview_unavailable");
        }
        if preview.deadline <= Instant::now() {
            return Err("import_preview_expired");
        }
        let original = preview
            .data
            .candidates
            .get(input.candidate)
            .ok_or("import_invalid_input")?
            .clone();
        let source = Source {
            requested_url: preview.data.requested_url.clone(),
            final_url: preview.data.final_url.clone(),
            method: preview.data.method,
            retrieved_at: preview.data.retrieved_at,
            importer_version: "schema-recipe-1".to_owned(),
            original,
        };
        crate::recipe_images::validate_reference(
            Some(&state.images),
            input.recipe.image_id.as_deref(),
        )
        .map_err(|e| storage(&e))?;
        let receipt = Receipt {
            actor: actor.user,
            key: input.idempotency_key,
            preview_id: input.preview_id.clone(),
            candidate: input.candidate,
            recorded_at: now()?,
            source,
        };
        let record = recipes
            .save_imported(input.recipe, &catalogue.catalogue(), receipt)
            .map_err(|e| storage(&e))?;
        imports.previews.remove(&input.preview_id);
        Ok::<_, &'static str>(record)
    })
    .await;
    match result {
        Ok(Ok(record)) => crate::recipe_api::saved_response(&record, StatusCode::CREATED),
        Ok(Err(code)) => failure(code),
        Err(_) => failure("import_storage_failed"),
    }
}
