use crate::{
    ingredient_groups::{Diet, Production},
    nutrition::catalogue::{Catalogue, Ingredient},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Preset {
    #[default]
    Unrestricted,
    Vegetarian,
    Vegan,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Exception {
    ingredient_id: String,
    #[serde(default)]
    household_produced_only: bool,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Window {
    weekdays: Vec<u8>,
    meal: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Rules {
    version: u32,
    #[serde(default)]
    preset: Preset,
    #[serde(default)]
    exceptions: Vec<Exception>,
    #[serde(default)]
    max_active_seconds: Option<u32>,
    #[serde(default)]
    availability: Vec<Window>,
}
impl Rules {
    pub(crate) fn validate(&self, ids: &BTreeSet<String>) -> bool {
        let mut exceptions = BTreeSet::new();
        self.version == 1
            && self.exceptions.len() <= 100
            && self
                .exceptions
                .iter()
                .all(|e| ids.contains(&e.ingredient_id) && exceptions.insert(&e.ingredient_id))
            && self
                .max_active_seconds
                .is_none_or(|n| (1..=604_800).contains(&n))
            && self.availability.len() <= 28
            && self.availability.iter().all(|w| {
                !w.weekdays.is_empty()
                    && w.weekdays.len() <= 7
                    && w.weekdays.iter().all(|d| *d <= 6)
                    && w.weekdays.iter().collect::<BTreeSet<_>>().len() == w.weekdays.len()
                    && matches!(w.meal.as_str(), "breakfast" | "lunch" | "snack" | "dinner")
            })
    }
    fn classify(&self, item: &Ingredient) -> &'static str {
        if self.preset == Preset::Unrestricted {
            return "allowed_by_preset";
        }
        if let Some(exception) = self.exceptions.iter().find(|e| e.ingredient_id == item.id) {
            return if !exception.household_produced_only
                || item.organization.production == Some(Production::HouseholdProduced)
            {
                "explicit_exception"
            } else if matches!(
                item.organization.production,
                None | Some(Production::Unknown)
            ) {
                "origin_unknown"
            } else {
                "excluded_by_preset"
            };
        }
        match (&self.preset, &item.organization.diet) {
            (_, Diet::Vegan) | (Preset::Vegetarian, Diet::Vegetarian) => "allowed_by_preset",
            (_, Diet::Unknown) => "diet_unknown",
            _ => "excluded_by_preset",
        }
    }
}
pub(crate) fn diet_review(
    rules: &Rules,
    ids: &BTreeSet<String>,
    direct: &BTreeSet<String>,
    current: &Catalogue,
    snapshot: Option<&Catalogue>,
) -> Value {
    let mut excluded = BTreeSet::new();
    let mut unknown = BTreeSet::new();
    let mut exceptions = BTreeSet::new();
    for id in ids {
        for catalogue in std::iter::once(current).chain(snapshot) {
            if let Some(item) = catalogue.find(id) {
                if item.organization.is_group && !direct.contains(id) {
                    continue;
                }
                match rules.classify(item) {
                    "excluded_by_preset" => {
                        excluded.insert(id);
                    }
                    "diet_unknown" | "origin_unknown" => {
                        unknown.insert(id);
                    }
                    "explicit_exception" => {
                        exceptions.insert(id);
                    }
                    _ => {}
                }
            }
        }
    }
    json!({"preset":rules.preset,"has_exceptions":!rules.exceptions.is_empty(),"excluded_ingredients":excluded,"unknown_ingredients":unknown,"exceptions":exceptions})
}
pub(crate) fn planning_conflicts(
    profiles: &super::super::Profiles,
    people: &[String],
    date: &str,
    meal: &str,
    active: u64,
) -> Vec<&'static str> {
    let weekday = date
        .parse::<jiff::civil::Date>()
        .ok()
        .and_then(|d| u8::try_from(d.weekday().to_monday_zero_offset()).ok());
    let mut conflicts = BTreeSet::new();
    for member in profiles.members.iter().filter(|m| people.contains(&m.id)) {
        if let Some(rules) = &member.rules {
            if rules
                .max_active_seconds
                .is_some_and(|limit| active > u64::from(limit))
            {
                conflicts.insert("person_active_time_exceeded");
            }
            if !rules.availability.is_empty()
                && !rules
                    .availability
                    .iter()
                    .any(|w| w.meal == meal && weekday.is_some_and(|d| w.weekdays.contains(&d)))
            {
                conflicts.insert("person_unavailable");
            }
        }
    }
    conflicts.into_iter().collect()
}
