//! Protected local household profiles and conservative dietary review.
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Member {
    id: String,
    name: String,
    allergies: Vec<String>,
    excluded_ingredients: Vec<String>,
    disliked_ingredients: Vec<String>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Profiles {
    schema_version: u32,
    revision: u64,
    members: Vec<Member>,
}
impl Default for Profiles {
    fn default() -> Self {
        Self {
            schema_version: 1,
            revision: 0,
            members: Vec::new(),
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Update {
    expected_revision: u64,
    members: Vec<Member>,
}
struct Store {
    path: PathBuf,
    lock: PathBuf,
    data: Profiles,
    writable: bool,
}
impl Drop for Store {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock);
    }
}
struct Household {
    token: String,
    store: Mutex<Store>,
    ingredients: BTreeSet<String>,
    aliases: BTreeMap<String, BTreeSet<String>>,
}
type Shared = Arc<Household>;
fn reply(status: StatusCode, body: serde_json::Value) -> Response {
    (status, [(header::CACHE_CONTROL, "no-store")], Json(body)).into_response()
}
fn authorized(state: &Household, headers: &HeaderMap) -> bool {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .is_some_and(|h| h.strip_prefix("Bearer ") == Some(state.token.as_str()))
}
fn denied() -> Response {
    reply(
        StatusCode::UNAUTHORIZED,
        json!({"code":"household_access_required"}),
    )
}

pub(crate) fn router(json: &str, directory: &Path, token: String) -> Result<Router, String> {
    if token.len() < 32
        || token.len() > 200
        || !token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(
            "Household token must contain 32–200 ASCII letters, digits, underscores or hyphens"
                .to_owned(),
        );
    }
    let catalogue = crate::nutrition::catalogue::Catalogue::from_json(json)?;
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder
        .create(directory)
        .map_err(|_| "Cannot create household directory")?;
    let path = directory.join("profiles.json");
    let lock = directory.join(".household.lock");
    let lock_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock)
        .map_err(|_| "Household directory locked; stop writers before recovering a stale lock")?;
    let mut store = Store {
        path,
        lock,
        data: Profiles::default(),
        writable: true,
    };
    lock_file
        .sync_all()
        .map_err(|_| "Cannot sync household lock")?;
    if directory.join(".pending").exists() {
        return Err("Interrupted household write: recover before startup".to_owned());
    }
    if store.path.exists() {
        let mut bytes = Vec::new();
        File::open(&store.path)
            .map_err(|_| "Cannot read profiles")?
            .take(1_048_577)
            .read_to_end(&mut bytes)
            .map_err(|_| "Cannot read profiles")?;
        if bytes.len() > 1_048_576 {
            return Err("Household data exceeds 1 MiB".to_owned());
        }
        store.data = serde_json::from_slice(&bytes).map_err(|_| "Invalid household data")?;
        if store.data.schema_version != 1 {
            return Err("Unsupported household schema".to_owned());
        }
    }
    let mut aliases: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for ingredient in &catalogue.ingredients {
        let mut labels = vec![ingredient.name.as_str()];
        labels.extend(ingredient.aliases.iter().map(String::as_str));
        for translated in ingredient.translations.values() {
            labels.push(&translated.name);
            labels.extend(translated.aliases.iter().map(String::as_str));
        }
        for label in labels {
            aliases
                .entry(label.to_lowercase())
                .or_default()
                .insert(ingredient.id.clone());
        }
    }
    let state = Household {
        aliases,
        token,
        store: Mutex::new(store),
        ingredients: catalogue.ingredients.into_iter().map(|i| i.id).collect(),
    };
    {
        let store = state.store.lock().map_err(|_| "Household lock failed")?;
        validate(&store.data.members, &state.ingredients)?;
    }
    Ok(Router::new()
        .route("/api/household/profiles", get(read).put(write))
        .route("/api/household/evaluate", post(evaluate))
        .layer(DefaultBodyLimit::max(131_072))
        .with_state(Arc::new(state)))
}
fn validate(members: &[Member], ingredients: &BTreeSet<String>) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    if members.len() > 100 {
        return Err("Use at most 100 members".to_owned());
    }
    for m in members {
        if m.id.is_empty()
            || m.id.len() > 80
            || !m
                .id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || !ids.insert(&m.id)
            || m.name.trim().is_empty()
            || m.name.len() > 200
            || m.name.chars().any(char::is_control)
            || m.allergies.len() > 50
            || m.allergies
                .iter()
                .any(|a| a.trim().is_empty() || a.len() > 200 || a.chars().any(char::is_control))
            || m.excluded_ingredients.len() > 1000
            || m.disliked_ingredients.len() > 1000
            || m.excluded_ingredients
                .iter()
                .chain(&m.disliked_ingredients)
                .any(|id| !ingredients.contains(id))
        {
            return Err("Invalid household member fields or ingredient IDs".to_owned());
        }
    }
    Ok(())
}
async fn read(State(state): State<Shared>, headers: HeaderMap) -> Response {
    if !authorized(&state, &headers) {
        return denied();
    }
    match state.store.lock() {
        Ok(store) => reply(StatusCode::OK, json!(store.data)),
        Err(_) => reply(
            StatusCode::SERVICE_UNAVAILABLE,
            json!({"code":"household_storage_failed"}),
        ),
    }
}
async fn write(
    State(state): State<Shared>,
    headers: HeaderMap,
    body: Result<Json<Update>, axum::extract::rejection::JsonRejection>,
) -> Response {
    if !authorized(&state, &headers) {
        return denied();
    }
    let Ok(Json(update)) = body else {
        return reply(StatusCode::BAD_REQUEST, json!({"code":"invalid_household"}));
    };
    if validate(&update.members, &state.ingredients).is_err() {
        return reply(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({"code":"invalid_household"}),
        );
    }
    tokio::task::spawn_blocking(move || {
        let Ok(mut store) = state.store.lock() else {
            return reply(
                StatusCode::SERVICE_UNAVAILABLE,
                json!({"code":"household_storage_failed"}),
            );
        };
        if !store.writable {
            return reply(
                StatusCode::SERVICE_UNAVAILABLE,
                json!({"code":"household_storage_failed"}),
            );
        }
        if update.expected_revision != store.data.revision {
            return reply(StatusCode::CONFLICT, json!({"code":"revision_conflict"}));
        }
        let Some(revision) = store.data.revision.checked_add(1) else {
            return reply(
                StatusCode::INSUFFICIENT_STORAGE,
                json!({"code":"household_revision_limit"}),
            );
        };
        let data = Profiles {
            schema_version: 1,
            revision,
            members: update.members,
        };
        let pending = store.path.with_file_name(".pending");
        let persist = || -> Result<(), std::io::Error> {
            let bytes = serde_json::to_vec(&data)?;
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&pending)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            fs::rename(&pending, &store.path)?;
            #[cfg(unix)]
            File::open(
                store
                    .path
                    .parent()
                    .ok_or_else(|| std::io::Error::other("Missing parent"))?,
            )?
            .sync_all()?;
            Ok(())
        };
        if persist().is_err() {
            store.writable = false;
            return reply(
                StatusCode::SERVICE_UNAVAILABLE,
                json!({"code":"household_storage_failed"}),
            );
        }
        store.data = data;
        reply(StatusCode::OK, json!(store.data))
    })
    .await
    .unwrap_or_else(|_| {
        reply(
            StatusCode::SERVICE_UNAVAILABLE,
            json!({"code":"household_storage_failed"}),
        )
    })
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Evaluation {
    member_ids: Vec<String>,
    ingredient_ids: Vec<String>,
}
async fn evaluate(
    State(state): State<Shared>,
    headers: HeaderMap,
    body: Result<Json<Evaluation>, axum::extract::rejection::JsonRejection>,
) -> Response {
    if !authorized(&state, &headers) {
        return denied();
    }
    let Ok(Json(input)) = body else {
        return reply(StatusCode::BAD_REQUEST, json!({"code":"invalid_household"}));
    };
    if input.member_ids.is_empty()
        || input.member_ids.len() > 100
        || input.ingredient_ids.is_empty()
        || input.ingredient_ids.len() > 1000
    {
        return reply(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({"code":"invalid_household"}),
        );
    }
    let Ok(store) = state.store.lock() else {
        return reply(
            StatusCode::SERVICE_UNAVAILABLE,
            json!({"code":"household_storage_failed"}),
        );
    };
    let members: BTreeMap<_, _> = store.data.members.iter().map(|m| (&m.id, m)).collect();
    if input.member_ids.iter().any(|id| !members.contains_key(id)) {
        return reply(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({"code":"unknown_household_member"}),
        );
    }
    let mut resolved = BTreeSet::new();
    for input in &input.ingredient_ids {
        if state.ingredients.contains(input) {
            resolved.insert(input.clone());
        } else if let Some(ids) = state.aliases.get(&input.to_lowercase()) {
            resolved.extend(ids.iter().cloned());
        }
    }
    let mut excluded = BTreeSet::new();
    let mut disliked = BTreeSet::new();
    for id in &input.member_ids {
        if let Some(m) = members.get(id) {
            for ingredient in &resolved {
                if m.excluded_ingredients.contains(ingredient) {
                    excluded.insert(ingredient);
                }
                if m.disliked_ingredients.contains(ingredient) {
                    disliked.insert(ingredient);
                }
            }
        }
    }
    reply(
        StatusCode::OK,
        json!({"status":if excluded.is_empty(){"review_required"}else{"blocked"},"excluded_ingredients":excluded,"disliked_ingredients":disliked,"unknown_ingredients":input.ingredient_ids.iter().filter(|id|!state.ingredients.contains(*id) && state.aliases.get(&id.to_lowercase()).is_none_or(|ids|ids.len()!=1)).collect::<Vec<_>>(),"warning_codes":["allergen_mapping_unavailable","cross_contamination_unmodelled"],"profile_revision":store.data.revision}),
    )
}
