//! Validates versioned ingredient records with explicit provenance.

use super::nutrients::Nutrient;
use crate::quantities::Amount;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SourceKind {
    #[default]
    Primary,
    HouseholdDeclaration,
}
impl SourceKind {
    fn is_primary(&self) -> bool {
        *self == Self::Primary
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Source {
    #[serde(default, skip_serializing_if = "SourceKind::is_primary")]
    pub(crate) kind: SourceKind,
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
pub(crate) struct IngredientText {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) aliases: Vec<String>,
    pub(crate) form: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Ingredient {
    pub(crate) id: String,
    #[serde(default = "other_category")]
    pub(crate) category: String,
    pub(crate) name: String,
    pub(crate) aliases: Vec<String>,
    pub(crate) form: String,
    pub(crate) basis: Basis,
    pub(crate) source: Source,
    pub(crate) uncertainty: String,
    #[serde(deserialize_with = "unique_readings")]
    pub(crate) nutrients: BTreeMap<Nutrient, Reading>,
    pub(crate) education: Vec<Education>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) translations: BTreeMap<String, IngredientText>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) components: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) declared_allergens: Vec<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) archived: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) favorite: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Catalogue {
    pub(crate) schema_version: u32,
    #[serde(default = "english")]
    pub(crate) language: String,
    pub(crate) version: String,
    pub(crate) ingredients: Vec<Ingredient>,
}

fn other_category() -> String {
    "other".to_owned()
}

fn english() -> String {
    "en".to_owned()
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
        if result.schema_version != 1 || result.language != "en" || result.ingredients.len() > 1000
        {
            return Err(
                "Expected English-base catalogue schema 1 with at most 1000 records".to_owned(),
            );
        }
        text(&result.version)?;
        let mut ids = BTreeSet::new();
        for ingredient in &result.ingredients {
            ingredient.validate()?;
            if !ids.insert(&ingredient.id) {
                return Err(format!("Duplicate ingredient ID: {}", ingredient.id));
            }
        }
        result.validate_composition()?;
        Ok(result)
    }
    pub(crate) fn closure(&self, ids: &BTreeSet<String>) -> BTreeSet<String> {
        let mut result = ids.clone();
        let mut pending: Vec<_> = ids.iter().cloned().collect();
        while let Some(id) = pending.pop() {
            if let Some(ingredient) = self.find(&id) {
                for component in &ingredient.components {
                    if result.insert(component.clone()) {
                        pending.push(component.clone());
                    }
                }
            }
        }
        result
    }
    fn validate_composition(&self) -> Result<(), String> {
        fn visit(
            catalogue: &Catalogue,
            id: &str,
            stack: &mut BTreeSet<String>,
            done: &mut BTreeSet<String>,
        ) -> Result<(), String> {
            if done.contains(id) {
                return Ok(());
            }
            if stack.len() >= 32 || !stack.insert(id.to_owned()) {
                return Err("Ingredient composition is cyclic or too deep".to_owned());
            }
            let ingredient = catalogue.find(id).ok_or("Unknown composition component")?;
            for component in &ingredient.components {
                visit(catalogue, component, stack, done)?;
            }
            stack.remove(id);
            done.insert(id.to_owned());
            Ok(())
        }
        let mut done = BTreeSet::new();
        for ingredient in &self.ingredients {
            visit(self, &ingredient.id, &mut BTreeSet::new(), &mut done)?;
        }
        Ok(())
    }

    pub(crate) fn find(&self, id: &str) -> Option<&Ingredient> {
        self.ingredients.iter().find(|i| i.id == id)
    }
}

impl Ingredient {
    pub(crate) fn matches(&self, query: &str) -> bool {
        let mut labels = vec![self.name.as_str()];
        labels.extend(self.aliases.iter().map(String::as_str));
        for translated in self.translations.values() {
            labels.push(&translated.name);
            labels.extend(translated.aliases.iter().map(String::as_str));
        }
        let searchable = labels.join(" ").to_lowercase();
        query
            .split_whitespace()
            .all(|term| searchable.contains(&term.to_lowercase()))
    }

    fn validate_household_fields(&self) -> Result<(), String> {
        if self.source.kind == SourceKind::HouseholdDeclaration {
            if !self.id.starts_with("custom-")
                || !self.nutrients.is_empty()
                || !self.education.is_empty()
            {
                return Err("Household declarations must retain explicit unknown nutrition and a custom identity".to_owned());
            }
            if !self.source.url.is_empty() {
                https(&self.source.url)?;
            }
        } else {
            https(&self.source.url)?;
        }
        if self.components.len() > 100
            || self.tags.len() > 30
            || self.declared_allergens.len() > 14
            || self.components.iter().collect::<BTreeSet<_>>().len() != self.components.len()
            || self.tags.iter().collect::<BTreeSet<_>>().len() != self.tags.len()
            || self
                .declared_allergens
                .iter()
                .any(|id| !crate::household::allergen_ids().contains(&id.as_str()))
        {
            return Err("Invalid ingredient components, tags or declared allergens".to_owned());
        }
        for tag in &self.tags {
            text(tag)?;
            if tag.len() > 80 || tag.trim() != tag {
                return Err("Ingredient tag exceeds 80 bytes".to_owned());
            }
        }
        Ok(())
    }
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
        if self.category.is_empty()
            || self.category.len() > 40
            || !self
                .category
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b == b'_')
        {
            return Err("Ingredient categories must be lowercase English identifiers".to_owned());
        }
        for (locale, translated) in &self.translations {
            if locale == "en"
                || locale.len() > 35
                || !locale
                    .split('-')
                    .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_alphanumeric()))
            {
                return Err("Translation language tags must be valid identifiers; English belongs in the base fields".to_owned());
            }
            text(&translated.name)?;
            for alias in &translated.aliases {
                text(alias)?;
            }
            if let Some(form) = &translated.form {
                text(form)?;
            }
        }
        self.validate_household_fields()?;
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
