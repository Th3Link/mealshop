//! Explicit reported supplements and source-backed comparison ranges.
use super::{
    catalogue::{Catalogue, Ingredient, Source, SourceKind},
    nutrients::Nutrient,
};
use crate::quantities::Amount;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Bounds {
    pub(crate) lower: Amount,
    pub(crate) upper: Amount,
}
impl Bounds {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.lower.atoms() > self.upper.atoms() {
            return Err("Nutrition range lower bound exceeds its upper bound".to_owned());
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Method {
    SourceSupplement,
    ComparisonEnvelope,
    GroupEnvelope,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Reference {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) form: String,
    pub(crate) source: Source,
    pub(crate) values: BTreeMap<Nutrient, Bounds>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Profile {
    pub(crate) version: String,
    pub(crate) method: Method,
    pub(crate) rationale: String,
    pub(crate) references: Vec<Reference>,
    pub(crate) values: BTreeMap<Nutrient, Bounds>,
}
fn text(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 4000 && !value.chars().any(char::is_control)
}
impl Profile {
    pub(crate) fn validate(&self, ingredient: &Ingredient) -> Result<(), String> {
        if !text(&self.version)
            || !text(&self.rationale)
            || self.references.is_empty()
            || self.references.len() > 64
            || self.values.is_empty()
            || (self.method == Method::ComparisonEnvelope && self.references.len() < 2)
            || (self.method == Method::SourceSupplement && self.references.len() != 1)
        {
            return Err(
                "Nutrition evidence requires a version, rationale and bounded source records"
                    .to_owned(),
            );
        }
        let mut ids = std::collections::BTreeSet::new();
        for r in &self.references {
            for (id, bounds) in &r.values {
                bounds.validate()?;
                if id.info().group == "component" {
                    return Err("Constituents require separate explicit evidence".to_owned());
                }
            }
            if !ids.insert(&r.id)
                || !text(&r.id)
                || !text(&r.name)
                || !text(&r.form)
                || r.source.kind != SourceKind::Primary
                || !r.source.url.starts_with("https://")
                || r.source.url.len() <= "https://".len()
                || r.source.url.chars().any(char::is_whitespace)
                || [
                    &r.source.name,
                    &r.source.record_id,
                    &r.source.version,
                    &r.source.retrieved_on,
                    &r.source.license,
                ]
                .iter()
                .any(|s| !text(s))
            {
                return Err(
                    "Nutrition comparison sources must retain identity, state and HTTPS provenance"
                        .to_owned(),
                );
            }
        }
        if self.method == Method::SourceSupplement {
            let source = &self.references[0].source;
            if source.record_id != ingredient.source.record_id
                || source.version != ingredient.source.version
                || source.name != ingredient.source.name
            {
                return Err(
                    "Reported supplements must match the original source identity and version"
                        .to_owned(),
                );
            }
        }
        for (id, bounds) in &self.values {
            bounds.validate()?;
            if id.info().group == "component" {
                return Err("Constituents require separate explicit evidence".to_owned());
            }
            let values: Vec<_> = self
                .references
                .iter()
                .filter_map(|r| r.values.get(id))
                .collect();
            if values.iter().map(|v| v.lower.atoms()).min() != Some(bounds.lower.atoms())
                || values.iter().map(|v| v.upper.atoms()).max() != Some(bounds.upper.atoms())
            {
                return Err(
                    "Comparison bounds must equal the observed reference envelope".to_owned(),
                );
            }
        }
        Ok(())
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    source_name: String,
    source_record_id: String,
    source_version: String,
    form: String,
    profiles: Vec<Profile>,
    #[serde(default)]
    members: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    version: String,
    profiles: BTreeMap<String, Entry>,
}

pub(crate) fn attach(catalogue: &mut Catalogue) -> Result<(), String> {
    static BASELINE: std::sync::OnceLock<Result<Catalogue, String>> = std::sync::OnceLock::new();
    static REGISTRY: std::sync::OnceLock<Result<Registry, String>> = std::sync::OnceLock::new();
    let registry = REGISTRY
        .get_or_init(|| {
            serde_json::from_str(include_str!("../../data/nutrition-estimates-v1.json"))
                .map_err(|e| format!("Invalid nutrition estimate registry: {e}"))
        })
        .as_ref()
        .map_err(Clone::clone)?;
    let baseline = BASELINE
        .get_or_init(|| {
            serde_json::from_str(include_str!("../../data/catalogue.json"))
                .map_err(|e| format!("Invalid original nutrition baseline: {e}"))
        })
        .as_ref()
        .map_err(Clone::clone)?;
    let incompatible_groups: std::collections::BTreeSet<_> = registry
        .profiles
        .iter()
        .filter(|(_, entry)| {
            entry.members.iter().any(|id| {
                catalogue.find(id).is_some_and(|item| {
                    baseline.find(id).is_none_or(|original| {
                        item.source.record_id != original.source.record_id
                            || item.source.version != original.source.version
                            || item.source.name != original.source.name
                            || item.form != original.form
                            || item.nutrients != original.nutrients
                    })
                })
            })
        })
        .map(|(id, _)| id)
        .collect();
    for item in &mut catalogue.ingredients {
        if incompatible_groups.contains(&item.id) {
            continue;
        }
        if item.nutrition_estimates.is_empty() {
            if let Some(entry) = registry.profiles.get(&item.id).filter(|e| {
                e.source_name == item.source.name
                    && e.source_record_id == item.source.record_id
                    && e.source_version == item.source.version
                    && e.form == item.form
            }) {
                for profile in &entry.profiles {
                    if profile.version != registry.version {
                        return Err("Nutrition estimate version mismatch".to_owned());
                    }
                    profile.validate(item)?;
                }
                item.nutrition_estimates.clone_from(&entry.profiles);
            }
        }
    }
    Ok(())
}

pub(crate) enum Value<'a> {
    Reported(&'a Amount),
    Estimated(&'a Bounds),
    Unknown,
}
pub(crate) fn value(ingredient: &Ingredient, id: Nutrient) -> Value<'_> {
    if let Some(reading) = ingredient.nutrients.get(&id) {
        if let Some(amount) = &reading.amount {
            return Value::Reported(amount);
        }
        if let Some(range) = &reading.range {
            return Value::Estimated(range);
        }
    }
    for profile in &ingredient.nutrition_estimates {
        if let Some(range) = profile.values.get(&id) {
            return if profile.method == Method::SourceSupplement
                && range.lower.atoms() == range.upper.atoms()
            {
                Value::Reported(&range.lower)
            } else {
                Value::Estimated(range)
            };
        }
    }
    Value::Unknown
}

pub(crate) fn is_estimated(ingredient: &Ingredient) -> bool {
    Nutrient::ALL
        .iter()
        .any(|id| matches!(value(ingredient, *id), Value::Estimated(_)))
}
