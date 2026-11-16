use super::{
    Actor,
    commands::Command,
    model::Draft,
    operations::{self, PlanChange},
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
#[serde(deny_unknown_fields)]
struct Mutation {
    expected_revision: u64,
    command: Command,
    #[serde(default)]
    idempotency_key: Option<String>,
    #[serde(default)]
    occurred_at: Option<i64>,
}
pub(crate) fn routes() -> Router<Shared> {
    Router::new()
        .route("/api/household/cooking", get(list).post(change))
        .route("/api/household/cooking/template", get(template))
        .route("/api/household/cooking/calendar", get(calendar))
        .route("/api/household/cooking/preview", post(preview))
        .route(
            "/api/household/cooking/action-preview",
            post(action_preview),
        )
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
        "cooking_revision_conflict" | "cooking_request_conflict" => StatusCode::CONFLICT,
        "cooking_operation_expired" | "cooking_preview_expired" => StatusCode::GONE,
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
            store.ready()?;
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
            store.ready()?;
            let run = store.data.run(&id)?;
            let (recipe, servings) = run.food()?;
            let record = source::recipe(&recipes, &recipe.id, recipe.revision)?;
            let mut value = view::detail(
                &store.data,
                &id,
                &actor,
                time::now()?,
                &source::ingredients(&record, servings)?,
            )?;
            value["scheduler"] = store.health();
            value["step_ingredients"] = source::step_ingredients(&record);
            value["parts"] = json!(record.recipe.parts);
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
            let plans = state.plans.lock().map_err(|_| "mealplan_storage_failed")?;
            let mut store = state.cooking.lock().map_err(|_| "cooking_storage_failed")?;
            if input.expected_revision != store.data.revision {
                return Err("cooking_revision_conflict");
            }
            let now = time::now()?;
            let (run, previous, proposal) = match input.proposal {
                Proposal::Create { draft } => {
                    if let Some(slot) = &draft.plan_slot
                        && !plans.cooking_source(
                            &slot.date,
                            &slot.meal,
                            &draft.recipe_id,
                            draft.recipe_revision,
                            draft.servings,
                        )
                    {
                        return Err("cooking_plan_source_changed");
                    }
                    let record = source::recipe(&recipes, &draft.recipe_id, draft.recipe_revision)?;
                    let (run, proposal) = source::create(draft, &record, now)?;
                    (run, None, proposal)
                }
                Proposal::Replan { run_id, change } => {
                    let previous = store.data.run(&run_id)?.clone();
                    previous.food()?;
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
            let plans = state.plans.lock().map_err(|_| "mealplan_storage_failed")?;
            let mut store = state.cooking.lock().map_err(|_| "cooking_storage_failed")?;
            execute(&plans, &mut store, &input, &actor, time::now()?)
        })
        .await,
    )
}

async fn action_preview(
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
    response(tokio::task::spawn_blocking(move || {
        if !matches!(input.command,Command::Timer {..}) || !input.idempotency_key.as_deref().is_some_and(crate::recipes::Creation::valid_key) { return Err("cooking_request_invalid"); }
        let now=time::now()?;
        let at=reported_time(&input,now)?;
        let store=state.cooking.lock().map_err(|_|"cooking_storage_failed")?;
        store.ready()?;
        let Command::Timer {run_id,..}=&input.command else { return Err("cooking_request_invalid"); };
        let next=store.prepare_change(Some(input.expected_revision),|data| {
            super::commands::execute(data,input.command.clone(),&actor,at,None)?;
            Ok(())
        })?;
        Ok(json!({"revision":store.data.revision,"server_now":now,"previous":view::run(store.data.run(run_id)?,&actor,now)?,"run":view::run(next.run(run_id)?,&actor,now)?}))
    }).await)
}

fn reported_time(input: &Mutation, now: i64) -> Result<i64, &'static str> {
    let Some(at) = input.occurred_at else {
        return Ok(now);
    };
    if input.idempotency_key.is_none() || !matches!(input.command, Command::Timer { .. }) {
        return Err("cooking_request_invalid");
    }
    if !time::valid(at) || at > now {
        return Err("cooking_clock_invalid");
    }
    if now - at > 604_800 {
        return Err("cooking_operation_expired");
    }
    Ok(at)
}
fn execute(
    plans: &crate::planning::PlanStore,
    store: &mut Store,
    input: &Mutation,
    actor: &Actor,
    now: i64,
) -> Result<Value, &'static str> {
    store.ready()?;
    if let Some(key) = &input.idempotency_key {
        if !crate::recipes::Creation::valid_key(key) {
            return Err("cooking_key_invalid");
        }
        if let Some(receipt) = super::receipts::replay(
            &store.data,
            &actor.user,
            key,
            input.expected_revision,
            &input.command,
            input.occurred_at,
        )? {
            return Ok(
                json!({"revision":store.data.revision,"run_id":receipt.run_id,"applied_revision":receipt.expected_revision+1,"replayed":true}),
            );
        }
    }
    let at = reported_time(input, now)?;
    let prepared = if let Command::Apply {
        preview_id,
        reviewed,
    } = &input.command
    {
        if !reviewed {
            return Err("cooking_review_required");
        }
        let prepared = store.reviewed(&actor.user, preview_id, input.expected_revision, now)?;
        if let Some(slot) = &prepared.run.plan_slot {
            let (recipe, servings) = prepared.run.food()?;
            if !plans.cooking_source(
                &slot.date,
                &slot.meal,
                &recipe.id,
                recipe.revision,
                servings,
            ) {
                return Err("cooking_plan_source_changed");
            }
        }
        Some(prepared)
    } else {
        None
    };
    let mut run_id = None;
    store.change(Some(input.expected_revision), |data| {
        run_id = super::commands::execute(data, input.command.clone(), actor, at, prepared)?;
        if let Some(key) = input.idempotency_key.as_ref() {
            super::receipts::record(
                data,
                super::receipts::Receipt {
                    actor: actor.user.clone(),
                    key: key.clone(),
                    expected_revision: input.expected_revision,
                    command: serde_json::to_value(&input.command)
                        .map_err(|_| "cooking_storage_failed")?,
                    occurred_at: input.occurred_at,
                    recorded_at: now,
                    run_id: run_id.clone(),
                },
            )?;
        }
        Ok(())
    })?;
    if let Command::Apply { preview_id, .. } = &input.command {
        store.consume_review(preview_id);
    }
    let mut value = json!({"revision":store.data.revision,"run_id":run_id});
    if input.idempotency_key.is_some() {
        value["applied_revision"] = json!(input.expected_revision + 1);
        value["replayed"] = json!(false);
    }
    Ok(value)
}

async fn calendar(
    State(state): State<Shared>,
    headers: HeaderMap,
    Query(range): Query<super::calendar::Range>,
) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    response(
        tokio::task::spawn_blocking(move || {
            let store = state.cooking.lock().map_err(|_| "cooking_storage_failed")?;
            store.ready()?;
            super::calendar::agenda(&store.data, &range, time::now()?)
        })
        .await,
    )
}
