//! Opt-in Nextcloud `AppAPI` transport, lifecycle and household authorization boundary.
use crate::membership::{Membership, Role};
use axum::{
    Json, Router,
    extract::{Path, Query, Request, State},
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use subtle::ConstantTimeEq;
use tower::ServiceExt;

/// Configuration supplied by the `AppAPI` deploy daemon or local test operator.
pub struct Config {
    /// AppAPI-assigned shared secret. Never sent to the browser.
    pub secret: String,
    /// Trusted server-side Nextcloud base URL, including any installation prefix.
    pub nextcloud_url: String,
    /// Dedicated persistent directory; never an existing local single-household store.
    pub directory: PathBuf,
    /// Directory containing the built Vite IIFE and stylesheet.
    pub assets: PathBuf,
}
struct App {
    config: Config,
    membership: Mutex<Membership>,
    routers: Mutex<BTreeMap<String, Router>>,
    enabled: AtomicBool,
    catalogue: Router,
    client: reqwest::Client,
}
type Shared = Arc<App>;
struct Failure(StatusCode, &'static str);
impl IntoResponse for Failure {
    fn into_response(self) -> Response {
        reply(self.0, self.1)
    }
}
fn reply(status: StatusCode, code: &str) -> Response {
    (
        status,
        [(header::CACHE_CONTROL, "no-store")],
        Json(json!({"code":code})),
    )
        .into_response()
}
fn authenticated(app: &App, request: &Request) -> Option<String> {
    if request.headers().get("ex-app-id")?.to_str().ok()? != "mealshop" {
        return None;
    }
    let encoded = request
        .headers()
        .get("authorization-app-api")?
        .to_str()
        .ok()?;
    if encoded.len() > 2000 {
        return None;
    }
    let decoded = String::from_utf8(STANDARD.decode(encoded).ok()?).ok()?;
    let (user, secret) = decoded.split_once(':')?;
    if !bool::from(secret.as_bytes().ct_eq(app.config.secret.as_bytes()))
        || user.len() > 255
        || user.chars().any(char::is_control)
    {
        return None;
    }
    Some(user.to_owned())
}
/// Builds the isolated `AppAPI` application. Local demo and unowned APIs are not published.
/// # Errors
/// Returns an actionable configuration or storage error; a second writer is rejected.
pub async fn router(config: Config) -> Result<Router, String> {
    if config.secret.len() < 32
        || config.secret.len() > 200
        || !config
            .secret
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(
            "APP_SECRET must contain 32–200 ASCII letters, digits, underscores or hyphens"
                .to_owned(),
        );
    }
    let url =
        reqwest::Url::parse(&config.nextcloud_url).map_err(|_| "NEXTCLOUD_URL must be absolute")?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("NEXTCLOUD_URL must be a trusted HTTP(S) server URL without credentials, query or fragment".to_owned());
    }
    for file in ["mealshop.js", "mealshop.css"] {
        if !config.assets.join(file).is_file() {
            return Err(format!(
                "Missing {file}; build clients/nextcloud before starting AppAPI mode"
            ));
        }
    }
    let membership = Membership::open(&config.directory)?;
    let app = Arc::new(App {
        config,
        membership: Mutex::new(membership),
        routers: Mutex::new(BTreeMap::new()),
        enabled: AtomicBool::new(false),
        catalogue: crate::router()?,
        client: reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|_| "Cannot initialize Nextcloud HTTP client")?,
    });
    let state_response = app
        .client
        .get(format!(
            "{}/ocs/v1.php/apps/app_api/ex-app/state?format=json",
            app.config.nextcloud_url.trim_end_matches('/')
        ))
        .header("EX-APP-ID", "mealshop")
        .header("EX-APP-VERSION", env!("CARGO_PKG_VERSION"))
        .header(
            "AUTHORIZATION-APP-API",
            STANDARD.encode(format!(":{}", app.config.secret)),
        )
        .header("OCS-APIRequest", "true")
        .send()
        .await
        .map_err(|_| "Cannot reach NEXTCLOUD_URL to restore ExApp lifecycle state")?;
    if state_response.status().is_success() {
        let state: Value = state_response
            .json()
            .await
            .map_err(|_| "Invalid AppAPI state response")?;
        if state
            .pointer("/ocs/data")
            .is_some_and(|v| v == true || v == 1)
        {
            app.enabled.store(true, Ordering::Release);
        }
    } else if !matches!(state_response.status().as_u16(), 401 | 404) {
        return Err(
            "Cannot restore AppAPI state; check Nextcloud availability before restarting"
                .to_owned(),
        );
    }
    Ok(Router::new()
        .route("/heartbeat", get(|| async { Json(json!({"status":"ok"})) }))
        .route("/enabled", put(enabled))
        .route("/api/households", get(list).post(create))
        .route("/api/households/{id}", get(detail))
        .route("/api/households/{id}/invitations", post(invite))
        .route("/api/households/{id}/invitation", put(accept))
        .route(
            "/api/households/{id}/members/{user}",
            put(change).delete(remove),
        )
        .fallback(dispatch)
        .layer(axum::extract::DefaultBodyLimit::max(131_072))
        .with_state(app))
}
async fn ocs(app: &App, path: &str, body: Value) -> Result<(), ()> {
    let response = app
        .client
        .post(format!(
            "{}/ocs/v1.php/apps/app_api/{path}?format=json",
            app.config.nextcloud_url.trim_end_matches('/')
        ))
        .header("EX-APP-ID", "mealshop")
        .header("EX-APP-VERSION", env!("CARGO_PKG_VERSION"))
        .header(
            "AUTHORIZATION-APP-API",
            STANDARD.encode(format!(":{}", app.config.secret)),
        )
        .header("OCS-APIRequest", "true")
        .json(&body)
        .send()
        .await
        .map_err(|_| ())?;
    if !response.status().is_success() {
        return Err(());
    }
    let body: Value = response.json().await.map_err(|_| ())?;
    if body
        .pointer("/ocs/meta/statuscode")
        .and_then(Value::as_u64)
        .is_some_and(|s| s == 100 || s == 200)
    {
        Ok(())
    } else {
        Err(())
    }
}
#[derive(Deserialize)]
struct Enabled {
    enabled: u8,
}
async fn enabled(
    State(app): State<Shared>,
    Query(query): Query<Enabled>,
    request: Request,
) -> Response {
    if authenticated(&app, &request).is_none() {
        return reply(StatusCode::UNAUTHORIZED, "appapi_auth_required");
    }
    if query.enabled > 1 {
        return reply(StatusCode::BAD_REQUEST, "invalid_enabled_state");
    }
    app.enabled.store(false, Ordering::Release);
    if query.enabled == 1 {
        for (path, body) in [
            (
                "api/v1/ui/top-menu",
                json!({"name":"main","displayName":"Mealshop","icon":"img/app.svg","adminRequired":0}),
            ),
            (
                "api/v1/ui/script",
                json!({"type":"top_menu","name":"main","path":"js/mealshop"}),
            ),
            (
                "api/v1/ui/style",
                json!({"type":"top_menu","name":"main","path":"css/mealshop"}),
            ),
        ] {
            if ocs(&app, path, body).await.is_err() {
                return (StatusCode::BAD_GATEWAY, Json(json!({"error":"Cannot register Mealshop UI with Nextcloud; verify NEXTCLOUD_URL and AppAPI credentials"}))).into_response();
            }
        }
        app.enabled.store(true, Ordering::Release);
    }
    Json(json!({"error":""})).into_response()
}
fn browser_write_allowed(request: &Request) -> bool {
    matches!(
        *request.method(),
        axum::http::Method::GET | axum::http::Method::HEAD
    ) || request
        .headers()
        .get("x-mealshop-request")
        .is_some_and(|value| value == "1")
}
fn user(app: &App, request: &Request) -> Result<String, Failure> {
    let user = authenticated(app, request)
        .filter(|u| !u.is_empty())
        .ok_or(Failure(StatusCode::UNAUTHORIZED, "appapi_user_required"))?;
    if !browser_write_allowed(request) {
        return Err(Failure(StatusCode::FORBIDDEN, "csrf_check_failed"));
    }
    if !app.enabled.load(Ordering::Acquire) {
        return Err(Failure(StatusCode::SERVICE_UNAVAILABLE, "app_disabled"));
    }
    Ok(user)
}
async fn registry(
    app: Shared,
    action: impl FnOnce(&mut Membership) -> Result<Value, &'static str> + Send + 'static,
) -> Response {
    match tokio::task::spawn_blocking(move || {
        let mut registry = app
            .membership
            .lock()
            .map_err(|_| "membership_storage_failed")?;
        action(&mut registry)
    })
    .await
    {
        Ok(Ok(value)) => ([(header::CACHE_CONTROL, "no-store")], Json(value)).into_response(),
        Ok(Err(code)) => reply(
            if code.ends_with("storage_failed") {
                StatusCode::SERVICE_UNAVAILABLE
            } else if code == "household_access_denied" {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::UNPROCESSABLE_ENTITY
            },
            code,
        ),
        Err(_) => reply(StatusCode::SERVICE_UNAVAILABLE, "membership_storage_failed"),
    }
}
async fn list(State(app): State<Shared>, request: Request) -> Response {
    let actor = match user(&app, &request) {
        Ok(u) => u,
        Err(r) => return r.into_response(),
    };
    registry(app, move |s| Ok(s.list(&actor))).await
}
async fn input<T: serde::de::DeserializeOwned>(request: Request) -> Result<T, Failure> {
    if !request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .is_some_and(|h| {
            h.split(';')
                .next()
                .is_some_and(|mime| mime.trim().eq_ignore_ascii_case("application/json"))
        })
    {
        return Err(Failure(StatusCode::UNSUPPORTED_MEDIA_TYPE, "json_required"));
    }

    let bytes = axum::body::to_bytes(request.into_body(), 131_072)
        .await
        .map_err(|_| Failure(StatusCode::PAYLOAD_TOO_LARGE, "request_too_large"))?;
    serde_json::from_slice(&bytes).map_err(|_| Failure(StatusCode::BAD_REQUEST, "invalid_request"))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Create {
    name: String,
}
async fn create(State(app): State<Shared>, request: Request) -> Response {
    let actor = match user(&app, &request) {
        Ok(u) => u,
        Err(r) => return r.into_response(),
    };
    let body: Create = match input(request).await {
        Ok(b) => b,
        Err(r) => return r.into_response(),
    };
    registry(app, move |s| {
        s.create(&actor, &body.name).map(|id| json!({"id":id}))
    })
    .await
}
async fn detail(State(app): State<Shared>, Path(id): Path<String>, request: Request) -> Response {
    let actor = match user(&app, &request) {
        Ok(u) => u,
        Err(r) => return r.into_response(),
    };
    registry(app, move |s| s.detail(&id, &actor)).await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Invite {
    user: String,
    role: Role,
}
async fn invite(State(app): State<Shared>, Path(id): Path<String>, request: Request) -> Response {
    let actor = match user(&app, &request) {
        Ok(u) => u,
        Err(r) => return r.into_response(),
    };
    let body: Invite = match input(request).await {
        Ok(b) => b,
        Err(r) => return r.into_response(),
    };
    registry(app, move |s| {
        s.invite(&id, &actor, &body.user, body.role)
            .map(|()| json!({}))
    })
    .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Accept {
    accept: bool,
}
async fn accept(State(app): State<Shared>, Path(id): Path<String>, request: Request) -> Response {
    let actor = match user(&app, &request) {
        Ok(u) => u,
        Err(r) => return r.into_response(),
    };
    let body: Accept = match input(request).await {
        Ok(b) => b,
        Err(r) => return r.into_response(),
    };
    registry(app, move |s| {
        s.accept(&id, &actor, body.accept).map(|()| json!({}))
    })
    .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Change {
    role: Role,
}
async fn change(
    State(app): State<Shared>,
    Path((id, target)): Path<(String, String)>,
    request: Request,
) -> Response {
    let actor = match user(&app, &request) {
        Ok(u) => u,
        Err(r) => return r.into_response(),
    };
    let body: Change = match input(request).await {
        Ok(b) => b,
        Err(r) => return r.into_response(),
    };
    registry(app, move |s| {
        s.change(&id, &actor, &target, Some(body.role))
            .map(|()| json!({}))
    })
    .await
}
async fn remove(
    State(app): State<Shared>,
    Path((id, target)): Path<(String, String)>,
    request: Request,
) -> Response {
    let actor = match user(&app, &request) {
        Ok(u) => u,
        Err(r) => return r.into_response(),
    };
    registry(app, move |s| {
        s.change(&id, &actor, &target, None).map(|()| json!({}))
    })
    .await
}
fn read_action(method: &axum::http::Method, path: &str) -> bool {
    method == axum::http::Method::GET
        || method == axum::http::Method::HEAD
        || (method == axum::http::Method::POST
            && matches!(
                path,
                "/api/household/evaluate"
                    | "/api/household/mealplan/preview"
                    | "/api/household/mealplan/remove-preview"
                    | "/api/household/mealplan/suggestions"
                    | "/api/nutrition/calculate"
                    | "/api/quantities/convert"
            ))
}
async fn dispatch(State(app): State<Shared>, request: Request) -> Response {
    let Some(actor) = authenticated(&app, &request) else {
        return reply(StatusCode::UNAUTHORIZED, "appapi_auth_required");
    };
    let path = request.uri().path().to_owned();
    if path == "/init" {
        return StatusCode::NOT_FOUND.into_response();
    }
    let asset = match path.as_str() {
        "/js/mealshop.js" => Some(("mealshop.js", "text/javascript")),
        "/css/mealshop.css" => Some(("mealshop.css", "text/css")),
        _ => None,
    };
    if let Some((file, mime)) = asset {
        return match std::fs::read(app.config.assets.join(file)) {
            Ok(body) => (
                [
                    (header::CONTENT_TYPE, mime),
                    (header::CACHE_CONTROL, "no-cache"),
                    (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
                ],
                body,
            )
                .into_response(),
            Err(_) => reply(StatusCode::SERVICE_UNAVAILABLE, "assets_unavailable"),
        };
    }
    if path == "/img/app.svg" {
        return (
            [(header::CONTENT_TYPE, "image/svg+xml")],
            include_str!("../../appinfo/app.svg"),
        )
            .into_response();
    }
    if actor.is_empty() {
        return reply(StatusCode::UNAUTHORIZED, "appapi_user_required");
    }
    if !app.enabled.load(Ordering::Acquire) {
        return reply(StatusCode::SERVICE_UNAVAILABLE, "app_disabled");
    }
    if !browser_write_allowed(&request) {
        return reply(StatusCode::FORBIDDEN, "csrf_check_failed");
    }
    if matches!(
        path.as_str(),
        "/api/ingredients"
            | "/api/conversions"
            | "/api/nutrition/calculate"
            | "/api/quantities/convert"
    ) || path.starts_with("/api/ingredients/")
    {
        return app
            .catalogue
            .clone()
            .oneshot(request)
            .await
            .unwrap_or_else(|_| reply(StatusCode::SERVICE_UNAVAILABLE, "request_failed"));
    }
    dispatch_household(app, actor, path, request).await
}

async fn dispatch_household(
    app: Shared,
    actor: String,
    path: String,
    mut request: Request,
) -> Response {
    let Some(rest) = path.strip_prefix("/api/households/") else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some((id, domain)) = rest.split_once("/domain") else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if !(domain.starts_with("/api/recipes") || domain.starts_with("/api/household/")) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let role = match app.membership.lock() {
        Ok(s) => s.role(id, &actor),
        Err(_) => return reply(StatusCode::SERVICE_UNAVAILABLE, "membership_storage_failed"),
    };
    let Some(role) = role else {
        return reply(StatusCode::FORBIDDEN, "household_access_denied");
    };
    if role == Role::Reader && !read_action(request.method(), domain) {
        return reply(StatusCode::FORBIDDEN, "household_read_only");
    }
    let owned_app = app.clone();
    let home_id = id.to_owned();
    let router = tokio::task::spawn_blocking(move || {
        let mut routers = owned_app
            .routers
            .lock()
            .map_err(|_| "Household router lock failed")?;
        if let Some(router) = routers.get(&home_id) {
            return Ok(router.clone());
        }
        let directory = owned_app.config.directory.join(&home_id);
        let router = crate::family_router(
            include_str!("../../data/catalogue.json"),
            directory.join("recipes"),
            directory.join("household"),
            owned_app.config.secret.clone(),
            None,
        )?;
        routers.insert(home_id, router.clone());
        Ok::<_, String>(router)
    })
    .await;
    let Ok(Ok(router)) = router else {
        return reply(StatusCode::SERVICE_UNAVAILABLE, "household_storage_failed");
    };
    let target = request
        .uri()
        .query()
        .map_or_else(|| domain.to_owned(), |q| format!("{domain}?{q}"));
    let Ok(uri) = target.parse::<Uri>() else {
        return reply(StatusCode::BAD_REQUEST, "invalid_request");
    };
    *request.uri_mut() = uri;
    let Ok(token) = format!("Bearer {}", app.config.secret).parse() else {
        return reply(StatusCode::SERVICE_UNAVAILABLE, "request_failed");
    };
    request.headers_mut().insert(header::AUTHORIZATION, token);
    request.headers_mut().remove(header::COOKIE);
    let mut response = router
        .oneshot(request)
        .await
        .unwrap_or_else(|_| reply(StatusCode::SERVICE_UNAVAILABLE, "request_failed"));
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    response
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::too_many_lines)]
mod tests;
