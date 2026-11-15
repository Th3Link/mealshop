//! Protected local household profiles and conservative dietary review.
pub(crate) mod people;

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
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Member {
    pub(crate) id: String,
    name: String,
    allergies: Vec<String>,
    #[serde(default)]
    allergen_ids: Vec<String>,
    #[serde(default)]
    intolerances: Vec<String>,
    excluded_ingredients: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    excluded_tags: Vec<String>,
    disliked_ingredients: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rules: Option<people::Rules>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    avatar: Option<people::Avatar>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Profiles {
    schema_version: u32,
    pub(crate) revision: u64,
    pub(crate) members: Vec<Member>,
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
    pub(crate) members: Vec<Member>,
}
pub(crate) struct Store {
    path: PathBuf,
    lock: PathBuf,
    pub(crate) data: Profiles,
    pub(in crate::household) private: people::Private,
    writable: bool,
}
impl Drop for Store {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock);
    }
}
pub(crate) struct Household {
    token: String,
    reader_token: Option<String>,
    pub(crate) store: Mutex<Store>,
    pub(crate) recipes: Option<Arc<Mutex<crate::recipes::Store>>>,
    pub(crate) plans: Mutex<crate::planning::PlanStore>,
    pub(crate) shopping: Mutex<crate::shopping::Store>,
    pub(crate) cooking: Mutex<crate::cooking::store::Store>,
    pub(crate) feedback: Mutex<crate::feedback::Store>,
    pub(crate) mcp: Mutex<crate::mcp::Store>,
    pub(crate) imports: Mutex<crate::imports::Store>,
    pub(crate) images: PathBuf,
    pub(crate) catalogue: crate::ingredients::Shared,
    pub(crate) allergen_data: serde_json::Value,
}
#[derive(Clone)]
pub(crate) struct Authority {
    pub(crate) user: String,
    pub(crate) role: crate::membership::Role,
    pub(crate) members: std::collections::BTreeMap<String, crate::membership::Role>,
    pub(crate) activity_epochs: BTreeMap<String, u64>,
}
pub(crate) fn authority(
    state: &crate::household::Household,
    headers: &HeaderMap,
    who: Option<axum::Extension<Authority>>,
) -> Authority {
    who.map_or_else(
        || Authority {
            user: if can_write(state, headers) {
                "local"
            } else {
                "local-reader"
            }
            .to_owned(),
            role: if can_write(state, headers) {
                crate::membership::Role::Owner
            } else {
                crate::membership::Role::Reader
            },
            activity_epochs: BTreeMap::new(),
            members: std::collections::BTreeMap::from([
                ("local".to_owned(), crate::membership::Role::Owner),
                ("local-reader".to_owned(), crate::membership::Role::Reader),
            ]),
        },
        |axum::Extension(a)| a,
    )
}
pub(crate) type Shared = Arc<Household>;
pub(crate) fn reply(status: StatusCode, body: serde_json::Value) -> Response {
    (status, [(header::CACHE_CONTROL, "no-store")], Json(body)).into_response()
}
pub(crate) fn can_write(state: &Household, headers: &HeaderMap) -> bool {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .is_some_and(|h| h.strip_prefix("Bearer ") == Some(state.token.as_str()))
}
pub(crate) fn authorized(state: &Household, headers: &HeaderMap) -> bool {
    can_write(state, headers)
        || state.reader_token.as_ref().is_some_and(|token| {
            headers
                .get(header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .is_some_and(|h| h.strip_prefix("Bearer ") == Some(token.as_str()))
        })
}
pub(crate) fn denied() -> Response {
    reply(
        StatusCode::UNAUTHORIZED,
        json!({"code":"household_access_required"}),
    )
}

fn validate_tokens(token: &str, reader_token: Option<&str>) -> Result<(), String> {
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
    if reader_token.is_some_and(|r| {
        r == token
            || r.len() < 32
            || r.len() > 200
            || !r
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    }) {
        return Err("Reader token must be distinct and use 32–200 supported characters".to_owned());
    }
    Ok(())
}
pub(crate) fn router(
    json: &str,
    directory: &Path,
    token: String,
    reader_token: Option<String>,
    recipes: Option<Arc<Mutex<crate::recipes::Store>>>,
    ingredients: Option<crate::ingredients::Shared>,
) -> Result<Router, String> {
    validate_tokens(&token, reader_token.as_deref())?;
    let mut allergen_data: serde_json::Value =
        serde_json::from_str(include_str!("../data/allergens.json"))
            .map_err(|_| "Invalid allergen registry")?;
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
        private: people::Private::default(),
        writable: true,
    };
    lock_file
        .sync_all()
        .map_err(|_| "Cannot sync household lock")?;
    if directory.join(".pending").exists() || directory.join("images/.image.pending").exists() {
        return Err("Interrupted household write: recover before startup".to_owned());
    }
    if store.path.exists() {
        let mut bytes = Vec::new();
        File::open(&store.path)
            .map_err(|_| "Cannot read profiles")?
            .take(4_194_305)
            .read_to_end(&mut bytes)
            .map_err(|_| "Cannot read profiles")?;
        if bytes.len() > 4_194_304 {
            return Err("Household data exceeds 4 MiB".to_owned());
        }
        (store.data, store.private) = people::Document::decode(&bytes)?;
        if store.data.schema_version != 1 {
            return Err("Unsupported household schema".to_owned());
        }
    }
    allergen_data["intolerance_triggers"] =
        serde_json::from_str(include_str!("../data/intolerance-triggers.json"))
            .map_err(|_| "Invalid intolerance registry")?;
    filter_allergens(&mut allergen_data, &catalogue);
    let ingredients = ingredients.map_or_else(
        || {
            crate::ingredients::Store::open(catalogue.clone(), directory)
                .map(|store| Arc::new(Mutex::new(store)))
        },
        Ok,
    )?;
    let plans = crate::planning::PlanStore::open(directory)?;
    let state = Household {
        recipes,
        images: directory.join("images"),
        plans: Mutex::new(plans),
        shopping: Mutex::new(crate::shopping::Store::open(directory)?),
        cooking: Mutex::new(crate::cooking::store::Store::open(directory)?),
        feedback: Mutex::new(crate::feedback::Store::open(directory)?),
        mcp: Mutex::new(crate::mcp::Store::open(directory)?),
        imports: Mutex::new(crate::imports::Store::default()),
        token,
        reader_token,
        allergen_data,
        store: Mutex::new(store),
        catalogue: ingredients,
    };
    {
        let store = state.store.lock().map_err(|_| "Household lock failed")?;
        validate(&store.data.members, &ingredient_ids(&state)?)?;
    }
    Ok(Router::new()
        .route("/api/household/allergens", get(allergens))
        .merge(people::routes())
        .merge(crate::planning::routes())
        .merge(crate::shopping::routes())
        .merge(crate::inventory::routes())
        .merge(crate::brewing::routes())
        .merge(crate::cooking::routes())
        .merge(crate::feedback::routes())
        .merge(crate::mcp::routes())
        .merge(crate::imports::routes())
        .merge(crate::ingredient_api::routes())
        .merge(crate::recipe_images::routes())
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
            || m.rules.as_ref().is_some_and(|r| !r.validate(ingredients))
            || m.avatar.as_ref().is_some_and(|a| !a.valid())
            || m.allergen_ids.len() > 14
            || m.allergen_ids
                .iter()
                .any(|id| !allergen_ids().contains(&id.as_str()))
            || m.intolerances.len() > 50
            || m.intolerances
                .iter()
                .any(|s| s.trim().is_empty() || s.len() > 200 || s.chars().any(char::is_control))
            || m.allergies.len() > 50
            || m.allergies
                .iter()
                .any(|a| a.trim().is_empty() || a.len() > 200 || a.chars().any(char::is_control))
            || m.excluded_tags.len() > 30
            || m.excluded_tags.iter().any(|tag| {
                tag.trim().is_empty()
                    || tag.trim() != tag
                    || tag.len() > 80
                    || tag.chars().any(char::is_control)
            })
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
    tokio::task::spawn_blocking(move || match state.store.lock() {
        Ok(store) => reply(StatusCode::OK, json!(store.data)),
        Err(_) => reply(
            StatusCode::SERVICE_UNAVAILABLE,
            json!({"code":"household_storage_failed"}),
        ),
    })
    .await
    .unwrap_or_else(|_| {
        reply(
            StatusCode::SERVICE_UNAVAILABLE,
            json!({"code":"household_storage_failed"}),
        )
    })
}
async fn write(
    State(state): State<Shared>,
    who: Option<axum::Extension<Authority>>,
    headers: HeaderMap,
    body: Result<Json<Update>, axum::extract::rejection::JsonRejection>,
) -> Response {
    if !can_write(&state, &headers) {
        return if authorized(&state, &headers) {
            reply(
                StatusCode::FORBIDDEN,
                json!({"code":"household_write_required"}),
            )
        } else {
            denied()
        };
    }
    let Ok(Json(update)) = body else {
        return reply(StatusCode::BAD_REQUEST, json!({"code":"invalid_household"}));
    };
    if !ingredient_ids(&state).is_ok_and(|ids| validate(&update.members, &ids).is_ok()) {
        return reply(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({"code":"invalid_household"}),
        );
    }
    let actor = authority(&state, &headers, who);
    tokio::task::spawn_blocking(move || {
        if update
            .members
            .iter()
            .filter_map(|m| m.avatar.as_ref())
            .filter_map(|a| a.image_id.as_deref())
            .any(|id| {
                crate::recipe_images::validate_reference(Some(&state.images), Some(id)).is_err()
            })
        {
            return reply(
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({"code":"invalid_household"}),
            );
        }
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
        match store.replace_profiles(update.members, update.expected_revision, actor.role) {
            Ok(()) => reply(StatusCode::OK, json!(store.data)),
            Err("person_owner_required") => reply(
                StatusCode::FORBIDDEN,
                json!({"code":"person_owner_required"}),
            ),
            Err(code) => reply(StatusCode::SERVICE_UNAVAILABLE, json!({"code":code})),
        }
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
pub(crate) struct Evaluation {
    pub(crate) member_ids: Vec<String>,
    pub(crate) ingredient_ids: Vec<String>,
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
    tokio::task::spawn_blocking(move || {
        let Ok(store) = state.store.lock() else {
            return reply(
                StatusCode::SERVICE_UNAVAILABLE,
                json!({"code":"household_storage_failed"}),
            );
        };
        match review(&state, &store.data, &input) {
            Ok(value) => reply(StatusCode::OK, value),
            Err(code) => reply(StatusCode::UNPROCESSABLE_ENTITY, json!({"code":code})),
        }
    })
    .await
    .unwrap_or_else(|_| {
        reply(
            StatusCode::SERVICE_UNAVAILABLE,
            json!({"code":"household_storage_failed"}),
        )
    })
}
pub(crate) fn review(
    state: &Household,
    profiles: &Profiles,
    input: &Evaluation,
) -> Result<serde_json::Value, &'static str> {
    let ingredients = state
        .catalogue
        .lock()
        .map_err(|_| "ingredient_storage_failed")?;
    review_snapshot(state, profiles, input, &ingredients, None)
}
pub(crate) fn ingredient_ids(state: &Household) -> Result<BTreeSet<String>, String> {
    Ok(state
        .catalogue
        .lock()
        .map_err(|_| "Ingredient lock failed")?
        .catalogue()
        .ingredients
        .into_iter()
        .map(|i| i.id)
        .collect())
}
pub(crate) fn allergen_version(
    state: &Household,
    ingredients: &crate::ingredients::Store,
) -> String {
    format!(
        "{}+household-{}",
        state.allergen_data["version"].as_str().unwrap_or("unknown"),
        ingredients.dietary_revision()
    )
}
pub(crate) fn review_snapshot(
    state: &Household,
    profiles: &Profiles,
    input: &Evaluation,
    current: &crate::ingredients::Store,
    snapshot: Option<&crate::nutrition::catalogue::Catalogue>,
) -> Result<serde_json::Value, &'static str> {
    let catalogue = current.catalogue();
    let ingredients: BTreeSet<_> = catalogue.ingredients.iter().map(|i| i.id.clone()).collect();
    let aliases = ingredient_aliases(&catalogue);
    let members: BTreeMap<_, _> = profiles.members.iter().map(|m| (&m.id, m)).collect();
    if input.member_ids.iter().any(|id| !members.contains_key(id)) {
        return Err("unknown_household_member");
    }
    let mut resolved = BTreeSet::new();
    for input in &input.ingredient_ids {
        if ingredients.contains(input) {
            resolved.insert(input.clone());
        } else if let Some(ids) = aliases.get(&input.to_lowercase()) {
            resolved.extend(ids.iter().cloned());
        }
    }
    let direct = resolved.clone();
    resolved = catalogue.closure(&resolved);
    if let Some(snapshot) = snapshot {
        resolved.extend(snapshot.closure(&resolved));
    }
    let mut excluded = BTreeSet::new();
    let mut disliked = BTreeSet::new();
    let mut allergen_matches = Vec::new();
    let mut dietary_rules = Vec::new();
    for id in &input.member_ids {
        if let Some(m) = members.get(id) {
            if let Some(rules) = &m.rules {
                let mut result =
                    people::diet_review(rules, &resolved, &direct, &catalogue, snapshot);
                if let Some(ids) = result["excluded_ingredients"].as_array() {
                    for id in ids.iter().filter_map(serde_json::Value::as_str) {
                        if let Some(id) = resolved.get(id) {
                            excluded.insert(id);
                        }
                    }
                }
                result["member_id"] = json!(m.id);
                dietary_rules.push(result);
            }
            for ingredient in &resolved {
                for entry in state.allergen_data["entries"]
                    .as_array()
                    .into_iter()
                    .flatten()
                {
                    let allergen = entry["allergen_id"].as_str().unwrap_or("");
                    if entry["ingredient_id"].as_str() == Some(ingredient.as_str())
                        && (m.allergen_ids.iter().any(|a| a == allergen)
                            || m.allergies
                                .iter()
                                .chain(&m.intolerances)
                                .any(|a| a.to_lowercase() == allergen))
                    {
                        excluded.insert(ingredient);
                        allergen_matches.push(json!({"member_id":m.id,"ingredient_id":ingredient,"allergen_id":allergen,"source_url":entry["source_url"]}));
                    }
                }
                for source_catalogue in std::iter::once(&catalogue).chain(snapshot) {
                    if let Some(item) = source_catalogue.find(ingredient) {
                        if item.tags.iter().any(|tag| {
                            m.excluded_tags
                                .iter()
                                .any(|excluded| excluded.to_lowercase() == tag.to_lowercase())
                        }) {
                            excluded.insert(ingredient);
                        }
                        for trigger in item.organization.intolerance_triggers.iter().flatten() {
                            if m.intolerances.iter().any(|id| {
                                id.eq_ignore_ascii_case(trigger)
                                    || (trigger == "sorbitol" && id.eq_ignore_ascii_case("sorbit"))
                            }) {
                                excluded.insert(ingredient);
                                allergen_matches.push(json!({"member_id":m.id,"ingredient_id":ingredient,"allergen_id":trigger,"source_url":null,"source_kind":"household_intolerance_declaration"}));
                            }
                        }
                        for allergen in &item.declared_allergens {
                            if m.allergen_ids.contains(allergen)
                                || m.allergies
                                    .iter()
                                    .chain(&m.intolerances)
                                    .any(|a| a.to_lowercase() == *allergen)
                            {
                                excluded.insert(ingredient);
                                allergen_matches.push(json!({"member_id":m.id,"ingredient_id":ingredient,"allergen_id":allergen,"source_url":null,"source_kind":"household_declaration"}));
                            }
                        }
                    }
                }
                if m.excluded_ingredients.contains(ingredient) {
                    excluded.insert(ingredient);
                }
                if m.disliked_ingredients.contains(ingredient) {
                    disliked.insert(ingredient);
                }
            }
        }
    }
    Ok(
        json!({"status":if excluded.is_empty(){"review_required"}else{"blocked"},"excluded_ingredients":excluded,"allergen_matches":allergen_matches,"review_notes":input.member_ids.iter().filter_map(|id|members.get(id)).map(|m|json!({"member_id":m.id,"name":m.name,"allergies":m.allergies,"intolerances":m.intolerances,"excluded_tags":m.excluded_tags,"allergen_ids":m.allergen_ids})).collect::<Vec<_>>(),"allergen_data_version":allergen_version(state, current),"disliked_ingredients":disliked,"unknown_ingredients":input.ingredient_ids.iter().filter(|id|!ingredients.contains(*id) && aliases.get(&id.to_lowercase()).is_none_or(|ids|ids.len()!=1)).collect::<Vec<_>>(),"warning_codes":["allergen_coverage_incomplete","cross_contamination_unmodelled"],"profile_revision":profiles.revision,"dietary_rules":dietary_rules}),
    )
}

pub(crate) fn allergen_ids() -> &'static [&'static str] {
    &[
        "milk",
        "egg",
        "soy",
        "gluten_cereals",
        "peanut",
        "tree_nuts",
        "sesame",
        "lupin",
        "celery",
        "mustard",
        "fish",
        "crustacean",
        "mollusc",
        "sulphites",
    ]
}
async fn allergens(State(state): State<Shared>, headers: HeaderMap) -> Response {
    if !authorized(&state, &headers) {
        return denied();
    }
    reply(StatusCode::OK, state.allergen_data.clone())
}

fn ingredient_aliases(
    catalogue: &crate::nutrition::catalogue::Catalogue,
) -> BTreeMap<String, BTreeSet<String>> {
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
    aliases
}

fn filter_allergens(
    allergen_data: &mut serde_json::Value,
    catalogue: &crate::nutrition::catalogue::Catalogue,
) {
    if let Some(entries) = allergen_data["entries"].as_array_mut() {
        entries.retain(|e| {
            catalogue.ingredients.iter().any(|i| {
                e["ingredient_id"] == i.id
                    && e["form"] == i.form
                    && e["source_record_id"] == i.source.record_id
            })
        });
    }
}
