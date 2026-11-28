//! Speech and picture transcription through Nextcloud Task Processing, on behalf of the user.
//! Mealshop holds no model and no key: the instance administrator chooses the provider. A
//! recording or picture is placed in the user's files only while its task runs, because file
//! inputs are Nextcloud file IDs. Nothing here is logged, and the text returned is unreviewed
//! source text for the capture screen, never a recipe.
use super::{App, Failure, Shared, reply, user};
use axum::{
    Json,
    extract::{Path, Query, Request, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{fmt::Write as _, io::Read, time::Duration};

const FOLDER: &str = ".mealshop-capture";
const MAX_PENDING: usize = 50;
const MAX_AUDIO: usize = 26_214_400;
const MAX_PICTURE: usize = 15_728_640;
const MAX_TEXT: usize = 32_000;
/// Asks a vision model for the words only; the deterministic reader does the structuring.
const HANDWRITING: &str = "Transcribe all text in this picture exactly as written, line by line, in its original language. Do not translate, summarise, correct or add anything. Output only the text.";

#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum Kind {
    Speech,
    Picture,
    Handwriting,
}
impl Kind {
    const ALL: [Self; 3] = [Self::Speech, Self::Picture, Self::Handwriting];
    fn name(self) -> &'static str {
        match self {
            Self::Speech => "speech",
            Self::Picture => "picture",
            Self::Handwriting => "handwriting",
        }
    }
    fn task_type(self) -> &'static str {
        match self {
            Self::Speech => "core:audio2text",
            Self::Picture => "core:image2text:ocr",
            Self::Handwriting => "core:analyze-images",
        }
    }
    fn extension(self, mime: &str) -> Option<&'static str> {
        match (self, mime) {
            (Self::Speech, "audio/webm") => Some("webm"),
            (Self::Speech, "audio/ogg") => Some("ogg"),
            (Self::Speech, "audio/mp4") => Some("m4a"),
            (Self::Speech, "audio/mpeg") => Some("mp3"),
            (Self::Speech, "audio/wav" | "audio/x-wav") => Some("wav"),
            (Self::Picture | Self::Handwriting, "image/jpeg") => Some("jpg"),
            (Self::Picture | Self::Handwriting, "image/png") => Some("png"),
            (Self::Picture | Self::Handwriting, "image/webp") => Some("webp"),
            _ => None,
        }
    }
    fn input(self, file: u64) -> Value {
        match self {
            Self::Speech => json!({"input":file}),
            Self::Picture => json!({"input":[file]}),
            Self::Handwriting => json!({"images":[file],"input":HANDWRITING}),
        }
    }
    fn text(output: &Value) -> Option<String> {
        let value = output.get("output")?;
        match value {
            Value::String(text) => Some(text.clone()),
            Value::Array(parts) => Some(
                parts
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            _ => None,
        }
    }
}

/// A scheduled task and the temporary file that has to be removed when it ends.
pub(super) struct Pending {
    user: String,
    kind: Kind,
    file: String,
}

