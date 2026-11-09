//! Explicit, optional food constituents; never inferred health effects.
use crate::nutrition::{catalogue::Ingredient, nutrients::Nutrient};
use crate::quantities::{Amount, Grams, SCALE, rounded};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Declaration {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) translations: BTreeMap<String, String>,
    pub(crate) unit: Unit,
    pub(crate) amount: Option<Amount>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) enum Unit {
    #[serde(rename = "g")]
    Grams,
    #[serde(rename = "mg")]
    Milligrams,
    #[serde(rename = "µg")]
    Micrograms,
}
#[derive(Serialize)]
pub(crate) struct ResultRow {
    id: String,
    label: String,
    translations: BTreeMap<String, String>,
    unit: Unit,
    total: Option<String>,
    per_serving: Option<String>,
    known_total: String,
    known_per_serving: String,
    status: &'static str,
    missing_ingredients: Vec<String>,
}
pub(crate) fn validate(catalogue: &crate::nutrition::catalogue::Catalogue) -> Result<(), String> {
    let mut definitions = BTreeMap::new();
    for item in &catalogue.ingredients {
        if item.constituents.len() > 30 {
            return Err("At most 30 optional constituents per ingredient".to_owned());
        }
        for (id, declaration) in &item.constituents {
            if id.is_empty()
                || id.len() > 100
                || !id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                || declaration.name.trim().is_empty()
                || declaration.name.len() > 200
                || declaration.name.chars().any(char::is_control)
                || declaration.translations.len() > 10
                || declaration.translations.iter().any(|(locale, name)| {
                    locale.is_empty()
                        || locale.len() > 35
                        || !locale
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                        || name.trim().is_empty()
                        || name.len() > 200
                        || name.chars().any(char::is_control)
                })
                || (matches!(id.as_str(), "beta_glucan" | "soluble_fibre")
                    && declaration.unit != Unit::Grams)
            {
                return Err("Constituents need stable IDs, names and explicit units".to_owned());
            }
            if let Some(previous) = definitions.insert(id, (&declaration.name, &declaration.unit)) {
                if previous != (&declaration.name, &declaration.unit) {
                    return Err("A constituent ID must retain the same name and unit throughout a catalogue".to_owned());
                }
            }
        }
    }
    Ok(())
}
fn reading(item: &Ingredient, id: &str) -> Option<Declaration> {
    item.constituents.get(id).cloned().or_else(|| {
        let n = match id {
            "beta_glucan" => Nutrient::BetaGlucan,
            "soluble_fibre" => Nutrient::SolubleFibre,
            _ => return None,
        };
        item.nutrients.get(&n).map(|r| Declaration {
            name: n.info().label.to_owned(),
            translations: BTreeMap::new(),
            unit: Unit::Grams,
            amount: r.amount.clone(),
        })
    })
}
pub(crate) fn calculate(
    ingredients: &[(&Ingredient, &Grams)],
    servings: u32,
    numerator: u32,
    denominator: u32,
) -> Result<Vec<ResultRow>, String> {
    let mut ids: BTreeSet<String> = ingredients
        .iter()
        .flat_map(|(i, _)| i.constituents.keys().cloned())
        .collect();
    for id in ["beta_glucan", "soluble_fibre"] {
        if ingredients
            .iter()
            .any(|(i, _)| reading(i, id).is_some_and(|r| r.amount.is_some()))
        {
            ids.insert(id.to_owned());
        }
    }
    ids.into_iter()
        .map(|id| {
            let declaration = ingredients
                .iter()
                .find_map(|(i, _)| reading(i, &id))
                .ok_or("Missing constituent definition")?;
            let mut sum = 0_u128;
            let mut missing = BTreeSet::new();
            let mut known = 0;
            for (item, grams) in ingredients {
                if let Some(amount) = reading(item, &id).and_then(|r| r.amount) {
                    sum = amount
                        .atoms()
                        .checked_mul(grams.0.atoms())
                        .and_then(|n| sum.checked_add(n))
                        .ok_or("Constituent quantity exceeds precision")?;
                    known += 1;
                } else {
                    missing.insert(item.id.clone());
                }
            }
            let sum = sum
                .checked_mul(u128::from(numerator))
                .ok_or("Constituent quantity exceeds precision")?;
            let denominator = 100 * SCALE * SCALE * u128::from(denominator);
            let total = rounded(sum, denominator)?;
            let portion = rounded(sum, denominator * u128::from(servings))?;
            Ok(ResultRow {
                id,
                label: declaration.name,
                translations: declaration.translations,
                unit: declaration.unit,
                total: missing.is_empty().then(|| total.clone()),
                per_serving: missing.is_empty().then(|| portion.clone()),
                known_total: total,
                known_per_serving: portion,
                status: if missing.is_empty() {
                    "complete"
                } else if known == 0 {
                    "unknown"
                } else {
                    "partial"
                },
                missing_ingredients: missing.into_iter().collect(),
            })
        })
        .collect()
}

pub(crate) fn unique<'de, D>(deserializer: D) -> Result<BTreeMap<String, Declaration>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Entries;
    impl<'de> serde::de::Visitor<'de> for Entries {
        type Value = BTreeMap<String, Declaration>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("unique constituent identifiers")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut result = BTreeMap::new();
            while let Some((id, declaration)) = map.next_entry::<String, Declaration>()? {
                if result.insert(id, declaration).is_some() {
                    return Err(serde::de::Error::custom("Duplicate constituent identifier"));
                }
            }
            Ok(result)
        }
    }
    deserializer.deserialize_map(Entries)
}

pub(crate) fn optional_unique<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<BTreeMap<String, Declaration>>, D::Error> {
    unique(deserializer).map(Some)
}
