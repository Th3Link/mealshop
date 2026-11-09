//! Protected, revision-pinned family plans and read-only calendar attendance imports.
use crate::{
    household::{self, Evaluation, Household, Profiles, Shared},
    quantities::SCALE,
    recipes::{Record, Store},
};
use axum::{
    Json, Router,
    extract::{State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RecipeRef {
    id: String,
    revision: u64,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Calendar {
    consent: bool,
    source: String,
    event_id: String,
    date: String,
    member_ids: Vec<String>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    date: String,
    meal: String,
    recipe: RecipeRef,
    attendees: Option<Vec<String>>,
    calendar: Option<Calendar>,
    servings: u32,
    active_seconds: u64,
    elapsed_seconds: u64,
    advance_seconds: u64,
    #[serde(default)]
    reviewed_profile_revision: Option<u64>,
    #[serde(default)]
    reviewed_allergen_version: Option<String>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Plans {
    schema_version: u32,
    revision: u64,
    meals: Vec<Plan>,
}
pub(crate) struct PlanStore {
    path: PathBuf,
    data: Plans,
    writable: bool,
}
impl PlanStore {
    pub(crate) fn open(directory: &Path) -> Result<Self, String> {
        let path = directory.join("mealplan.json");
        if directory.join(".mealplan.pending").exists() {
            return Err("Interrupted meal-plan write; recover before startup".to_owned());
        }
        let mut data = Plans {
            schema_version: 1,
            revision: 0,
            meals: Vec::new(),
        };
        if path.exists() {
            let mut bytes = Vec::new();
            File::open(&path)
                .map_err(|_| "Cannot read meal plan")?
                .take(1_048_577)
                .read_to_end(&mut bytes)
                .map_err(|_| "Cannot read meal plan")?;
            if bytes.len() > 1_048_576 {
                return Err("Meal plan exceeds 1 MiB".to_owned());
            }
            data = serde_json::from_slice(&bytes).map_err(|_| "Invalid meal plan")?;
        }
        if data.schema_version != 1 || data.meals.len() > 100 {
            return Err("Invalid meal-plan schema or capacity".to_owned());
        }
        let mut keys = BTreeSet::new();
        for p in &data.meals {
            validate(p).map_err(str::to_owned)?;
            if !keys.insert((&p.date, &p.meal)) {
                return Err("Duplicate meal slot".to_owned());
            }
        }
        Ok(Self {
            path,
            data,
            writable: true,
        })
    }
    fn save(&mut self, data: Plans) -> Result<(), &'static str> {
        let bytes = serde_json::to_vec(&data).map_err(|_| "mealplan_storage_failed")?;
        if bytes.len() > 1_048_576 {
            return Err("mealplan_capacity");
        }
        if !self.writable {
            return Err("mealplan_storage_failed");
        }
        let persist = || -> std::io::Result<()> {
            let pending = self.path.with_file_name(".mealplan.pending");
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
            fs::rename(pending, &self.path)?;
            #[cfg(unix)]
            File::open(
                self.path
                    .parent()
                    .ok_or_else(|| std::io::Error::other("parent"))?,
            )?
            .sync_all()?;
            Ok(())
        };
        if persist().is_err() {
            self.writable = false;
            return Err("mealplan_storage_failed");
        }
        self.data = data;
        Ok(())
    }
}
fn date(value: &str) -> bool {
    let b = value.as_bytes();
    if b.len() != 10
        || b[4] != b'-'
        || b[7] != b'-'
        || b.iter()
            .enumerate()
            .any(|(i, c)| i != 4 && i != 7 && !c.is_ascii_digit())
    {
        return false;
    }
    let year = value[..4].parse::<u32>().unwrap_or(0);
    let month = value[5..7].parse::<u32>().unwrap_or(0);
    let day = value[8..].parse::<u32>().unwrap_or(0);
    let max = match month {
        4 | 6 | 9 | 11 => 30,
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    };
    year > 0 && day > 0 && day <= max
}
fn validate(p: &Plan) -> Result<(), &'static str> {
    if !date(&p.date)
        || p.meal.is_empty()
        || p.meal.len() > 40
        || !p.meal.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
        || p.servings == 0
        || p.servings > 1_000_000
        || p.recipe.revision == 0
        || !crate::recipes::valid_id(&p.recipe.id)
        || p.active_seconds > 86_400
        || p.elapsed_seconds > 86_400
        || p.advance_seconds > 2_592_000
    {
        return Err("invalid_meal_plan");
    }
    if let Some(c) = &p.calendar {
        if !c.consent
            || c.source.trim().is_empty()
            || c.source.len() > 200
            || c.event_id.trim().is_empty()
            || c.event_id.len() > 200
            || c.date != p.date
            || c.member_ids.is_empty()
            || c.member_ids.len() > 100
        {
            return Err("calendar_consent_or_source_required");
        }
    }
    let attendees = p
        .attendees
        .as_ref()
        .or(p.calendar.as_ref().map(|c| &c.member_ids))
        .ok_or("attendees_required")?;
    if attendees.is_empty()
        || attendees.len() > 100
        || attendees.iter().collect::<BTreeSet<_>>().len() != attendees.len()
    {
        return Err("invalid_attendees");
    }
    Ok(())
}
fn attendees(p: &Plan) -> Vec<String> {
    p.attendees
        .clone()
        .or_else(|| p.calendar.as_ref().map(|c| c.member_ids.clone()))
        .unwrap_or_default()
}
fn load_recipe(store: &Store, p: &Plan) -> Result<Record, &'static str> {
    store
        .get(&p.recipe.id, Some(p.recipe.revision))
        .map_err(|_| "recipe_revision_not_found")
}
fn masses(record: &Record, servings: u32) -> Result<BTreeMap<String, u128>, &'static str> {
    let mut result = BTreeMap::new();
    for line in &record.recipe.lines {
        let atoms = line
            .grams
            .0
            .scaled(servings, record.recipe.servings)
            .map_err(|_| "scaled_quantity_out_of_range")?
            .atoms();
        *result.entry(line.ingredient_id.clone()).or_insert(0) += atoms;
    }
    Ok(result)
}
fn decimal(atoms: i128) -> String {
    let sign = if atoms < 0 { "-" } else { "" };
    let atoms = atoms.unsigned_abs();
    format!("{sign}{}.{:06}", atoms / SCALE, atoms % SCALE)
}
fn preview(
    state: &Household,
    profiles: &Profiles,
    store: &Store,
    plans: &Plans,
    p: &Plan,
) -> Result<Value, &'static str> {
    validate(p)?;
    let record = load_recipe(store, p)?;
    let people = attendees(p);
    let review = household::review(
        state,
        profiles,
        &Evaluation {
            member_ids: people.clone(),
            ingredient_ids: record
                .recipe
                .lines
                .iter()
                .map(|l| l.ingredient_id.clone())
                .collect(),
        },
    )?;
    let active: u64 = record
        .recipe
        .steps
        .iter()
        .map(|s| u64::from(s.active_seconds))
        .sum();
    let elapsed = active
        + record
            .recipe
            .steps
            .iter()
            .map(|s| u64::from(s.passive_seconds))
            .sum::<u64>();
    let mut conflicts = Vec::new();
    if review["status"] == "blocked" {
        conflicts.push("dietary_exclusion");
    }
    if record.recipe.steps.is_empty() {
        conflicts.push("preparation_time_unknown");
    }
    if active > p.active_seconds {
        conflicts.push("active_time_exceeded");
    }
    if elapsed > p.elapsed_seconds + p.advance_seconds {
        conflicts.push("elapsed_time_exceeded");
    }
    let mut delta = masses(&record, p.servings)?
        .into_iter()
        .map(|(id, n)| signed_atoms(n).map(|n| (id, n)))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    if let Some(old) = plans
        .meals
        .iter()
        .find(|old| old.date == p.date && old.meal == p.meal)
    {
        for (id, n) in masses(&load_recipe(store, old)?, old.servings)? {
            *delta.entry(id).or_insert(0) -= i128::try_from(n).map_err(|_| "quantity_overflow")?;
        }
    }
    let nutrition = crate::nutrition::scaled_nutrition(
        &record.catalogue,
        &record.recipe.calculation(),
        p.servings,
    )
    .map_err(|_| "scaled_quantity_out_of_range")?;
    Ok(
        json!({"plan":p,"profile_revision":profiles.revision,"plan_revision":plans.revision,"attendees":people,"attendance_source":p.calendar.as_ref().map(|c|&c.source),"attendance_overridden":p.attendees.is_some() && p.calendar.is_some(),"review":review,"conflicts":conflicts,"feasible":conflicts.is_empty(),"timing":{"reference_servings":record.recipe.servings,"active_seconds":active,"elapsed_seconds":elapsed,"advance_preparation_seconds":elapsed.saturating_sub(p.elapsed_seconds)},"shopping_delta":delta.into_iter().filter(|(_,n)|*n!=0).map(|(id,n)|json!({"ingredient_id":id,"grams_delta":decimal(n)})).collect::<Vec<_>>(),"nutrition":nutrition}),
    )
}
fn error(code: &'static str) -> Response {
    household::reply(
        match code {
            "revision_conflict" => StatusCode::CONFLICT,
            "mealplan_storage_failed" | "recipe_store_unavailable" => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            _ => StatusCode::UNPROCESSABLE_ENTITY,
        },
        json!({"code":code}),
    )
}
fn access(state: &Household, headers: &HeaderMap, write: bool) -> bool {
    if write {
        household::can_write(state, headers)
    } else {
        household::authorized(state, headers)
    }
}
pub(crate) fn routes() -> Router<Shared> {
    Router::new()
        .route("/api/household/mealplan", get(list))
        .route("/api/household/mealplan/preview", post(show_preview))
        .route("/api/household/mealplan/apply", post(apply))
        .route(
            "/api/household/mealplan/remove-preview",
            post(remove_preview),
        )
        .route("/api/household/mealplan/suggestions", post(suggest))
}
async fn list(State(state): State<Shared>, headers: HeaderMap) -> Response {
    if !access(&state, &headers, false) {
        return household::reply(
            StatusCode::UNAUTHORIZED,
            json!({"code":"household_access_required"}),
        );
    }
    work(state,|state,profiles,recipes,plans| {
        let mut quantities:BTreeMap<String,u128>=BTreeMap::new();
        let mut revisions = BTreeMap::new();
        for plan in &plans.data.meals {
            let record = load_recipe(recipes, plan)?;
            revisions.entry((record.id.clone(), record.revision)).or_insert_with(|| json!({"id":record.id,"revision":record.revision,"title":record.recipe.title}));
            for (id, atoms) in masses(&record, plan.servings)? { *quantities.entry(id).or_default() += atoms; }
        }
        let revisions = revisions.into_values().collect::<Vec<_>>();
        let requirements=quantities.into_iter().map(|(id,n)|signed_atoms(n).map(|n|json!({"ingredient_id":id,"grams":decimal(n)}))).collect::<Result<Vec<_>,_>>()?;
        let pending=plans.data.meals.iter().filter(|p|p.reviewed_profile_revision!=Some(profiles.revision) || p.reviewed_allergen_version.as_deref()!=state.allergen_data["version"].as_str()).map(|p|json!({"date":p.date,"meal":p.meal})).collect::<Vec<_>>();
        Ok(json!({"schema_version":1,"revision":plans.data.revision,"meals":plans.data.meals,"profile_revision":profiles.revision,"review_required_slots":pending,"shopping_requirements":requirements,"recipe_revisions":revisions}))
    }).await
}
async fn show_preview(
    State(state): State<Shared>,
    headers: HeaderMap,
    body: Result<Json<Plan>, JsonRejection>,
) -> Response {
    if !access(&state, &headers, false) {
        return household::reply(
            StatusCode::UNAUTHORIZED,
            json!({"code":"household_access_required"}),
        );
    }
    let Ok(Json(plan)) = body else {
        return error("invalid_meal_plan");
    };
    work(state, move |state, profiles, recipes, plans| {
        preview(state, profiles, recipes, &plans.data, &plan)
    })
    .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Apply {
    plan: Plan,
    expected_revision: u64,
    profile_revision: u64,
    reviewed: bool,
    #[serde(default)]
    remove: bool,
}
async fn apply(
    State(state): State<Shared>,
    headers: HeaderMap,
    body: Result<Json<Apply>, JsonRejection>,
) -> Response {
    if !access(&state, &headers, true) {
        return household::reply(
            StatusCode::FORBIDDEN,
            json!({"code":"household_write_required"}),
        );
    }
    let Ok(Json(input)) = body else {
        return error("invalid_meal_plan");
    };
    work(state, move |state, profiles, recipes, plans| {
        if input.expected_revision != plans.data.revision
            || input.profile_revision != profiles.revision
        {
            return Err("revision_conflict");
        }
        validate(&input.plan)?;
        let result = if input.remove {
            removal(recipes, &plans.data, &input.plan)?
        } else {
            preview(state, profiles, recipes, &plans.data, &input.plan)?
        };
        if !input.remove && result["feasible"] != true {
            return Err("meal_constraints_conflict");
        }
        if !input.reviewed {
            return Err("dietary_review_required");
        }
        let mut meals = plans.data.meals.clone();
        meals.retain(|p| p.date != input.plan.date || p.meal != input.plan.meal);
        if !input.remove {
            let mut plan = input.plan;
            plan.reviewed_profile_revision = Some(profiles.revision);
            plan.reviewed_allergen_version =
                state.allergen_data["version"].as_str().map(str::to_owned);
            meals.push(plan);
        }
        if meals.len() > 100 {
            return Err("mealplan_capacity");
        }
        let revision = plans
            .data
            .revision
            .checked_add(1)
            .ok_or("mealplan_capacity")?;
        plans.save(Plans {
            schema_version: 1,
            revision,
            meals,
        })?;
        Ok(json!({"revision":revision,"preview":result}))
    })
    .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Suggestions {
    template: Plan,
    candidates: Vec<RecipeRef>,
}
async fn suggest(
    State(state): State<Shared>,
    headers: HeaderMap,
    body: Result<Json<Suggestions>, JsonRejection>,
) -> Response {
    if !access(&state, &headers, false) {
        return household::reply(
            StatusCode::UNAUTHORIZED,
            json!({"code":"household_access_required"}),
        );
    }
    let Ok(Json(input)) = body else {
        return error("invalid_meal_plan");
    };
    if input.candidates.is_empty() || input.candidates.len() > 50 {
        return error("candidate_limit");
    }
    work(state,move|state,profiles,recipes,plans| {
        let mut matches=Vec::new(); let mut rejected=Vec::new();
        for candidate in input.candidates {
            let mut plan=input.template.clone(); plan.recipe=candidate;
            match preview(state,profiles,recipes,&plans.data,&plan) {
                Ok(value)=> {if value["feasible"]==true {matches.push(value);} else {rejected.push(value);}},
                Err(code)=>rejected.push(json!({"recipe":plan.recipe,"error":code})),
            }
        }
        matches.sort_by_key(|v|v["review"]["disliked_ingredients"].as_array().map_or(0,Vec::len));
        Ok(json!({"suggestions":matches,"rejected":rejected,"reason":if matches.is_empty(){Some("no_feasible_meal")}else{None}}))
    }).await
}
async fn work(
    state: Shared,
    action: impl FnOnce(&Household, &Profiles, &Store, &mut PlanStore) -> Result<Value, &'static str>
    + Send
    + 'static,
) -> Response {
    tokio::task::spawn_blocking(move || {
        let Some(recipes) = &state.recipes else {
            return error("recipe_store_unavailable");
        };
        let Ok(recipes) = recipes.lock() else {
            return error("recipe_store_unavailable");
        };
        let Ok(profiles) = state.store.lock() else {
            return error("mealplan_storage_failed");
        };
        let Ok(mut plans) = state.plans.lock() else {
            return error("mealplan_storage_failed");
        };
        match action(&state, &profiles.data, &recipes, &mut plans) {
            Ok(value) => household::reply(StatusCode::OK, value),
            Err(code) => error(code),
        }
    })
    .await
    .unwrap_or_else(|_| error("mealplan_storage_failed"))
}

fn removal(recipes: &Store, plans: &Plans, p: &Plan) -> Result<Value, &'static str> {
    validate(p)?;
    let old = plans
        .meals
        .iter()
        .find(|old| old.date == p.date && old.meal == p.meal)
        .ok_or("meal_slot_not_found")?;
    let delta = masses(&load_recipe(recipes, old)?, old.servings)?
        .into_iter()
        .map(|(id, n)| {
            signed_atoms(n).map(|n| json!({"ingredient_id":id,"grams_delta":decimal(-n)}))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(
        json!({"removed":old,"plan_revision":plans.revision,"shopping_delta":delta,"feasible":true,"conflicts":[]}),
    )
}
async fn remove_preview(
    State(state): State<Shared>,
    headers: HeaderMap,
    body: Result<Json<Plan>, JsonRejection>,
) -> Response {
    if !access(&state, &headers, false) {
        return household::reply(
            StatusCode::UNAUTHORIZED,
            json!({"code":"household_access_required"}),
        );
    }
    let Ok(Json(plan)) = body else {
        return error("invalid_meal_plan");
    };
    work(state, move |_, profiles, recipes, plans| {
        let mut result = removal(recipes, &plans.data, &plan)?;
        result["profile_revision"] = json!(profiles.revision);
        Ok(result)
    })
    .await
}

fn signed_atoms(atoms: u128) -> Result<i128, &'static str> {
    i128::try_from(atoms).map_err(|_| "quantity_overflow")
}
