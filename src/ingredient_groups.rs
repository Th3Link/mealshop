//! Explicit group membership and provenance-aware household nutrition.
use crate::nutrition::{
    catalogue::{Catalogue, Reading, Source, SourceKind},
    nutrients::Nutrient,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Organization {
    #[serde(default)]
    pub(crate) is_group: bool,
    #[serde(default)]
    pub(crate) intolerance_triggers: Vec<String>,
    #[serde(default)]
    pub(crate) group_id: Option<String>,
    #[serde(default)]
    pub(crate) nutrition: Nutrition,
    #[serde(default)]
    pub(crate) diet: Diet,
    #[serde(default)]
    pub(crate) origin: Origin,
}
#[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Nutrition {
    #[default]
    Reference,
    Own {
        #[serde(deserialize_with = "crate::nutrition::catalogue::unique_readings")]
        values: BTreeMap<Nutrient, Reading>,
        #[serde(default, deserialize_with = "crate::constituents::unique")]
        constituents: BTreeMap<String, crate::constituents::Declaration>,
        note: String,
        source_url: String,
    },
    Inherit,
}
#[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Diet {
    #[default]
    Unknown,
    Vegan,
    Vegetarian,
    MeatFish,
}
#[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Origin {
    #[default]
    Unknown,
    Plant,
    Animal,
    Mixed,
}

pub(crate) fn resolve(catalogue: &mut Catalogue, revision: u64) -> Result<(), String> {
    let original = catalogue.clone();
    for item in &mut catalogue.ingredients {
        let org = &item.organization;
        if org.intolerance_triggers.len() > 1
            || org.intolerance_triggers.iter().any(|id| id != "sorbitol")
        {
            return Err("Unsupported intolerance trigger".to_owned());
        }
        if org.is_group && org.group_id.is_some() {
            return Err("A group cannot itself be a subtype".to_owned());
        }
        let parent = org
            .group_id
            .as_ref()
            .map(|id| original.find(id).ok_or("Unknown ingredient group"))
            .transpose()?;
        if parent.is_some_and(|p| {
            !p.organization.is_group || p.id == item.id || p.organization.group_id.is_some()
        }) {
            return Err("Invalid ingredient group".to_owned());
        }
        let (profile, inherited) = match &org.nutrition {
            Nutrition::Reference => continue,
            Nutrition::Own { .. } => (&org.nutrition, None),
            Nutrition::Inherit => {
                let p = parent.ok_or("Inherited nutrition needs a group")?;
                (&p.organization.nutrition, Some(p.id.clone()))
            }
        };
        let (values, constituents, note, url) = match profile {
            Nutrition::Own {
                values,
                note,
                source_url,
                constituents,
            } => (
                values.clone(),
                constituents.clone(),
                note.clone(),
                source_url.clone(),
            ),
            Nutrition::Reference => {
                let p = parent.ok_or("Missing group")?;
                (
                    p.nutrients.clone(),
                    p.constituents.clone(),
                    format!(
                        "Explicit household inheritance from {} ({})",
                        p.id, p.source.version
                    ),
                    p.source.url.clone(),
                )
            }
            Nutrition::Inherit => return Err("Nested group inheritance is unsupported".to_owned()),
        };
        if note.trim().is_empty()
            || note.len() > 2000
            || note.chars().any(char::is_control)
            || (!url.is_empty()
                && (!url.starts_with("https://") || url.chars().any(char::is_whitespace)))
        {
            return Err("Provide nutrient evidence and an optional HTTPS source".to_owned());
        }
        if item.reference_source.is_none() {
            item.reference_source = Some(item.source.clone());
        }
        item.source = Source {
            kind: SourceKind::HouseholdDeclaration,
            name: "Household nutrient declaration".to_owned(),
            url,
            record_id: inherited.clone().unwrap_or_else(|| item.id.clone()),
            version: revision.to_string(),
            retrieved_on: "Not recorded for this household declaration".to_owned(),
            license: "User declaration; reuse rights unspecified".to_owned(),
        };
        item.education.clear();
        item.nutrients = values;
        item.constituents = constituents;
        for (id, nutrient) in [
            ("beta_glucan", Nutrient::BetaGlucan),
            ("soluble_fibre", Nutrient::SolubleFibre),
        ] {
            if let Some(declaration) = item.constituents.get(id) {
                item.nutrients.insert(
                    nutrient,
                    Reading {
                        amount: declaration.amount.clone(),
                        uncertainty: item.uncertainty.clone(),
                    },
                );
            }
        }
        item.uncertainty = note;
        item.nutrition_inherited_from = inherited;
    }
    Ok(())
}

pub(crate) fn seed(catalogue: &mut Catalogue) {
    for (prefix, name, german) in [
        ("wheat", "Wheat flour", "Weizenmehl"),
        ("rye", "Rye flour", "Roggenmehl"),
        ("spelt", "Spelt flour", "Dinkelmehl"),
    ] {
        let ids: Vec<_> = catalogue
            .ingredients
            .iter()
            .filter(|i| {
                i.id.starts_with(&format!("{prefix}-flour-type-"))
                    || i.id == format!("{prefix}-wholemeal-flour")
            })
            .map(|i| i.id.clone())
            .collect();
        let Some(mut group) = ids.first().and_then(|id| catalogue.find(id)).cloned() else {
            continue;
        };
        let id = format!("custom-group-{prefix}-flour");
        group.id.clone_from(&id);
        name.clone_into(&mut group.name);
        "Ingredient group".clone_into(&mut group.form);
        group.nutrients.clear();
        group.constituents.clear();
        group.education.clear();
        group.aliases.clear();
        group.components.clear();
        group.tags.clear();
        group.declared_allergens.clear();
        group.organization = Organization {
            is_group: true,
            ..Organization::default()
        };
        group.translations = BTreeMap::from([(
            "de".to_owned(),
            crate::nutrition::catalogue::IngredientText {
                name: german.to_owned(),
                aliases: vec![],
                form: Some("Zutatengruppe".to_owned()),
            },
        )]);
        group.source = Source {
            kind: SourceKind::HouseholdDeclaration,
            name: "Catalogue organization".to_owned(),
            url: String::new(),
            record_id: id.clone(),
            version: "1".to_owned(),
            retrieved_on: "Not applicable: organization only".to_owned(),
            license: "No nutrient values supplied".to_owned(),
        };
        "Organizational group; no inferred nutrient values".clone_into(&mut group.uncertainty);
        for item in &mut catalogue.ingredients {
            if ids.contains(&item.id) {
                item.organization.group_id = Some(id.clone());
            }
        }
        catalogue.ingredients.push(group);
    }
}
