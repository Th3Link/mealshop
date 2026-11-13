//! Explicit group membership and provenance-aware household nutrition.
use crate::nutrition::{
    catalogue::{Catalogue, Ingredient, Reading, Source, SourceKind},
    nutrients::Nutrient,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Organization {
    #[serde(default)]
    pub(crate) is_group: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) intolerance_triggers: Option<Vec<String>>,
    #[serde(default)]
    pub(crate) group_id: Option<String>,
    #[serde(default)]
    pub(crate) nutrition: Nutrition,
    #[serde(default)]
    pub(crate) diet: Diet,
    #[serde(default)]
    pub(crate) origin: Origin,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) production: Option<Production>,
}
#[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Nutrition {
    #[default]
    Reference,
    Own {
        #[serde(deserialize_with = "crate::nutrition::catalogue::unique_readings")]
        values: BTreeMap<Nutrient, Reading>,
        #[serde(
            default,
            deserialize_with = "crate::constituents::optional_unique",
            skip_serializing_if = "Option::is_none"
        )]
        constituents: Option<BTreeMap<String, crate::constituents::Declaration>>,
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

#[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Production {
    #[default]
    Unknown,
    Purchased,
    HouseholdProduced,
}

fn inherited_evidence(parent: &Ingredient) -> Vec<crate::nutrition::estimates::Profile> {
    if !matches!(parent.organization.nutrition, Nutrition::Reference) {
        return Vec::new();
    }
    parent
        .nutrition_estimates
        .iter()
        .cloned()
        .map(|mut profile| {
            profile.method = crate::nutrition::estimates::Method::GroupEnvelope;
            profile.rationale = format!("Explicit group inheritance. {}", profile.rationale);
            profile
        })
        .collect()
}

pub(crate) fn resolve(catalogue: &mut Catalogue, revision: u64) -> Result<(), String> {
    let original = catalogue.clone();
    for item in &mut catalogue.ingredients {
        let org = &item.organization;
        if org.intolerance_triggers.as_ref().is_some_and(|triggers| {
            triggers.len() > 1 || triggers.iter().any(|id| id != "sorbitol")
        }) {
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
                constituents.clone().unwrap_or_default(),
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
        item.nutrition_estimates = parent
            .filter(|_| inherited.is_some())
            .map_or_else(Vec::new, inherited_evidence);
        item.constituents = constituents;
        for (id, nutrient) in [
            ("beta_glucan", Nutrient::BetaGlucan),
            ("soluble_fibre", Nutrient::SolubleFibre),
        ] {
            if let Some(declaration) = item.constituents.get(id) {
                item.nutrients.insert(
                    nutrient,
                    Reading {
                        range: None,
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GroupMapping {
    version: String,
    groups: Vec<Group>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Group {
    id: String,
    name: String,
    translations: BTreeMap<String, crate::nutrition::catalogue::IngredientText>,
    ingredient_ids: Vec<String>,
}
pub(crate) fn seed(catalogue: &mut Catalogue) -> Result<(), String> {
    let mapping: GroupMapping =
        serde_json::from_str(include_str!("../data/ingredient-groups.json"))
            .map_err(|_| "Invalid bundled ingredient grouping")?;
    let mut assigned = std::collections::BTreeSet::new();
    let mut groups = std::collections::BTreeSet::new();
    for definition in &mapping.groups {
        if !definition.id.starts_with("custom-group-")
            || !groups.insert(definition.id.clone())
            || definition.ingredient_ids.is_empty()
            || definition
                .ingredient_ids
                .iter()
                .any(|id| !assigned.insert(id))
        {
            return Err("Duplicate or invalid ingredient grouping".to_owned());
        }
    }
    for definition in mapping.groups {
        let ids: Vec<_> = definition
            .ingredient_ids
            .iter()
            .filter(|id| catalogue.find(id).is_some())
            .cloned()
            .collect();
        let Some(mut group) = ids.first().and_then(|id| catalogue.find(id)).cloned() else {
            continue;
        };
        if catalogue.find(&definition.id).is_some() {
            return Err("Ingredient group identity collides with a catalogue record".to_owned());
        }
        group.id = definition.id;
        group.name = definition.name;
        "Ingredient group".clone_into(&mut group.form);
        group.nutrients.clear();
        group.nutrition_estimates.clear();
        group.constituents.clear();
        group.education.clear();
        group.aliases.clear();
        group.components.clear();
        group.tags.clear();
        group.declared_allergens.clear();
        group.reference_source = None;
        group.nutrition_inherited_from = None;
        group.archived = false;
        group.favorite = false;
        group.organization = Organization {
            is_group: true,
            ..Organization::default()
        };
        group.translations = definition.translations;
        group.source = Source {
            kind: SourceKind::HouseholdDeclaration,
            name: "Catalogue organization".to_owned(),
            url: String::new(),
            record_id: group.id.clone(),
            version: mapping.version.clone(),
            retrieved_on: "Not applicable: organization only".to_owned(),
            license: "No nutrient values supplied".to_owned(),
        };
        "Organizational group; no inferred nutrient values".clone_into(&mut group.uncertainty);
        for item in &mut catalogue.ingredients {
            if ids.contains(&item.id) {
                item.organization.group_id = Some(group.id.clone());
            }
        }
        catalogue.ingredients.push(group);
    }
    crate::nutrition::estimates::attach(catalogue)?;
    let metadata = Catalogue {
        schema_version: catalogue.schema_version,
        language: catalogue.language.clone(),
        version: catalogue.version.clone(),
        ingredients: catalogue
            .ingredients
            .iter()
            .filter(|item| groups.contains(&item.id))
            .cloned()
            .collect(),
    };
    let json =
        serde_json::to_string(&metadata).map_err(|_| "Cannot validate ingredient grouping")?;
    Catalogue::from_json(&json)?;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn grouping_preserves_all_exact_records_and_separates_group_estimates() {
        let original = Catalogue::from_json(include_str!("../data/catalogue.json")).unwrap();
        let mut grouped = original.clone();
        seed(&mut grouped).unwrap();
        assert_eq!(grouped.ingredients.len(), original.ingredients.len() + 73);
        for item in &original.ingredients {
            let actual = grouped.find(&item.id).unwrap();
            let parent = grouped
                .find(actual.organization.group_id.as_deref().unwrap())
                .unwrap();
            assert!(parent.organization.is_group);
            let mut value = serde_json::to_value(actual).unwrap();
            value["organization"]["group_id"] = serde_json::Value::Null;
            assert_eq!(value, serde_json::to_value(item).unwrap());
        }
        for group in grouped
            .ingredients
            .iter()
            .filter(|i| i.organization.is_group)
        {
            assert!(!group.nutrition_estimates.is_empty());
            assert!(
                group.nutrients.is_empty()
                    && group.constituents.is_empty()
                    && group.education.is_empty()
            );
            assert!(
                group.declared_allergens.is_empty()
                    && group.components.is_empty()
                    && group.tags.is_empty()
            );
            assert_eq!(group.source.kind, SourceKind::HouseholdDeclaration);
        }
    }
}