fn nextcloud(app: &App) -> Result<&str, Failure> {
    app.config
        .nextcloud()
        .map(|host| host.url.trim_end_matches('/'))
        .ok_or(Failure(StatusCode::CONFLICT, "assist_unavailable"))
}
fn signed(app: &App, request: reqwest::RequestBuilder, actor: &str) -> reqwest::RequestBuilder {
    request
        .header("EX-APP-ID", "mealshop")
        .header("EX-APP-VERSION", env!("CARGO_PKG_VERSION"))
        .header(
            "AUTHORIZATION-APP-API",
            STANDARD.encode(format!("{actor}:{}", app.config.secret)),
        )
        .header("OCS-APIRequest", "true")
}
/// The data object of a successful OCS reply, bounded in size.
async fn ocs(mut reply: reqwest::Response) -> Option<Value> {
    if !reply.status().is_success() {
        return None;
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = reply.chunk().await.ok()? {
        if bytes.len() + chunk.len() > 2_097_152 {
            return None;
        }
        bytes.extend_from_slice(&chunk);
    }
    let mut body: Value = serde_json::from_slice(&bytes).ok()?;
    Some(body.pointer_mut("/ocs/data")?.take())
}
/// Task types that currently have a provider on this instance.
async fn offered(app: &App, actor: &str) -> Vec<Kind> {
    let Ok(base) = nextcloud(app) else {
        return Vec::new();
    };
    let reply = signed(
        app,
        app.client
            .get(format!(
                "{base}/ocs/v2.php/taskprocessing/tasktypes?format=json"
            ))
            .timeout(Duration::from_secs(10)),
        actor,
    )
    .send()
    .await;
    let Ok(reply) = reply else {
        return Vec::new();
    };
    let Some(data) = ocs(reply).await else {
        return Vec::new();
    };
    Kind::ALL
        .into_iter()
        .filter(|kind| {
            data.get("types")
                .and_then(Value::as_object)
                .is_some_and(|types| types.contains_key(kind.task_type()))
        })
        .collect()
}
pub(super) async fn capabilities(State(app): State<Shared>, request: Request) -> Response {
    let actor = match user(&app, &request) {
        Ok(actor) => actor,
        Err(failure) => return failure.into_response(),
    };
    let kinds = offered(&app, &actor).await;
    let has = |kind| kinds.contains(&kind);
    (
        [(header::CACHE_CONTROL, "no-store")],
        Json(json!({"speech":has(Kind::Speech),"picture":has(Kind::Picture),"handwriting":has(Kind::Handwriting)})),
    )
        .into_response()
}

async fn discard(app: &App, actor: &str, file: &str) {
    if let Ok(base) = nextcloud(app) {
        let _ = signed(
            app,
            app.client
                .delete(format!(
                    "{base}/remote.php/dav/files/{actor}/{FOLDER}/{file}"
                ))
                .timeout(Duration::from_secs(10)),
            actor,
        )
        .send()
        .await;
    }
}
fn random_name() -> Option<String> {
    let mut bytes = [0u8; 16];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut source| source.read_exact(&mut bytes))
        .ok()?;
    let mut name = String::with_capacity(32);
    for byte in bytes {
        write!(&mut name, "{byte:02x}").ok()?;
    }
    Some(name)
}

#[derive(Deserialize)]
pub(super) struct Start {
    kind: Kind,
}
pub(super) async fn start(
    State(app): State<Shared>,
    Query(query): Query<Start>,
    request: Request,
) -> Response {
    match begin(&app, query.kind, request).await {
        Ok(id) => (
            StatusCode::ACCEPTED,
            [(header::CACHE_CONTROL, "no-store")],
            Json(json!({"id":id,"kind":query.kind.name(),"status":"waiting"})),
        )
            .into_response(),
        Err(failure) => failure.into_response(),
    }
}
/// Places the input in the user's files and returns its name and Nextcloud file ID.
async fn store(
    app: &Shared,
    base: &str,
    actor: &str,
    extension: &str,
    mime: String,
    body: axum::body::Bytes,
) -> Result<(String, u64), Failure> {
    let file = format!(
        "{}.{extension}",
        random_name().ok_or(Failure(StatusCode::SERVICE_UNAVAILABLE, "assist_failed"))?
    );
    let upload = Failure(StatusCode::BAD_GATEWAY, "assist_upload_failed");
    let folder = format!("{base}/remote.php/dav/files/{actor}/{FOLDER}");
    // The folder usually exists already; only the upload decides whether this worked.
    let _ = signed(
        app,
        app.client
            .request(
                reqwest::Method::from_bytes(b"MKCOL").map_err(|_| Failure(upload.0, upload.1))?,
                &folder,
            )
            .timeout(Duration::from_secs(10)),
        actor,
    )
    .send()
    .await;
    let stored = signed(
        app,
        app.client
            .put(format!("{folder}/{file}"))
            .timeout(Duration::from_secs(120))
            .header(header::CONTENT_TYPE, mime)
            .body(body),
        actor,
    )
    .send()
    .await
    .map_err(|_| Failure(upload.0, upload.1))?;
    // `OC-FileId` is the zero-padded numeric file ID followed by the instance ID.
    let id = stored
        .status()
        .is_success()
        .then(|| stored.headers().get("oc-fileid")?.to_str().ok())
        .flatten()
        .and_then(|value| {
            value
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse::<u64>()
                .ok()
        });
    let Some(id) = id else {
        discard(app, actor, &file).await;
        return Err(upload);
    };
    Ok((file, id))
}
async fn begin(app: &Shared, kind: Kind, request: Request) -> Result<u64, Failure> {
    let actor = user(app, &request)?;
    let base = nextcloud(app)?.to_owned();
    let mime = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(|value| value.trim().to_ascii_lowercase())
        .unwrap_or_default();
    let extension = kind.extension(&mime).ok_or(Failure(
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "assist_media_unsupported",
    ))?;
    let limit = if kind == Kind::Speech {
        MAX_AUDIO
    } else {
        MAX_PICTURE
    };
    let body = axum::body::to_bytes(request.into_body(), limit)
        .await
        .map_err(|_| Failure(StatusCode::PAYLOAD_TOO_LARGE, "assist_too_large"))?;
    if body.is_empty() {
        return Err(Failure(
            StatusCode::UNPROCESSABLE_ENTITY,
            "assist_media_unsupported",
        ));
    }
    if app
        .assist
        .lock()
        .map_err(|_| Failure(StatusCode::SERVICE_UNAVAILABLE, "assist_failed"))?
        .len()
        >= MAX_PENDING
    {
        return Err(Failure(StatusCode::TOO_MANY_REQUESTS, "assist_busy"));
    }
    if !offered(app, &actor).await.contains(&kind) {
        return Err(Failure(StatusCode::CONFLICT, "assist_unavailable"));
    }
    let (file, id) = store(app, &base, &actor, extension, mime, body).await?;
    let scheduled = signed(
        app,
        app.client
            .post(format!("{base}/ocs/v2.php/taskprocessing/schedule?format=json"))
            .timeout(Duration::from_secs(20))
            .json(&json!({"type":kind.task_type(),"appId":"mealshop","customId":file,"input":kind.input(id)})),
        &actor,
    )
    .send()
    .await;
    let task = match scheduled {
        Ok(reply) => ocs(reply)
            .await
            .and_then(|data| data.pointer("/task/id")?.as_u64()),
        Err(_) => None,
    };
    let Some(task) = task else {
        discard(app, &actor, &file).await;
        return Err(Failure(StatusCode::BAD_GATEWAY, "assist_schedule_failed"));
    };
    // The guard must not live across the cleanup below.
    let entry = Pending {
        user: actor.clone(),
        kind,
        file: file.clone(),
    };
    let kept = app
        .assist
        .lock()
        .map(|mut pending| {
            pending.insert(task, entry);
        })
        .is_ok();
    if kept {
        Ok(task)
    } else {
        discard(app, &actor, &file).await;
        Err(Failure(StatusCode::SERVICE_UNAVAILABLE, "assist_failed"))
    }
}

