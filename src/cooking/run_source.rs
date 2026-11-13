//! Food wire records stay compatible; beer sources cannot masquerade as food servings.
use super::{
    model::{Kind, PlanSlot, RecipeRef, Run, RunState, Step, Subscription, Timing},
    time::LocalTime,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrewRef {
    #[serde(
        default,
        skip_serializing_if = "crate::brewing::model::Beverage::is_beer"
    )]
    pub(crate) beverage: crate::brewing::model::Beverage,
    pub(crate) id: String,
    pub(crate) revision: u64,
    pub(crate) title: String,
    pub(crate) language: String,
    pub(crate) volume: crate::shopping::model::Quantity,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Source {
    Food { recipe: RecipeRef, servings: u32 },
    Brewing { recipe: BrewRef },
}
impl Source {
    pub(crate) fn validate(&self, kind: Kind) -> Result<(), &'static str> {
        let valid = match self {
            Self::Food { recipe, servings } => {
                !kind.is_brewing()
                    && crate::recipes::valid_id(&recipe.id)
                    && recipe.revision > 0
                    && super::model::text(&recipe.title, 200)
                    && matches!(recipe.language.as_str(), "en" | "de")
                    && (1..=1_000_000).contains(servings)
                    && (1..=1_000_000).contains(&recipe.reference_servings)
            }
            Self::Brewing { recipe } => {
                kind == recipe.beverage.kind()
                    && super::model::id(&recipe.id)
                    && recipe.id.starts_with("brew-recipe-")
                    && recipe.revision > 0
                    && super::model::text(&recipe.title, 200)
                    && matches!(recipe.language.as_str(), "en" | "de")
                    && matches!(
                        recipe.volume.unit,
                        crate::shopping::model::Unit::Ml | crate::shopping::model::Unit::L
                    )
                    && recipe.volume.normalized().is_ok()
            }
        };
        if valid {
            Ok(())
        } else {
            Err("cooking_data_invalid")
        }
    }
}
impl Run {
    pub(crate) fn food(&self) -> Result<(&RecipeRef, u32), &'static str> {
        match &self.source {
            Source::Food { recipe, servings } => Ok((recipe, *servings)),
            Source::Brewing { .. } => Err("cooking_wrong_process"),
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Wire {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    plan_slot: Option<PlanSlot>,
    id: String,
    name: String,
    kind: Kind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    recipe: Option<RecipeRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    servings: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    brew_recipe: Option<BrewRef>,
    timezone: String,
    target: Option<LocalTime>,
    created_at: i64,
    status: RunState,
    steps: Vec<Step>,
    schedule: Vec<Timing>,
    subscriptions: BTreeMap<String, Subscription>,
}
impl TryFrom<Wire> for Run {
    type Error = &'static str;
    fn try_from(w: Wire) -> Result<Self, Self::Error> {
        let source = match (w.recipe, w.servings, w.brew_recipe) {
            (Some(recipe), Some(servings), None) => Source::Food { recipe, servings },
            (None, None, Some(recipe)) if w.plan_slot.is_none() => Source::Brewing { recipe },
            _ => return Err("cooking_data_invalid"),
        };
        source.validate(w.kind)?;
        Ok(Self {
            plan_slot: w.plan_slot,
            id: w.id,
            name: w.name,
            kind: w.kind,
            source,
            timezone: w.timezone,
            target: w.target,
            created_at: w.created_at,
            status: w.status,
            steps: w.steps,
            schedule: w.schedule,
            subscriptions: w.subscriptions,
        })
    }
}
impl From<Run> for Wire {
    fn from(r: Run) -> Self {
        let (recipe, servings, brew_recipe) = match r.source {
            Source::Food { recipe, servings } => (Some(recipe), Some(servings), None),
            Source::Brewing { recipe } => (None, None, Some(recipe)),
        };
        Self {
            plan_slot: r.plan_slot,
            id: r.id,
            name: r.name,
            kind: r.kind,
            recipe,
            servings,
            brew_recipe,
            timezone: r.timezone,
            target: r.target,
            created_at: r.created_at,
            status: r.status,
            steps: r.steps,
            schedule: r.schedule,
            subscriptions: r.subscriptions,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    fn old_food() -> Value {
        json!({"id":"cook-1","name":"Synthetic food","kind":"cooking","recipe":{"id":"recipe-00000001","revision":1,"title":"Synthetic food","language":"en","reference_servings":2},"servings":3,"timezone":"UTC","target":null,"created_at":1_794_524_400_i64,"status":"active","steps":[],"schedule":[],"subscriptions":{}})
    }
    #[test]
    fn existing_food_wire_is_unchanged_and_cannot_gain_a_false_brew_source() {
        let original = old_food();
        let parsed: Run = serde_json::from_value(original.clone()).unwrap();
        assert_eq!(parsed.food().unwrap().1, 3);
        assert_eq!(serde_json::to_value(parsed).unwrap(), original);
        let mut mixed = original;
        mixed["brew_recipe"] = json!({"id":"brew-recipe-1","revision":1,"title":"Synthetic brew","language":"en","volume":{"amount":"0.100001","unit":"l"}});
        assert!(serde_json::from_value::<Run>(mixed).is_err());
    }
    #[test]
    fn beer_preserves_exact_liquid_volume_and_has_no_food_recipe_or_servings() {
        let mut original = old_food();
        let object = original.as_object_mut().unwrap();
        object.remove("recipe");
        object.remove("servings");
        original["kind"] = json!("beer");
        original["brew_recipe"] = json!({"id":"brew-recipe-1","revision":1,"title":"Synthetic brew","language":"en","volume":{"amount":"0.100001","unit":"l"}});
        let parsed: Run = serde_json::from_value(original.clone()).unwrap();
        assert_eq!(parsed.food().unwrap_err(), "cooking_wrong_process");
        assert_eq!(serde_json::to_value(parsed).unwrap(), original);
        original["brew_recipe"]["volume"]["unit"] = json!("g");
        assert!(serde_json::from_value::<Run>(original).is_err());
    }
    #[test]
    fn fermentation_kinds_never_accept_food_or_mismatched_drink_references() {
        for kind in ["cider", "wine", "mead"] {
            let mut food = old_food();
            food["kind"] = json!(kind);
            assert!(serde_json::from_value::<Run>(food.clone()).is_err());
            food.as_object_mut().unwrap().remove("recipe");
            food.as_object_mut().unwrap().remove("servings");
            food["brew_recipe"] = json!({"beverage":kind,"id":"brew-recipe-1","revision":1,"title":"Synthetic drink","language":"en","volume":{"amount":"20.000001","unit":"l"}});
            let run: Run = serde_json::from_value(food.clone()).unwrap();
            assert!(run.food().is_err());
            assert_eq!(serde_json::to_value(run).unwrap(), food);
            food["brew_recipe"]["beverage"] = json!("beer");
            assert!(serde_json::from_value::<Run>(food).is_err());
        }
    }
}
