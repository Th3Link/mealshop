use super::{
    Plan, Plans, RecipeRef, access, attendance, date, decimal, error, includes, load_recipe,
    masses, preview, signed_atoms, validate, validate_meals, work,
};
use crate::{
    household::{self, Household, Profiles, Shared},
    recipes::Store,
};
use axum::{
    Json, Router,
    extract::{State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::post,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn first_id() -> u64 {
    1
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Proposal {
    id: String,
    expires_at: u64,
    plan_revision: u64,
    profile_revision: u64,
    ingredient_revision: u64,
    allergen_version: String,
    candidates: Vec<RecipeRef>,
    slots: Vec<WeekSlot>,
    applied_revision: Option<u64>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WeekSlot {
    date: String,
    meal: String,
    plan: Option<Plan>,
    reason: Option<String>,
    evidence: Value,
}
pub(super) fn validate_store(data: &Plans) -> Result<(), &'static str> {
    let mut ids = BTreeSet::new();
    if data.next_proposal == 0 || data.proposals.len() > 5 {
        return Err("week_capacity");
    }
    for p in &data.proposals {
        let n =
            p.id.strip_prefix("week-")
                .and_then(|n| n.parse::<u64>().ok());
        if !n.is_some_and(|n| n > 0 && n < data.next_proposal && p.id == format!("week-{n}"))
            || !ids.insert(&p.id)
            || p.slots.len() > 28
            || p.candidates.len() > 100
            || p.expires_at == 0
            || p.plan_revision > data.revision
            || p.applied_revision.is_some_and(|r| r > data.revision)
            || p.candidates
                .iter()
                .any(|r| !crate::recipes::valid_id(&r.id) || r.revision == 0)
            || p.candidates
                .iter()
                .map(|r| &r.id)
                .collect::<BTreeSet<_>>()
                .len()
                != p.candidates.len()
        {
            return Err("week_invalid");
        }
        let mut occupied = BTreeSet::new();
        for s in &p.slots {
            if !date(&s.date) || !attendance::meal(&s.meal) || !occupied.insert((&s.date, &s.meal))
            {
                return Err("week_invalid");
            }
            if let Some(plan) = &s.plan {
                validate(plan)?;
                if plan.date != s.date
                    || plan.meal != s.meal
                    || !p
                        .candidates
                        .iter()
                        .any(|r| r.id == plan.recipe.id && r.revision == plan.recipe.revision)
                {
                    return Err("week_invalid");
                }
            }
        }
    }
    Ok(())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    start_date: String,
    meals: Vec<String>,
    candidates: Vec<RecipeRef>,
    active_seconds: u64,
    elapsed_seconds: u64,
    advance_seconds: u64,
    servings_per_person: u32,
    prefer_quick: bool,
    prefer_reuse: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfirmedSlot {
    date: String,
    meal: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ApplyWeek {
    proposal_id: String,
    confirmed_slots: Vec<ConfirmedSlot>,
    reviewed: bool,
}

pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/api/household/mealplan/week/preview", post(generate))
        .route("/api/household/mealplan/week/apply", post(confirm))
}
fn now() -> Result<u64, &'static str> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|n| n.as_secs())
        .map_err(|_| "mealplan_storage_failed")
}
async fn generate(
    State(state): State<Shared>,
    headers: HeaderMap,
    input: Result<Json<Input>, JsonRejection>,
) -> Response {
    if !access(&state, &headers, true) {
        return household::reply(
            StatusCode::FORBIDDEN,
            json!({"code":"household_write_required"}),
        );
    }
    let Ok(Json(input)) = input else {
        return error("week_invalid");
    };
    work(state, move |state, profiles, recipes, plans, ingredients| {
        let start = input.start_date.parse::<jiff::civil::Date>().map_err(|_| "week_invalid")?;
        if !date(&input.start_date) || !date(&start.checked_add(jiff::Span::new().days(6)).map_err(|_| "week_invalid")?.to_string())
            || input.meals.is_empty() || input.meals.len() > 4
            || input.meals.iter().any(|m| !attendance::meal(m))
            || input.meals.iter().collect::<BTreeSet<_>>().len() != input.meals.len()
            || input.candidates.is_empty() || input.candidates.len() > 100
            || input.candidates.iter().map(|r| &r.id).collect::<BTreeSet<_>>().len() != input.candidates.len()
            || !(1..=100).contains(&input.servings_per_person)
            || input.active_seconds > 86_400 || input.elapsed_seconds > 86_400
            || input.advance_seconds > 2_592_000 { return Err("week_invalid"); }
        for r in &input.candidates {
            if recipes.get(&r.id, None).map_err(|_| "recipe_revision_not_found")?.revision != r.revision {
                return Err("revision_conflict");
            }
        }
        let time = now()?;
        let feedback = state.feedback.lock().map_err(|_| "history_storage_failed")?;
        let mut working = plans.data.clone();
        let mut used = BTreeMap::new();
        let mut reused = BTreeSet::new();
        for p in &working.meals {
            if p.date >= start.to_string() && p.date <= start.checked_add(jiff::Span::new().days(6))
                .map_err(|_| "week_invalid")?.to_string() {
                *used.entry(p.recipe.id.clone()).or_insert(0u32) += 1;
                reused.extend(masses(&load_recipe(recipes, p)?, p.servings)?.into_keys());
            }
        }
        let mut slots = Vec::new();
        for day in 0..7 {
            let date = start.checked_add(jiff::Span::new().days(day)).map_err(|_| "week_invalid")?.to_string();
            for meal in &input.meals {
                let stock = state.shopping.lock().map_err(|_| "shopping_storage_failed")?;
                stock.ready()?;
                let prepared = stock.data.inventory.prepared.iter().filter(|b| !b.reversed)
                    .flat_map(|b| &b.allocations).any(|a| a.date == date && a.meal == *meal
                        && a.state != crate::inventory::model::AllocationState::Released);
                drop(stock);
                if prepared {
                    slots.push(WeekSlot { date: date.clone(), meal: meal.clone(), plan: None,
                        reason: Some("prepared_portions_available".to_owned()), evidence: json!({}) });
                    continue;
                }
                if working.meals.iter().any(|p| includes(p, &date, meal) || (p.date == date && p.meal == *meal)) {
                    slots.push(WeekSlot { date: date.clone(), meal: meal.clone(), plan: None,
                        reason: Some("existing_meal_preserved".to_owned()), evidence: json!({}) });
                    continue;
                }
                let people = profiles.members.iter().filter(|m| {
                    working.attendance.available(&m.id, &date, meal) &&
                    !household::people::planning_conflicts(profiles, std::slice::from_ref(&m.id), &date, meal, 0)
                        .contains(&"person_unavailable")
                }).map(|m| m.id.clone()).collect::<Vec<_>>();
                let (plan, reason, evidence) = if people.is_empty() {
                    (None, Some("no_attendees".to_owned()), json!({}))
                } else {
                    choose(state, profiles, recipes, ingredients, &working, &input, &feedback.data,
                        &date, meal, &people, &used, &reused)?
                };
                if let Some(p) = &plan {
                    *used.entry(p.recipe.id.clone()).or_insert(0u32) += 1;
                    reused.extend(masses(&load_recipe(recipes, p)?, p.servings)?.into_keys());
                    working.meals.push(p.clone());
                }
                slots.push(WeekSlot { date: date.clone(), meal: meal.clone(), plan, reason, evidence });
            }
        }
        validate_meals(&working.meals)?;
        if working.meals.len() > 100 { return Err("mealplan_capacity"); }
        let proposal = Proposal {
            id: format!("week-{}", plans.data.next_proposal), expires_at: time + 1800,
            plan_revision: plans.data.revision, profile_revision: profiles.revision,
            ingredient_revision: ingredients.revision(), allergen_version: household::allergen_version(state, ingredients),
            candidates: input.candidates, slots, applied_revision: None,
        };
        let delta = delta(recipes, &plans.data.meals, &working.meals)?;
        let mut next = plans.data.clone();
        next.proposals.retain(|p| p.expires_at > time);
        if next.proposals.len() >= 5 { return Err("week_capacity"); }
        next.proposals.push(proposal.clone());
        next.next_proposal = next.next_proposal.checked_add(1).ok_or("week_capacity")?;
        plans.save(next)?;
        Ok(json!({"proposal":proposal,"shopping_delta":delta,
            "model_version":"household-week-v1","criteria":["declared_dislikes","opted_in_history","variety","ingredient_reuse","active_time"],
            "limits":["deterministic_greedy_not_global_optimum","cost_not_ranked_no_comparable_coverage","nutrition_not_optimized","existing_meals_preserved"],
            "history_revision":feedback.data.revision}))
    }).await
}

type Choice = (Option<Plan>, Option<String>, Value);
#[allow(clippy::too_many_arguments)]
fn choose(
    state: &Household,
    profiles: &Profiles,
    recipes: &Store,
    ingredients: &crate::ingredients::Store,
    plans: &Plans,
    input: &Input,
    feedback: &crate::feedback::Data,
    date: &str,
    meal: &str,
    people: &[String],
    used: &BTreeMap<String, u32>,
    reused: &BTreeSet<String>,
) -> Result<Choice, &'static str> {
    let mut feasible = Vec::new();
    let mut rejection_counts = BTreeMap::<String, usize>::new();
    for candidate in &input.candidates {
        let plan = Plan {
            allocations: None,
            preparation: None,
            date: date.to_owned(),
            meal: meal.to_owned(),
            recipe: candidate.clone(),
            attendees: Some(people.to_vec()),
            calendar: None,
            servings: u32::try_from(people.len()).map_err(|_| "week_invalid")?
                * input.servings_per_person,
            active_seconds: input.active_seconds,
            elapsed_seconds: input.elapsed_seconds,
            advance_seconds: input.advance_seconds,
            reviewed_profile_revision: None,
            reviewed_allergen_version: None,
        };
        match preview(state, profiles, recipes, plans, &plan, ingredients) {
            Ok(p) if p["feasible"] == true => feasible.push((plan, p)),
            Ok(p) => {
                for c in p["conflicts"].as_array().into_iter().flatten() {
                    *rejection_counts
                        .entry(c.as_str().unwrap_or("meal_constraints_conflict").to_owned())
                        .or_default() += 1;
                }
            }
            Err(e) => {
                *rejection_counts.entry(e.to_owned()).or_default() += 1;
            }
        }
    }
    let ranked = crate::feedback::ranking::rank(
        feasible
            .iter()
            .map(|(plan, p)| crate::feedback::ranking::Candidate {
                id: plan.recipe.id.clone(),
                revision: plan.recipe.revision,
                dislikes: p["review"]["disliked_ingredients"]
                    .as_array()
                    .map_or(0, Vec::len),
            })
            .collect(),
        feedback,
        people,
        date,
    )?;
    let mut choices = Vec::new();
    for r in ranked {
        let (plan, p) = feasible
            .iter()
            .find(|(p, _)| p.recipe.id == r.candidate.id)
            .ok_or("week_invalid")?;
        let prior = used.get(&plan.recipe.id).copied().unwrap_or(0);
        let common = masses(&load_recipe(recipes, plan)?, plan.servings)?
            .keys()
            .filter(|id| reused.contains(*id))
            .count();
        let seconds = p["timing"]["active_seconds"].as_u64().unwrap_or(0);
        let score = i64::try_from(r.candidate.dislikes).map_err(|_| "week_invalid")? * 1000
            + i64::from(prior) * 100
            - i64::from(r.explanation.score.clamp(-100, 100))
            + if input.prefer_quick {
                i64::try_from(seconds / 60).map_err(|_| "week_invalid")?
            } else {
                0
            }
            - if input.prefer_reuse {
                i64::try_from(common.min(10)).map_err(|_| "week_invalid")? * 2
            } else {
                0
            };
        choices.push((
            score,
            plan.clone(),
            json!({"score":score,"prior_week_uses":prior,
            "reused_ingredients":common,"active_seconds":seconds,"history":r.explanation,
            "review":p["review"],"rejected_by_constraint":rejection_counts}),
        ));
    }
    choices.sort_by(|a, b| (a.0, &a.1.recipe.id).cmp(&(b.0, &b.1.recipe.id)));
    Ok(choices.into_iter().next().map_or_else(
        || {
            (
                None,
                Some("no_feasible_meal".to_owned()),
                json!({"rejected_by_constraint":rejection_counts}),
            )
        },
        |(_, p, e)| (Some(p), None, e),
    ))
}
fn delta(recipes: &Store, before: &[Plan], after: &[Plan]) -> Result<Vec<Value>, &'static str> {
    let mut amounts = BTreeMap::<String, i128>::new();
    for (plans, sign) in [(before, -1i128), (after, 1)] {
        for p in plans {
            for (id, n) in masses(&load_recipe(recipes, p)?, p.servings)? {
                *amounts.entry(id).or_default() += sign * signed_atoms(n)?;
            }
        }
    }
    Ok(amounts
        .into_iter()
        .filter(|(_, n)| *n != 0)
        .map(|(id, n)| json!({"ingredient_id":id,"grams_delta":decimal(n)}))
        .collect())
}
async fn confirm(
    State(state): State<Shared>,
    headers: HeaderMap,
    input: Result<Json<ApplyWeek>, JsonRejection>,
) -> Response {
    if !access(&state, &headers, true) {
        return household::reply(
            StatusCode::FORBIDDEN,
            json!({"code":"household_write_required"}),
        );
    }
    let Ok(Json(input)) = input else {
        return error("week_invalid");
    };
    work(
        state,
        move |state, profiles, recipes, plans, ingredients| {
            let proposal = plans
                .data
                .proposals
                .iter()
                .find(|p| p.id == input.proposal_id)
                .cloned()
                .ok_or("week_expired")?;
            let expected = proposal
                .slots
                .iter()
                .filter(|s| s.plan.is_some())
                .map(|s| (&s.date, &s.meal))
                .collect::<BTreeSet<_>>();
            let confirmed = input
                .confirmed_slots
                .iter()
                .map(|s| (&s.date, &s.meal))
                .collect::<BTreeSet<_>>();
            if !input.reviewed
                || expected.is_empty()
                || expected != confirmed
                || confirmed.len() != input.confirmed_slots.len()
            {
                return Err("week_confirmation_required");
            }
            if let Some(revision) = proposal.applied_revision {
                return Ok(json!({"revision":revision,"replayed":true}));
            }
            if now()? >= proposal.expires_at {
                return Err("week_expired");
            }
            if proposal.plan_revision != plans.data.revision
                || proposal.profile_revision != profiles.revision
                || proposal.ingredient_revision != ingredients.revision()
                || proposal.allergen_version != household::allergen_version(state, ingredients)
            {
                return Err("revision_conflict");
            }
            for r in &proposal.candidates {
                if recipes
                    .get(&r.id, None)
                    .map_err(|_| "recipe_revision_not_found")?
                    .revision
                    != r.revision
                {
                    return Err("revision_conflict");
                }
            }
            let mut next = plans.data.clone();
            for s in &proposal.slots {
                if let Some(plan) = &s.plan {
                    if preview(state, profiles, recipes, &next, plan, ingredients)?["feasible"]
                        != true
                    {
                        return Err("meal_constraints_conflict");
                    }
                    let mut plan = plan.clone();
                    plan.reviewed_profile_revision = Some(profiles.revision);
                    plan.reviewed_allergen_version = Some(proposal.allergen_version.clone());
                    next.meals.push(plan);
                }
            }
            validate_meals(&next.meals)?;
            if next.meals.len() > 100 {
                return Err("mealplan_capacity");
            }
            next.revision = next.revision.checked_add(1).ok_or("mealplan_capacity")?;
            let revision = next.revision;
            next.proposals
                .iter_mut()
                .find(|p| p.id == proposal.id)
                .ok_or("week_expired")?
                .applied_revision = Some(revision);
            plans.save(next)?;
            Ok(json!({"revision":revision,"replayed":false}))
        },
    )
    .await
}