pub(super) async fn status(
    State(app): State<Shared>,
    Path(id): Path<u64>,
    request: Request,
) -> Response {
    let actor = match user(&app, &request) {
        Ok(actor) => actor,
        Err(failure) => return failure.into_response(),
    };
    // Another account's task is indistinguishable from one that never existed.
    let known = app.assist.lock().ok().and_then(|pending| {
        pending
            .get(&id)
            .filter(|entry| entry.user == actor)
            .map(|entry| (entry.kind, entry.file.clone()))
    });
    let (Some((kind, file)), Ok(base)) = (known, nextcloud(&app)) else {
        return reply(StatusCode::NOT_FOUND, "assist_not_found");
    };
    let fetched = signed(
        &app,
        app.client
            .get(format!(
                "{base}/ocs/v2.php/taskprocessing/task/{id}?format=json"
            ))
            .timeout(Duration::from_secs(10)),
        &actor,
    )
    .send()
    .await;
    let task = match fetched {
        Ok(reply) => ocs(reply)
            .await
            .and_then(|mut data| Some(data.get_mut("task")?.take())),
        Err(_) => None,
    };
    let Some(task) = task else {
        // A lookup that failed says nothing about the task; the client asks again.
        return reply(StatusCode::BAD_GATEWAY, "assist_lookup_failed");
    };
    let body = match task.get("status").and_then(Value::as_str) {
        Some("STATUS_SUCCESSFUL") => {
            let mut text = task.get("output").and_then(Kind::text).unwrap_or_default();
            if text.len() > MAX_TEXT {
                let mut end = MAX_TEXT;
                while !text.is_char_boundary(end) {
                    end -= 1;
                }
                text.truncate(end);
            }
            json!({"id":id,"kind":kind.name(),"status":"done","text":text})
        }
        Some("STATUS_FAILED" | "STATUS_CANCELLED") => {
            json!({"id":id,"kind":kind.name(),"status":"failed"})
        }
        Some("STATUS_RUNNING") => {
            json!({"id":id,"kind":kind.name(),"status":"running","progress":task.get("progress")})
        }
        _ => json!({"id":id,"kind":kind.name(),"status":"waiting"}),
    };
    if matches!(body["status"].as_str(), Some("done" | "failed")) {
        if let Ok(mut pending) = app.assist.lock() {
            pending.remove(&id);
        }
        discard(&app, &actor, &file).await;
    }
    ([(header::CACHE_CONTROL, "no-store")], Json(body)).into_response()
}
