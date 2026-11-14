use super::{
    Plan, Plans, attendees, decimal, load_recipe, masses, planning_conflicts, signed_atoms,
    validate, validate_meals,
};
use crate::{
    household::{self, Evaluation, Household, Profiles},
    ingredients,
    recipes::{Record, Store},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(super) struct Assessment {
    people: Vec<String>,
    review: Value,
    active: u64,
    elapsed: u64,
    reference_servings: u32,
    has_steps: bool,
}
impl Assessment {
    pub(super) fn new(
        state: &Household,
        profiles: &Profiles,
        record: &Record,
        p: &Plan,
        ingredients: &ingredients::Store,
    ) -> Result<Self, &'static str> {
        validate(p)?;
        let people = attendees(p);
        let review = if people.is_empty() {
            json!({"status":"review_required","excluded_ingredients":[],"allergen_matches":[],"review_notes":[],"disliked_ingredients":[],"unknown_ingredients":[],"warning_codes":["attendees_not_assigned"]})
        } else {
            household::review_snapshot(
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
                ingredients,
                Some(&record.catalogue),
            )?
        };
        let active = record
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
        Ok(Self {
            people,
            review,
            active,
            elapsed,
            reference_servings: record.recipe.servings,
            has_steps: !record.recipe.steps.is_empty(),
        })
    }
    pub(super) fn rank_value(
        &self,
        state: &Household,
        profiles: &Profiles,
        plans: &Plans,
        p: &Plan,
    ) -> Result<Value, &'static str> {
        validate(p)?;
        let mut proposed = plans
            .meals
            .iter()
            .filter(|old| old.date != p.date || old.meal != p.meal)
            .cloned()
            .collect::<Vec<_>>();
        proposed.push(p.clone());
        validate_meals(&proposed)?;
        let mut conflicts =
            planning_conflicts(state, profiles, plans, p, &self.people, self.active)?;
        if self.review["status"] == "blocked" {
            conflicts.push("dietary_exclusion");
        }
        if !self.has_steps {
            conflicts.push("preparation_time_unknown");
        }
        if self.active > p.active_seconds {
            conflicts.push("active_time_exceeded");
        }
        if self.elapsed > p.elapsed_seconds + p.advance_seconds {
            conflicts.push("elapsed_time_exceeded");
        }
        Ok(
            json!({"review":self.review,"conflicts":conflicts,"feasible":conflicts.is_empty(),
            "timing":{"reference_servings":self.reference_servings,"active_seconds":self.active,"elapsed_seconds":self.elapsed,
            "advance_preparation_seconds":self.elapsed.saturating_sub(p.elapsed_seconds)}}),
        )
    }
    pub(super) fn preview(
        &self,
        state: &Household,
        profiles: &Profiles,
        store: &Store,
        plans: &Plans,
        p: &Plan,
        record: &Record,
    ) -> Result<Value, &'static str> {
        let mut result = self.rank_value(state, profiles, plans, p)?;
        let mut delta = masses(record, p.servings)?
            .into_iter()
            .map(|(id, n)| signed_atoms(n).map(|n| (id, n)))
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        if let Some(old) = plans
            .meals
            .iter()
            .find(|old| old.date == p.date && old.meal == p.meal)
        {
            for (id, n) in masses(&load_recipe(store, old)?, old.servings)? {
                *delta.entry(id).or_insert(0) -=
                    i128::try_from(n).map_err(|_| "quantity_overflow")?;
            }
        }
        let nutrition = crate::nutrition::scaled_nutrition(
            &record.catalogue,
            &record.recipe.calculation(),
            p.servings,
        )
        .map_err(|_| "scaled_quantity_out_of_range")?;
        let fields = json!({"plan":p,"profile_revision":profiles.revision,"plan_revision":plans.revision,"attendees":self.people,
            "attendance_source":p.calendar.as_ref().map(|c|&c.source),"attendance_overridden":p.attendees.is_some()&&p.calendar.is_some(),
            "shopping_delta":delta.into_iter().filter(|(_,n)|*n!=0).map(|(id,n)|json!({"ingredient_id":id,"grams_delta":decimal(n)})).collect::<Vec<_>>(),
            "nutrition":nutrition});
        result
            .as_object_mut()
            .ok_or("mealplan_storage_failed")?
            .extend(fields.as_object().ok_or("mealplan_storage_failed")?.clone());
        Ok(result)
    }
}
