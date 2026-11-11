use super::{Actor, Entry, Input, Policy, Recipe, Source, Store, store::Draft};
use crate::household::{self, Shared};
use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::get,
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RequestedSource {
    Manual {
        recipe_id: String,
        revision: u64,
        servings: u32,
    },
    Session {
        run_id: String,
    },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Save {
    expected_revision: u64,
    expected_profile_revision: u64,
    confirmed_completed: bool,
    source: RequestedSource,
    input: Input,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Delete {
    expected_revision: u64,
    confirmed: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Configure {
    expected_revision: u64,
    expected_profile_revision: u64,
    policy: Policy,
}
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Listing {
    page: Option<usize>,
    q: String,
    session: Option<String>,
}

pub(crate) fn routes() -> Router<Shared> {
    Router::new()
        .route(
            "/api/household/history",
            get(list).post(create).delete(clear),
        )
        .route(
            "/api/household/history/{id}",
            get(detail).put(update).delete(remove),
        )
        .route("/api/household/learning", get(learning).put(configure))
}
fn actor(value: Option<Extension<Actor>>) -> Actor {
    value.map_or_else(
        || Actor {
            user: "local".to_owned(),
            owner: true,
        },
        |Extension(a)| a,
    )
}
fn failure(code: &'static str) -> Response {
    household::reply(
        match code {
            "history_not_found" => StatusCode::NOT_FOUND,
            "history_owner_required" | "household_write_required" => StatusCode::FORBIDDEN,
            "history_revision_conflict"
            | "history_profiles_changed"
            | "history_duplicate_session" => StatusCode::CONFLICT,
            "history_storage_failed" | "recipe_store_unavailable" | "cooking_storage_failed" => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            "history_capacity" => StatusCode::INSUFFICIENT_STORAGE,
            _ => StatusCode::UNPROCESSABLE_ENTITY,
        },
        json!({"code":code}),
    )
}
fn response(
    status: StatusCode,
    result: Result<Result<Value, &'static str>, tokio::task::JoinError>,
) -> Response {
    match result {
        Ok(Ok(v)) => household::reply(status, v),
        Ok(Err(code)) => failure(code),
        Err(_) => failure("history_storage_failed"),
    }
}
pub(super) fn policy_view(store: &Store) -> Value {
    json!({"revision":store.data.revision,"epoch":store.data.epoch,"policy":store.data.policy,
        "model_version":if store.data.policy.enabled {"history-v1"}else{"rules-v1"},"learned_cooks":store.data.stats.values().map(|s|u64::from(s.cooks)).sum::<u64>(),"learned_recipes":store.data.stats.len()})
}
async fn learning(State(state): State<Shared>, headers: HeaderMap) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    response(
        StatusCode::OK,
        tokio::task::spawn_blocking(move || {
            let store = state
                .feedback
                .lock()
                .map_err(|_| "history_storage_failed")?;
            Ok(policy_view(&store))
        })
        .await,
    )
}
async fn configure(
    State(state): State<Shared>,
    headers: HeaderMap,
    who: Option<Extension<Actor>>,
    body: Result<Json<Configure>, JsonRejection>,
) -> Response {
    if !household::can_write(&state, &headers) {
        return failure("household_write_required");
    }
    if !actor(who).owner {
        return failure("history_owner_required");
    }
    let Ok(Json(body)) = body else {
        return failure("history_input_invalid");
    };
    response(
        StatusCode::OK,
        tokio::task::spawn_blocking(move || {
            let profiles = state.store.lock().map_err(|_| "history_storage_failed")?;
            if profiles.data.revision != body.expected_profile_revision {
                return Err("history_profiles_changed");
            }
            if !body.policy.enabled && !body.policy.member_ids.is_empty() {
                return Err("history_input_invalid");
            }
            if body
                .policy
                .member_ids
                .iter()
                .any(|id| !profiles.data.members.iter().any(|m| m.id == *id))
            {
                return Err("unknown_household_member");
            }
            let mut store = state
                .feedback
                .lock()
                .map_err(|_| "history_storage_failed")?;
            store.policy(body.expected_revision, body.policy)?;
            Ok(policy_view(&store))
        })
        .await,
    )
}
async fn list(
    State(state): State<Shared>,
    headers: HeaderMap,
    query: Result<Query<Listing>, axum::extract::rejection::QueryRejection>,
) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    let Ok(Query(query)) = query else {
        return failure("history_input_invalid");
    };
    let page = query.page.unwrap_or(1);
    if page == 0
        || page.checked_mul(20).is_none()
        || query.q.len() > 200
        || query.session.as_ref().is_some_and(|s| !super::valid_id(s))
    {
        return failure("history_input_invalid");
    }
    response(StatusCode::OK,tokio::task::spawn_blocking(move||{
        let store=state.feedback.lock().map_err(|_|"history_storage_failed")?;
        let q=query.q.trim().to_lowercase();
        let mut matching=store.data.entries.values().filter(|e|e.recipe.title.to_lowercase().contains(&q)
            && query.session.as_ref().is_none_or(|id|matches!(&e.source,Source::Session {run_id} if run_id==id))).collect::<Vec<_>>();
        matching.sort_by_key(|e|(std::cmp::Reverse(&e.input.date),std::cmp::Reverse(e.recorded_at),&e.id));
        let total=matching.len();let entries=matching.into_iter().skip((page-1)*20).take(20).collect::<Vec<_>>();
        Ok(json!({"revision":store.data.revision,"page":page,"per_page":20,"total":total,"entries":entries}))
    }).await)
}
async fn detail(
    State(state): State<Shared>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    response(
        StatusCode::OK,
        tokio::task::spawn_blocking(move || {
            let store = state
                .feedback
                .lock()
                .map_err(|_| "history_storage_failed")?;
            let entry = store.data.entries.get(&id).ok_or("history_not_found")?;
            Ok(json!({"revision":store.data.revision,"entry":entry}))
        })
        .await,
    )
}
fn manual(
    recipes: &crate::recipes::Store,
    id: &str,
    revision: u64,
    servings: u32,
) -> Result<(Source, Recipe, u32), &'static str> {
    let record = recipes
        .get(id, Some(revision))
        .map_err(|_| "history_recipe_invalid")?;
    Ok((
        Source::Manual,
        Recipe {
            id: record.id,
            revision: record.revision,
            title: record.recipe.title,
            language: record.recipe.language,
        },
        servings,
    ))
}
fn source(
    recipes: &crate::recipes::Store,
    cooking: &crate::cooking::store::Store,
    requested: RequestedSource,
    previous: Option<&Entry>,
) -> Result<(Source, Recipe, u32), &'static str> {
    if let Some(previous) = previous {
        if let Source::Session { run_id: old } = &previous.source {
            return if matches!(&requested,RequestedSource::Session{run_id} if run_id==old) {
                Ok((
                    previous.source.clone(),
                    previous.recipe.clone(),
                    previous.servings,
                ))
            } else {
                Err("history_source_changed")
            };
        }
    }
    match requested {
        RequestedSource::Manual {
            recipe_id,
            revision,
            servings,
        } => manual(recipes, &recipe_id, revision, servings),
        RequestedSource::Session { run_id } => {
            let (id, revision, servings) = cooking.completed_recipe(&run_id)?;
            let (_, recipe, _) = manual(recipes, &id, revision, servings)?;
            Ok((Source::Session { run_id }, recipe, servings))
        }
    }
}
fn actual_date(input: &Input, now: i64) -> Result<(), &'static str> {
    input.validate()?;
    let date = input
        .date
        .parse::<jiff::civil::Date>()
        .map_err(|_| "history_input_invalid")?;
    let zone = jiff::tz::TimeZone::get(&input.timezone).map_err(|_| "history_input_invalid")?;
    if date
        > jiff::Timestamp::from_second(now)
            .map_err(|_| "history_clock_invalid")?
            .to_zoned(zone)
            .date()
    {
        return Err("history_future_date");
    }
    Ok(())
}
async fn save(
    state: Shared,
    headers: HeaderMap,
    who: Option<Extension<Actor>>,
    body: Result<Json<Save>, JsonRejection>,
    id: Option<String>,
) -> Response {
    if !household::can_write(&state, &headers) {
        return failure("household_write_required");
    }
    let Ok(Json(body)) = body else {
        return failure("history_input_invalid");
    };
    if !body.confirmed_completed {
        return failure("history_completion_required");
    }
    let actor = actor(who);
    let status = if id.is_none() {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    response(
        status,
        tokio::task::spawn_blocking(move || {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .and_then(|d| i64::try_from(d.as_secs()).ok())
                .ok_or("history_clock_invalid")?;
            actual_date(&body.input, now)?;
            let recipes = state
                .recipes
                .as_ref()
                .ok_or("recipe_store_unavailable")?
                .lock()
                .map_err(|_| "recipe_store_unavailable")?;
            let profiles = state.store.lock().map_err(|_| "history_storage_failed")?;
            if profiles.data.revision != body.expected_profile_revision {
                return Err("history_profiles_changed");
            }
            if body
                .input
                .attendees
                .iter()
                .any(|id| !profiles.data.members.iter().any(|m| m.id == *id))
            {
                return Err("unknown_household_member");
            }
            let cooking = state.cooking.lock().map_err(|_| "cooking_storage_failed")?;
            let mut store = state
                .feedback
                .lock()
                .map_err(|_| "history_storage_failed")?;
            let previous = id
                .as_ref()
                .map(|id| store.data.entries.get(id).ok_or("history_not_found"))
                .transpose()?;
            let (source, recipe, servings) = source(&recipes, &cooking, body.source, previous)?;
            let draft = Draft {
                source,
                recipe,
                servings,
                input: body.input,
            };
            let id = store.save(
                body.expected_revision,
                id.as_deref(),
                draft,
                &actor.user,
                now,
            )?;
            Ok(json!({"id":id,"revision":store.data.revision}))
        })
        .await,
    )
}
async fn create(
    State(state): State<Shared>,
    headers: HeaderMap,
    who: Option<Extension<Actor>>,
    body: Result<Json<Save>, JsonRejection>,
) -> Response {
    save(state, headers, who, body, None).await
}
async fn update(
    State(state): State<Shared>,
    Path(id): Path<String>,
    headers: HeaderMap,
    who: Option<Extension<Actor>>,
    body: Result<Json<Save>, JsonRejection>,
) -> Response {
    save(state, headers, who, body, Some(id)).await
}
async fn erase(
    state: Shared,
    headers: HeaderMap,
    who: Option<Extension<Actor>>,
    body: Result<Json<Delete>, JsonRejection>,
    id: Option<String>,
) -> Response {
    if !household::can_write(&state, &headers) {
        return failure("household_write_required");
    }
    if id.is_none() && !actor(who).owner {
        return failure("history_owner_required");
    }
    let Ok(Json(body)) = body else {
        return failure("history_input_invalid");
    };
    if !body.confirmed {
        return failure("history_delete_confirmation");
    }
    response(
        StatusCode::OK,
        tokio::task::spawn_blocking(move || {
            let mut store = state
                .feedback
                .lock()
                .map_err(|_| "history_storage_failed")?;
            if let Some(id) = id {
                store.remove(body.expected_revision, &id)?;
            } else {
                store.clear(body.expected_revision)?;
            }
            Ok(json!({"revision":store.data.revision}))
        })
        .await,
    )
}
async fn remove(
    State(state): State<Shared>,
    Path(id): Path<String>,
    headers: HeaderMap,
    who: Option<Extension<Actor>>,
    body: Result<Json<Delete>, JsonRejection>,
) -> Response {
    erase(state, headers, who, body, Some(id)).await
}
async fn clear(
    State(state): State<Shared>,
    headers: HeaderMap,
    who: Option<Extension<Actor>>,
    body: Result<Json<Delete>, JsonRejection>,
) -> Response {
    erase(state, headers, who, body, None).await
}
