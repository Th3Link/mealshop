//! Validates versioned ingredient records with explicit provenance.

use super::nutrients::Nutrient;
use crate::quantities::Amount;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Source {
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) record_id: String,
    pub(crate) version: String,
    pub(crate) retrieved_on: String,
    pub(crate) license: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Education {
    pub(crate) component: String,
    pub(crate) title: String,
    pub(crate) summary: String,
    pub(crate) source_url: String,
    pub(crate) evidence_reference: String,
    pub(crate) conditions: String,
    pub(crate) limitations: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Reading {
    pub(crate) amount: Option<Amount>,
    pub(crate) uncertainty: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Basis {
    #[serde(rename = "per_100g_edible_portion")]
    Per100gEdiblePortion,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Ingredient {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) aliases: Vec<String>,
    pub(crate) form: String,
    pub(crate) basis: Basis,
    pub(crate) source: Source,
    pub(crate) uncertainty: String,
    #[serde(deserialize_with = "unique_readings")]
    pub(crate) nutrients: BTreeMap<Nutrient, Reading>,
    pub(crate) education: Vec<Education>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Catalogue {
    pub(crate) schema_version: u32,
    pub(crate) version: String,
    pub(crate) ingredients: Vec<Ingredient>,
}

fn text(value: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.len() > 4000 || value.chars().any(char::is_control) {
        Err("Catalogue text must be nonempty, printable and at most 4000 bytes".to_owned())
    } else {
        Ok(())
    }
}

fn https(value: &str) -> Result<(), String> {
    text(value)?;
    if !value.starts_with("https://") || value.len() <= 8 || value.chars().any(char::is_whitespace)
    {
        return Err("Catalogue source links must use HTTPS".to_owned());
    }
    Ok(())
}

impl Catalogue {
    pub(crate) fn from_json(json: &str) -> Result<Self, String> {
        if json.len() > 1_048_576 {
            return Err("Catalogue exceeds the 1 MiB input limit".to_owned());
        }
        let result: Self =
            serde_json::from_str(json).map_err(|e| format!("Invalid catalogue JSON: {e}"))?;
        if result.schema_version != 1 || result.ingredients.len() > 1000 {
            return Err("Expected catalogue schema 1 with at most 1000 records".to_owned());
        }
        text(&result.version)?;
        let mut ids = BTreeSet::new();
        for ingredient in &result.ingredients {
            ingredient.validate()?;
            if !ids.insert(&ingredient.id) {
                return Err(format!("Duplicate ingredient ID: {}", ingredient.id));
            }
        }
        Ok(result)
    }

    pub(crate) fn find(&self, id: &str) -> Option<&Ingredient> {
        self.ingredients.iter().find(|i| i.id == id)
    }
}

impl Ingredient {
    fn validate(&self) -> Result<(), String> {
        if self.id.is_empty()
            || self.id.len() > 100
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(
                "Ingredient IDs must use 1–100 ASCII letters, digits, hyphens or underscores"
                    .to_owned(),
            );
        }
        for field in [
            &self.name,
            &self.form,
            &self.uncertainty,
            &self.source.name,
            &self.source.record_id,
            &self.source.version,
            &self.source.retrieved_on,
            &self.source.license,
        ] {
            text(field)?;
        }
        https(&self.source.url)?;
        for alias in &self.aliases {
            text(alias)?;
        }
        for reading in self.nutrients.values() {
            text(&reading.uncertainty)?;
        }
        for note in &self.education {
            for field in [
                &note.component,
                &note.title,
                &note.summary,
                &note.evidence_reference,
                &note.conditions,
                &note.limitations,
            ] {
                text(field)?;
            }
            https(&note.source_url)?;
        }
        Ok(())
    }
}

fn unique_readings<'de, D>(deserializer: D) -> Result<BTreeMap<Nutrient, Reading>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Readings;
    impl<'de> serde::de::Visitor<'de> for Readings {
        type Value = BTreeMap<Nutrient, Reading>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a nutrient map with unique keys")
        }
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut readings = BTreeMap::new();
            while let Some((nutrient, reading)) = map.next_entry::<Nutrient, Reading>()? {
                if readings.insert(nutrient, reading).is_some() {
                    return Err(serde::de::Error::custom(
                        "Duplicate nutrient key in ingredient",
                    ));
                }
            }
            Ok(readings)
        }
    }
    deserializer.deserialize_map(Readings)
}
