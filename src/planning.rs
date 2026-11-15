//! Protected, revision-pinned family plans and read-only calendar attendance imports.
mod assessment;
mod attendance;
mod week;
use crate::{
    household::{self, Household, Profiles, Shared},
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
struct Allocation {
    date: String,
    meal: String,
    servings: u32,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Preparation {
    date: String,
    time: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    allocations: Option<Vec<Allocation>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    preparation: Option<Preparation>,
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
    #[serde(default)]
    attendance: attendance::Schedule,
    #[serde(default)]
    proposals: Vec<week::Proposal>,
    #[serde(default = "week::first_id")]
    next_proposal: u64,
}
pub(crate) struct PlanStore {
    path: PathBuf,
    data: Plans,
    writable: bool,
}
impl PlanStore {
    pub(crate) fn attending(&self, id: &str, date: &str, meal: &str) -> bool {
        self.data.attendance.available(id, date, meal)
    }
    pub(crate) fn production(
        &self,
        date: &str,
        meal: &str,
        recipe: &str,
        revision: u64,
    ) -> Option<u32> {
        self.data
            .meals
            .iter()
            .find(|p| {
                p.date == date
                    && p.meal == meal
                    && p.recipe.id == recipe
                    && p.recipe.revision == revision
            })
            .map(|p| p.servings)
    }
    pub(crate) fn cooking_source(
        &self,
        date: &str,
        meal: &str,
        recipe: &str,
        revision: u64,
        servings: u32,
    ) -> bool {
        self.data.meals.iter().any(|p| {
            p.date == date
                && p.meal == meal
                && p.recipe.id == recipe
                && p.recipe.revision == revision
                && p.servings == servings
        })
    }

    pub(crate) fn shopping_recipes(
        &self,
        revision: u64,
        slots: &[crate::shopping::sources::MealSlot],
    ) -> Result<Vec<crate::shopping::sources::MealSelection>, &'static str> {
        if revision != self.data.revision {
            return Err("shopping_revision_conflict");
        }
        if slots.len() > 100
            || slots
                .iter()
                .map(|s| (&s.date, &s.meal))
                .collect::<BTreeSet<_>>()
                .len()
                != slots.len()
        {
            return Err("shopping_plan_invalid");
        }
        let mut seen = BTreeSet::new();
        let mut result = Vec::new();
        for slot in slots {
            let plan = self
                .data
                .meals
                .iter()
                .find(|p| p.date == slot.date && p.meal == slot.meal)
                .or_else(|| {
                    self.data
                        .meals
                        .iter()
                        .find(|p| includes(p, &slot.date, &slot.meal))
                })
                .ok_or("shopping_plan_invalid")?;
            if seen.insert((&plan.date, &plan.meal)) {
                result.push(crate::shopping::sources::MealSelection {
                    date: plan.date.clone(),
                    meal: plan.meal.clone(),
                    recipe: crate::shopping::sources::Selection {
                        recipe_id: plan.recipe.id.clone(),
                        revision: plan.recipe.revision,
                        servings: plan.servings,
                    },
                });
            }
        }
        Ok(result)
    }

    pub(crate) fn open(directory: &Path) -> Result<Self, String> {
        let path = directory.join("mealplan.json");
        if directory.join(".mealplan.pending").exists() {
            return Err("Interrupted meal-plan write; recover before startup".to_owned());
        }
        let mut data = Plans {
            schema_version: 1,
            revision: 0,
            meals: Vec::new(),
            attendance: attendance::Schedule::default(),
            proposals: Vec::new(),
            next_proposal: 1,
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
        validate_meals(&data.meals).map_err(str::to_owned)?;
        data.attendance.validate().map_err(str::to_owned)?;
        week::validate_store(&data).map_err(str::to_owned)?;
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
pub(crate) fn date(value: &str) -> bool {
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
fn slots(p: &Plan) -> Vec<(&str, &str)> {
    if let Some(allocations) = &p.allocations {
        allocations
            .iter()
            .map(|a| (a.date.as_str(), a.meal.as_str()))
            .collect()
    } else {
        vec![(&p.date, &p.meal)]
    }
}
fn includes(p: &Plan, date: &str, meal: &str) -> bool {
    slots(p).contains(&(date, meal))
}
fn validate_meals(meals: &[Plan]) -> Result<(), &'static str> {
    let mut occupied = BTreeSet::new();
    for p in meals {
        validate(p)?;
        for slot in slots(p) {
            if !occupied.insert(slot) {
                return Err("meal_slot_conflict");
            }
        }
    }
    Ok(())
}
fn validate(p: &Plan) -> Result<(), &'static str> {
    let allocations = p.allocations.as_deref().unwrap_or_default();
    if allocations.len() > 100 {
        return Err("invalid_meal_allocations");
    }
    let mut allocated = 0u64;
    let mut unique = BTreeSet::new();
    for a in allocations {
        if !date(&a.date)
            || a.meal.is_empty()
            || a.meal.len() > 40
            || !a.meal.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
            || a.servings == 0
            || !unique.insert((&a.date, &a.meal))
        {
            return Err("invalid_meal_allocations");
        }
        allocated += u64::from(a.servings);
    }
    if allocated > u64::from(p.servings) {
        return Err("meal_allocations_exceed_production");
    }
    if let Some(prep) = &p.preparation {
        let valid_time = prep.time.is_ascii()
            && prep.time.len() == 5
            && prep.time.as_bytes().get(2) == Some(&b':')
            && prep.time[..2].parse::<u8>().is_ok_and(|h| h < 24)
            && prep.time[3..].parse::<u8>().is_ok_and(|m| m < 60);
        if !date(&prep.date)
            || !valid_time
            || slots(p).iter().any(|(date, _)| *date < prep.date.as_str())
        {
            return Err("invalid_preparation_time");
        }
    }

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
    if (attendees.is_empty() && !slots(p).is_empty() && p.meal != "preparation")
        || (attendees.is_empty() && !allocations.is_empty())
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
    for line in record.recipe.input_lines() {
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
fn planning_conflicts(
    state: &Household,
    profiles: &Profiles,
    plans: &Plans,
    p: &Plan,
    people: &[String],
    active: u64,
) -> Result<Vec<&'static str>, &'static str> {
    let mut conflicts =
        household::people::planning_conflicts(profiles, people, &p.date, &p.meal, active);
    let shopping = state
        .shopping
        .lock()
        .map_err(|_| "shopping_storage_failed")?;
    shopping.ready()?;
    for batch in shopping
        .data
        .inventory
        .prepared
        .iter()
        .filter(|b| !b.reversed)
    {
        let origin = batch
            .source
            .as_ref()
            .is_some_and(|s| s.date == p.date && s.meal == p.meal);
        if origin {
            if batch.recipe_id != p.recipe.id
                || batch.revision != p.recipe.revision
                || batch.planned_portions != Some(p.servings)
            {
                conflicts.push("prepared_production_frozen");
            }
        } else if batch.allocations.iter().any(|a| {
            a.state != crate::inventory::model::AllocationState::Released
                && includes(p, &a.date, &a.meal)
        }) {
            conflicts.push("prepared_slot_conflict");
        }
    }
    drop(shopping);
    for (date, meal) in slots(p) {
        if people
            .iter()
            .any(|id| !plans.attendance.available(id, date, meal))
        {
            conflicts.push("attendance_excluded");
        }
    }
    if p.allocations.is_some() {
        // Attendance applies to eating slots, not the separate preparation slot.
        conflicts.retain(|code| *code != "person_unavailable");
        for (date, meal) in slots(p) {
            conflicts.extend(household::people::planning_conflicts(
                profiles, people, date, meal, 0,
            ));
        }
        conflicts.sort_unstable();
        conflicts.dedup();
    }
    Ok(conflicts)
}
fn preview(
    state: &Household,
    profiles: &Profiles,
    store: &Store,
    plans: &Plans,
    p: &Plan,
    ingredients: &crate::ingredients::Store,
) -> Result<Value, &'static str> {
    validate(p)?;
    let record = load_recipe(store, p)?;
    assessment::Assessment::new(state, profiles, &record, p, ingredients)?
        .preview(state, profiles, store, plans, p, &record)
}

fn error(code: &'static str) -> Response {
    household::reply(
        match code {
            "revision_conflict" => StatusCode::CONFLICT,
            "mealplan_storage_failed"
            | "shopping_storage_failed"
            | "recipe_store_unavailable"
            | "ingredient_storage_failed"
            | "history_storage_failed" => StatusCode::SERVICE_UNAVAILABLE,
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
        .merge(attendance::routes())
        .merge(week::routes())
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
    work(state,|state,profiles,recipes,plans,ingredients| {
        let mut quantities:BTreeMap<String,u128>=BTreeMap::new();
        let mut revisions = BTreeMap::new();
        for plan in &plans.data.meals {
            let record = load_recipe(recipes, plan)?;
            revisions.entry((record.id.clone(), record.revision)).or_insert_with(|| json!({"id":record.id,"revision":record.revision,"title":record.recipe.title}));
            for (id, atoms) in masses(&record, plan.servings)? { *quantities.entry(id).or_default() += atoms; }
        }
        let revisions = revisions.into_values().collect::<Vec<_>>();
        let requirements=quantities.into_iter().map(|(id,n)|signed_atoms(n).map(|n|json!({"ingredient_id":id,"grams":decimal(n)}))).collect::<Result<Vec<_>,_>>()?;
        let allergen_version = household::allergen_version(state, ingredients);
        let pending=plans.data.meals.iter().filter(|p|p.reviewed_profile_revision!=Some(profiles.revision) || p.reviewed_allergen_version.as_deref()!=Some(allergen_version.as_str())).map(|p|json!({"date":p.date,"meal":p.meal})).collect::<Vec<_>>();
        Ok(json!({"schema_version":1,"revision":plans.data.revision,"meals":plans.data.meals,"profile_revision":profiles.revision,"review_required_slots":pending,"shopping_requirements":requirements,"recipe_revisions":revisions,"attendance":plans.data.attendance}))
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
    work(
        state,
        move |state, profiles, recipes, plans, ingredients| {
            preview(state, profiles, recipes, &plans.data, &plan, ingredients)
        },
    )
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
    work(
        state,
        move |state, profiles, recipes, plans, ingredients| {
            if input.expected_revision != plans.data.revision
                || input.profile_revision != profiles.revision
            {
                return Err("revision_conflict");
            }
            validate(&input.plan)?;
            let result = if input.remove {
                removal(recipes, &plans.data, &input.plan)?
            } else {
                preview(
                    state,
                    profiles,
                    recipes,
                    &plans.data,
                    &input.plan,
                    ingredients,
                )?
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
                    Some(household::allergen_version(state, ingredients));
                meals.push(plan);
            }
            validate_meals(&meals)?;
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
                ..plans.data.clone()
            })?;
            Ok(json!({"revision":revision,"preview":result}))
        },
    )
    .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Suggestions {
    template: Plan,
    candidates: Vec<RecipeRef>,
    #[serde(default)]
    page: Option<usize>,
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
    let page = input.page.unwrap_or(1);
    if input.candidates.is_empty()
        || input.candidates.len() > 1000
        || page == 0
        || page.checked_mul(20).is_none()
        || input
            .candidates
            .iter()
            .map(|r| &r.id)
            .collect::<BTreeSet<_>>()
            .len()
            != input.candidates.len()
    {
        return error("candidate_limit");
    }
    work(state, move |state, profiles, recipes, plans, ingredients| {
        let feedback = state.feedback.lock().map_err(|_| "history_storage_failed")?;
        let mut feasible = Vec::new();
        let mut rejected = Vec::new();
        for candidate in input.candidates {
            let mut plan = input.template.clone();
            plan.recipe = candidate;
            match preview(state, profiles, recipes, &plans.data, &plan, ingredients) {
                Ok(value) if value["feasible"] == true => {
                    feasible.push(crate::feedback::ranking::Candidate {
                        id: plan.recipe.id,
                        revision: plan.recipe.revision,
                        dislikes: value["review"]["disliked_ingredients"].as_array().map_or(0,Vec::len),
                    });
                }
                Ok(value) => rejected.push(json!({"recipe":plan.recipe,"conflicts":value["conflicts"]})),
                Err(code) => rejected.push(json!({"recipe":plan.recipe,"error":code})),
            }
        }
        let ranked = crate::feedback::ranking::rank(feasible, &feedback.data, &attendees(&input.template), &input.template.date)?;
        let total = ranked.len();
        let page = page.min(total.div_ceil(20).max(1));
        let mut matches = Vec::new();
        for candidate in ranked.into_iter().skip((page-1)*20).take(20) {
            let mut plan = input.template.clone();
            plan.recipe = RecipeRef {id:candidate.candidate.id, revision:candidate.candidate.revision};
            let mut value = preview(state, profiles, recipes, &plans.data, &plan, ingredients)?;
            value["recommendation"] = serde_json::to_value(candidate.explanation).map_err(|_| "history_storage_failed")?;
            matches.push(value);
        }
        Ok(json!({"suggestions":matches,"rejected":rejected,"reason":if total==0{Some("no_feasible_meal")}else{None},
            "page":page,"per_page":20,"total":total,"history_revision":feedback.data.revision,"learning_epoch":feedback.data.epoch,
            "model_version":if feedback.data.policy.enabled {crate::feedback::ranking::LEARNED_VERSION}else{crate::feedback::ranking::BASELINE_VERSION}}))
    }).await
}

async fn work(
    state: Shared,
    action: impl FnOnce(
        &Household,
        &Profiles,
        &Store,
        &mut PlanStore,
        &crate::ingredients::Store,
    ) -> Result<Value, &'static str>
    + Send
    + 'static,
) -> Response {
    tokio::task::spawn_blocking(move || {
        let Ok(profiles) = state.store.lock() else {
            return error("mealplan_storage_failed");
        };
        let Some(recipes) = &state.recipes else {
            return error("recipe_store_unavailable");
        };
        let Ok(recipes) = recipes.lock() else {
            return error("recipe_store_unavailable");
        };
        let Ok(mut plans) = state.plans.lock() else {
            return error("mealplan_storage_failed");
        };
        let Ok(ingredients) = state.catalogue.lock() else {
            return error("ingredient_storage_failed");
        };
        match action(&state, &profiles.data, &recipes, &mut plans, &ingredients) {
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
    work(state, move |_, profiles, recipes, plans, _| {
        let mut result = removal(recipes, &plans.data, &plan)?;
        result["profile_revision"] = json!(profiles.revision);
        Ok(result)
    })
    .await
}

fn signed_atoms(atoms: u128) -> Result<i128, &'static str> {
    i128::try_from(atoms).map_err(|_| "quantity_overflow")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod production_tests {
    use super::*;
    #[test]
    fn selecting_two_consumption_days_exports_one_production_to_shopping() {
        let data: Plans = serde_json::from_value(json!({"schema_version":1,"revision":7,"meals":[{"date":"2026-11-12","meal":"lunch","recipe":{"id":"recipe-00000001","revision":1},"attendees":["person"],"calendar":null,"servings":4,"active_seconds":600,"elapsed_seconds":3600,"advance_seconds":0,"allocations":[{"date":"2026-11-12","meal":"lunch","servings":2},{"date":"2026-11-13","meal":"lunch","servings":2}]}]})).unwrap();
        validate_meals(&data.meals).unwrap();
        let store = PlanStore {
            path: PathBuf::new(),
            data,
            writable: true,
        };
        let slots = [
            crate::shopping::sources::MealSlot {
                date: "2026-11-12".into(),
                meal: "lunch".into(),
            },
            crate::shopping::sources::MealSlot {
                date: "2026-11-13".into(),
                meal: "lunch".into(),
            },
        ];
        let selections = store.shopping_recipes(7, &slots).unwrap();
        assert_eq!(selections.len(), 1);
        assert_eq!(selections[0].recipe.servings, 4);
        assert!(store.shopping_recipes(6, &slots).is_err());
    }

    #[test]
    fn explicit_empty_allocations_retain_production_without_consumption_slots() {
        let legacy = json!({"date":"2026-11-12","meal":"lunch","recipe":{"id":"recipe-00000001","revision":1},"attendees":[],"calendar":null,"servings":4,"active_seconds":600,"elapsed_seconds":3600,"advance_seconds":0});
        let mut plan: Plan = serde_json::from_value(legacy).unwrap();
        assert!(validate(&plan).is_err());
        plan.allocations = Some(Vec::new());
        validate(&plan).unwrap();
        assert_eq!(slots(&plan), [] as [(&str, &str); 0]);
        let saved = serde_json::to_value(&plan).unwrap();
        assert_eq!(saved["allocations"], json!([]));
        let reopened: Plan = serde_json::from_value(saved).unwrap();
        assert_eq!(slots(&reopened), [] as [(&str, &str); 0]);
        let store = PlanStore {
            path: PathBuf::new(),
            data: Plans {
                schema_version: 1,
                revision: 1,
                meals: vec![reopened],
                attendance: attendance::Schedule::default(),
                proposals: Vec::new(),
                next_proposal: 1,
            },
            writable: true,
        };
        let selections = store
            .shopping_recipes(
                1,
                &[crate::shopping::sources::MealSlot {
                    date: "2026-11-12".into(),
                    meal: "lunch".into(),
                }],
            )
            .unwrap();
        assert_eq!(selections[0].recipe.servings, 4);
    }
}
