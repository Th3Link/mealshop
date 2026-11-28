//! Interdata: buffered intake of shared Mealshop recipes and ingredients (F43, slice 1).
//!
//! Uploads are validated for shape and size and stored unchanged. Nothing uploaded here is
//! published; checking and review are later stages.
mod model;
mod store;

use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path as Segment, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use store::{Kind, Outcome, Record, Store};
use subtle::ConstantTimeEq;

/// Largest accepted upload body in bytes.
pub const MAX_BODY_BYTES: usize = 262_144;
/// Shortest accepted upload secret in bytes.
pub const MIN_SECRET_BYTES: usize = 32;

/// Operator configuration of one service instance.
pub struct Config {
    /// Shared secret every upload presents as a bearer token (F43-ADMISSION).
    pub upload_secret: String,
    /// Directory that holds the submission buffer.
    pub data_dir: PathBuf,
    /// HTTPS origin image links must point to; image links are rejected while it is absent.
    pub cdn_origin: Option<String>,
    /// Number of buffered submissions after which uploads are refused.
    pub capacity: usize,
    /// New submissions accepted per minute across all uploaders.
    pub uploads_per_minute: u32,
}

struct Shared {
    secret: Vec<u8>,
    cdn: Option<model::CdnOrigin>,
    store: Mutex<Store>,
}

/// Builds the HTTP API.
///
/// # Errors
/// Returns an operator-facing message when the secret is too short, the CDN origin is not a
/// plain HTTPS origin or the data directory cannot be prepared.
pub fn router(config: Config) -> Result<Router, String> {
    if config.upload_secret.len() < MIN_SECRET_BYTES {
        return Err(format!(
            "The upload secret needs at least {MIN_SECRET_BYTES} bytes"
        ));
    }
    let cdn = config
        .cdn_origin
        .as_deref()
        .map(model::CdnOrigin::parse)
        .transpose()
        .map_err(|e| format!("CDN origin: {e}"))?;
    let store = Store::open(
        &config.data_dir,
        store::Limits {
            capacity: config.capacity,
            uploads_per_minute: config.uploads_per_minute,
        },
    )
    .map_err(|e| format!("Cannot prepare the data directory: {e}"))?;
    let shared = Arc::new(Shared {
        secret: config.upload_secret.into_bytes(),
        cdn,
        store: Mutex::new(store),
    });
    Ok(Router::new()
        .route("/health", get(|| async { Json(json!({"status": "ok"})) }))
        .route(
            "/v1/submissions/recipes",
            post(|s, h, b| submit(s, h, b, Kind::Recipe)),
        )
        .route(
            "/v1/submissions/ingredients",
            post(|s, h, b| submit(s, h, b, Kind::Ingredient)),
        )
        .route("/v1/submissions/{id}", get(read))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .with_state(shared))
}

fn problem(status: StatusCode, code: &str, message: &str) -> Response {
    (
        status,
        Json(json!({"error": {"code": code, "message": message}})),
    )
        .into_response()
}

fn admitted(shared: &Shared, headers: &HeaderMap) -> bool {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.as_bytes().strip_prefix(b"Bearer "))
        .is_some_and(|token| bool::from(token.ct_eq(&shared.secret)))
}

fn unauthorized() -> Response {
    let mut response = problem(
        StatusCode::UNAUTHORIZED,
        "upload_secret_required",
        "Send the upload secret as \"Authorization: Bearer <secret>\"",
    );
    response
        .headers_mut()
        .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
    response
}

fn storage(error: &store::Error) -> Response {
    match error {
        store::Error::Conflict => problem(
            StatusCode::CONFLICT,
            "idempotency_conflict",
            "This Idempotency-Key was already used for a different upload; use a new key",
        ),
        store::Error::Full => problem(
            StatusCode::INSUFFICIENT_STORAGE,
            "buffer_full",
            "The submission buffer is full until reviewers catch up; try again later",
        ),
        store::Error::Busy => {
            let mut response = problem(
                StatusCode::TOO_MANY_REQUESTS,
                "too_many_uploads",
                "Too many uploads right now; try again in a minute",
            );
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from_static("60"));
            response
        }
        store::Error::NotFound => problem(
            StatusCode::NOT_FOUND,
            "submission_not_found",
            "No submission has this id",
        ),
        store::Error::Storage => problem(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_failed",
            "The submission could not be stored or read; nothing was accepted",
        ),
    }
}

fn summary(record: &Record) -> Value {
    json!({
        "id": record.id,
        "kind": record.kind,
        "status": record.status,
        "received_at": record.received_at,
    })
}

/// Runs a store operation off the async workers; file I/O and the lock are blocking.
async fn with_store<T: Send + 'static>(
    shared: Arc<Shared>,
    operation: impl FnOnce(&mut Store) -> Result<T, store::Error> + Send + 'static,
) -> Result<T, store::Error> {
    tokio::task::spawn_blocking(move || {
        let mut store = shared.store.lock().map_err(|_| store::Error::Storage)?;
        operation(&mut store)
    })
    .await
    .unwrap_or(Err(store::Error::Storage))
}

async fn submit(
    State(shared): State<Arc<Shared>>,
    headers: HeaderMap,
    body: Bytes,
    kind: Kind,
) -> Response {
    if !admitted(&shared, &headers) {
        return unauthorized();
    }
    let json_body = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            v.split(';')
                .next()
                .is_some_and(|m| m.trim().eq_ignore_ascii_case("application/json"))
        });
    if !json_body {
        return problem(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "json_required",
            "Send the upload as application/json",
        );
    }
    let key = headers
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .filter(|k| store::valid_key(k));
    let Some(key) = key.map(str::to_owned) else {
        return problem(
            StatusCode::BAD_REQUEST,
            "idempotency_key_required",
            "Send an Idempotency-Key header of 16–128 letters, digits, hyphens or underscores, for example a UUID",
        );
    };
    let Ok(original) = String::from_utf8(body.to_vec()) else {
        return problem(
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "The body is not UTF-8",
        );
    };
    let checked = match kind {
        Kind::Recipe => model::recipe(original.as_bytes(), shared.cdn.as_ref()),
        Kind::Ingredient => model::ingredient(original.as_bytes()),
    };
    if let Err(invalid) = checked {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            invalid.code,
            &invalid.message,
        );
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    match with_store(shared, move |store| {
        store.submit(kind, &key, &original, now)
    })
    .await
    {
        Ok(Outcome::Created(record)) => {
            (StatusCode::CREATED, Json(summary(&record))).into_response()
        }
        Ok(Outcome::Replayed(record)) => Json(summary(&record)).into_response(),
        Err(error) => storage(&error),
    }
}

async fn read(
    State(shared): State<Arc<Shared>>,
    headers: HeaderMap,
    Segment(id): Segment<String>,
) -> Response {
    if !admitted(&shared, &headers) {
        return unauthorized();
    }
    match with_store(shared, move |store| store.get(&id)).await {
        Ok(record) => {
            let Ok(content) = serde_json::from_str::<Value>(&record.original) else {
                return storage(&store::Error::Storage);
            };
            let mut body = summary(&record);
            body["content"] = content;
            Json(body).into_response()
        }
        Err(error) => storage(&error),
    }
}
