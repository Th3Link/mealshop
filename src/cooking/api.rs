use super::{
    Actor,
    model::{Draft, Permission},
    operations::{self, PlanChange, TimerAction},
    source,
    store::Store,
    time, view,
};
use crate::household::{self, Shared};
use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Template {
    recipe_id: String,
    revision: u64,
    servings: u32,
    timezone: String,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Proposal {
    Create { draft: Draft },
    Replan { run_id: String, change: PlanChange },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Preview {
    expected_revision: u64,
    proposal: Proposal,
}
#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
enum Command {
    Apply {
        preview_id: String,
        reviewed: bool,
    },
    Timer {
        run_id: String,
        step_id: String,
        action: TimerAction,
    },
    Cancel {
        run_id: String,
    },
    Remove {
        run_id: String,
    },
    Subscribe {
        run_id: String,
        enabled: bool,
        language: String,
        permission: Permission,
    },
    Acknowledge {
        notice_id: String,
    },
    Retry {
        notice_id: String,
        duplicate_risk_accepted: bool,
    },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Mutation {
    expected_revision: u64,
    command: Command,
}
pub(crate) fn routes() -> Router<Shared> {
    Router::new()
        .route("/api/household/cooking", get(list).post(change))
        .route("/api/household/cooking/template", get(template))
        .route("/api/household/cooking/preview", post(preview))
        .route(
            "/api/household/cooking/internal",
            post(super::internal::dispatch),
        )
        .route("/api/household/cooking/{id}", get(detail))
}
fn actor(value: Option<Extension<Actor>>) -> Actor {
    value.map_or_else(
        || Actor {
            user: "local".to_owned(),
            notifications_available: false,
        },
        |Extension(actor)| actor,
    )
}
fn failure(code: &str) -> Response {
    let status = match code {
        "cooking_revision_conflict" => StatusCode::CONFLICT,
        "cooking_preview_expired" => StatusCode::GONE,
        "cooking_run_not_found" | "cooking_step_not_found" => StatusCode::NOT_FOUND,
        "cooking_storage_failed" => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::UNPROCESSABLE_ENTITY,
    };
    household::reply(status, json!({"code":code}))
}
fn response(result: Result<Result<Value, &'static str>, tokio::task::JoinError>) -> Response {
    match result {
        Ok(Ok(value)) => household::reply(StatusCode::OK, value),
        Ok(Err(code)) => failure(code),
        Err(_) => failure("cooking_storage_failed"),
    }
}
async fn list(
    State(state): State<Shared>,
    headers: HeaderMap,
    who: Option<Extension<Actor>>,
) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    let actor = actor(who);
    response(
        tokio::task::spawn_blocking(move || {
            let store = state.cooking.lock().map_err(|_| "cooking_storage_failed")?;
            let mut value = view::list(&store.data, &actor, time::now()?);
            value["scheduler"] = store.health();
            Ok(value)
        })
        .await,
    )
}
async fn template(
    State(state): State<Shared>,
    headers: HeaderMap,
    who: Option<Extension<Actor>>,
    Query(input): Query<Template>,
) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    let actor = actor(who);
    response(tokio::task::spawn_blocking(move || {
        let recipes=state.recipes.as_ref().ok_or("recipe_store_unavailable")?.lock().map_err(|_|"recipe_store_unavailable")?;
        let record=source::recipe(&recipes,&input.recipe_id,input.revision)?;
        let draft=source::template(&record,input.servings,input.timezone)?;
        let store=state.cooking.lock().map_err(|_|"cooking_storage_failed")?;
        Ok(json!({"revision":store.data.revision,"server_now":time::now()?,"draft":draft,"notification_transport":actor.notifications_available}))
    }).await)
}
async fn detail(
    State(state): State<Shared>,
    headers: HeaderMap,
    who: Option<Extension<Actor>>,
    Path(id): Path<String>,
) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    let actor = actor(who);
    response(
        tokio::task::spawn_blocking(move || {
            let recipes = state
                .recipes
                .as_ref()
                .ok_or("recipe_store_unavailable")?
                .lock()
                .map_err(|_| "recipe_store_unavailable")?;
            let store = state.cooking.lock().map_err(|_| "cooking_storage_failed")?;
            let run = store.data.run(&id)?;
            let record = source::recipe(&recipes, &run.recipe.id, run.recipe.revision)?;
            let mut value = view::detail(
                &store.data,
                &id,
                &actor,
                time::now()?,
                &source::ingredients(&record, run.servings)?,
            )?;
            value["scheduler"] = store.health();
            Ok(value)
        })
        .await,
    )
}
async fn preview(
    State(state): State<Shared>,
    headers: HeaderMap,
    who: Option<Extension<Actor>>,
    body: Result<Json<Preview>, JsonRejection>,
) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    let actor = actor(who);
    let Ok(Json(input)) = body else {
        return failure("cooking_request_invalid");
    };
    response(
        tokio::task::spawn_blocking(move || {
            let recipes = state
                .recipes
                .as_ref()
                .ok_or("recipe_store_unavailable")?
                .lock()
                .map_err(|_| "recipe_store_unavailable")?;
            let mut store = state.cooking.lock().map_err(|_| "cooking_storage_failed")?;
            if input.expected_revision != store.data.revision {
                return Err("cooking_revision_conflict");
            }
            let now = time::now()?;
            let (run, previous, proposal) = match input.proposal {
                Proposal::Create { draft } => {
                    let record = source::recipe(&recipes, &draft.recipe_id, draft.recipe_revision)?;
                    let (run, proposal) = source::create(draft, &record, now)?;
                    (run, None, proposal)
                }
                Proposal::Replan { run_id, change } => {
                    let previous = store.data.run(&run_id)?.clone();
                    let (run, proposal) = operations::replan(previous.clone(), change, now)?;
                    (run, Some(previous), proposal)
                }
            };
            let value = store.prepare(actor.user.clone(), run, previous, proposal, now)?;
            view::prepared(&value, &actor)
        })
        .await,
    )
}
async fn change(
    State(state): State<Shared>,
    headers: HeaderMap,
    who: Option<Extension<Actor>>,
    body: Result<Json<Mutation>, JsonRejection>,
) -> Response {
    if !household::can_write(&state, &headers) {
        return household::reply(
            StatusCode::FORBIDDEN,
            json!({"code":"household_write_required"}),
        );
    }
    let Ok(Json(input)) = body else {
        return failure("cooking_request_invalid");
    };
    let actor = actor(who);
    response(
        tokio::task::spawn_blocking(move || {
            let mut store = state.cooking.lock().map_err(|_| "cooking_storage_failed")?;
            let run_id = execute(&mut store, input, &actor, time::now()?)?;
            Ok(json!({"revision":store.data.revision,"run_id":run_id}))
        })
        .await,
    )
}
fn execute(
    store: &mut Store,
    input: Mutation,
    actor: &Actor,
    now: i64,
) -> Result<Option<String>, &'static str> {
    let expected = input.expected_revision;
    match input.command {
        Command::Apply {
            preview_id,
            reviewed,
        } => {
            if !reviewed {
                return Err("cooking_review_required");
            }
            store
                .apply(&actor.user, &preview_id, expected, now)
                .map(Some)
        }
        Command::Timer {
            run_id,
            step_id,
            action,
        } => {
            store.change(Some(expected), |data| {
                operations::timer(data, &run_id, &step_id, action, now)
            })?;
            Ok(Some(run_id))
        }
        Command::Cancel { run_id } => {
            store.change(Some(expected), |data| operations::cancel(data, &run_id))?;
            Ok(Some(run_id))
        }
        Command::Remove { run_id } => {
            store.change(Some(expected), |data| {
                if data.run(&run_id)?.status == super::model::RunState::Active {
                    return Err("cooking_run_active");
                }
                data.runs.retain(|r| r.id != run_id);
                data.notices.retain(|n| n.run_id != run_id);
                Ok(())
            })?;
            Ok(None)
        }
        Command::Subscribe {
            run_id,
            enabled,
            language,
            permission,
        } => {
            store.change(Some(expected), |data| {
                operations::subscribe(data, &run_id, actor, enabled, language, permission, now)
            })?;
            Ok(Some(run_id))
        }
        Command::Acknowledge { notice_id } => {
            notice_action(store, expected, actor, &notice_id, false, false, now)
        }
        Command::Retry {
            notice_id,
            duplicate_risk_accepted,
        } => notice_action(
            store,
            expected,
            actor,
            &notice_id,
            true,
            duplicate_risk_accepted,
            now,
        ),
    }
}
fn notice_action(
    store: &mut Store,
    expected: u64,
    actor: &Actor,
    id: &str,
    retry: bool,
    accepted_risk: bool,
    now: i64,
) -> Result<Option<String>, &'static str> {
    if expected != store.data.revision {
        return Err("cooking_revision_conflict");
    }
    let mut run_id = None;
    store.change(None, |data| {
        let index = data
            .notices
            .iter()
            .position(|n| n.id == id && n.user == actor.user)
            .ok_or("cooking_notice_not_found")?;
        if retry {
            super::delivery::retry(data, index, accepted_risk, now)?;
        } else {
            data.notices[index].acknowledged_at = Some(now);
            if matches!(
                data.notices[index].delivery,
                super::delivery::Delivery::Queued { .. }
            ) {
                data.notices[index].delivery = super::delivery::Delivery::Cancelled;
            }
        }
        run_id = Some(data.notices[index].run_id.clone());
        Ok(())
    })?;
    Ok(run_id)
}
