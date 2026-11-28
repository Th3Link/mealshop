use super::model::{Data, Item, Origin, Requirement, Source, Unit, amount};
use crate::recipes::Store;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Selection {
    pub(crate) recipe_id: String,
    pub(crate) revision: u64,
    pub(crate) servings: u32,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MealSlot {
    pub(crate) date: String,
    pub(crate) meal: String,
}
pub(crate) struct MealSelection {
    pub(crate) date: String,
    pub(crate) meal: String,
    pub(crate) recipe: Selection,
}
pub(crate) struct Draft {
    pub(crate) origin: Origin,
    pub(crate) requirements: Vec<Requirement>,
}
pub(crate) fn recipe(store: &Store, selection: &Selection) -> Result<Draft, &'static str> {
    let record = store
        .get(&selection.recipe_id, Some(selection.revision))
        .map_err(|_| "recipe_revision_not_found")?;
    let mut amounts: BTreeMap<(String, &'static str), (Unit, u128)> = BTreeMap::new();
    for line in record.recipe.input_lines() {
        let line = line
            .scaled(selection.servings, record.recipe.servings)
            .map_err(|_| "shopping_quantity_invalid")?;
        // A line without a known mass is bought in its own unit. Spoons are listed as
        // millilitres of the metric 5/15 ml spoon: a volume, never a derived mass.
        let (unit, atoms) = match (&line.grams, &line.quantity) {
            (Some(grams), _) => (Unit::G, grams.0.atoms()),
            (None, Some(quantity)) => {
                use crate::quantities::Unit as Input;
                let (unit, factor) = match quantity.unit {
                    Input::Ml => (Unit::Ml, 1),
                    Input::L => (Unit::Ml, 1000),
                    Input::Tsp => (Unit::Ml, 5),
                    Input::Tbsp => (Unit::Ml, 15),
                    Input::Piece => (Unit::Piece, 1),
                    Input::G | Input::Kg => return Err("shopping_quantity_invalid"),
                };
                (unit, quantity.amount.atoms() * factor)
            }
            (None, None) => return Err("shopping_quantity_invalid"),
        };
        amounts
            .entry((line.ingredient_id.clone(), unit.key()))
            .or_insert((unit, 0))
            .1 += atoms;
    }
    let requirements = amounts
        .into_iter()
        .map(|((id, key), (unit, atoms))| {
            let ingredient = record
                .catalogue
                .ingredients
                .iter()
                .find(|i| i.id == id)
                .ok_or("unknown_ingredient")?;
            Ok(Requirement {
                item: Item {
                    key: format!("ingredient:{id}:{key}"),
                    ingredient_id: Some(id),
                    name: ingredient.name.clone(),
                    translations: ingredient
                        .translations
                        .iter()
                        .map(|(lang, tr)| (lang.clone(), tr.name.clone()))
                        .collect(),
                    unit,
                },
                amount: amount(atoms)?,
            })
        })
        .collect::<Result<_, &'static str>>()?;
    Ok(Draft {
        origin: Origin::Recipe {
            recipe_id: record.id,
            revision: record.revision,
            title: record.recipe.title,
            language: record.recipe.language,
            servings: selection.servings,
        },
        requirements,
    })
}
pub(crate) fn upsert(
    data: &mut Data,
    id: Option<String>,
    draft: Draft,
) -> Result<(), &'static str> {
    if let Some(id) = id {
        let changed = data.sources.iter().find(|s| s.id == id).is_some_and(|s| {
            serde_json::to_value((&s.origin, &s.requirements)).ok()
                != serde_json::to_value((&draft.origin, &draft.requirements)).ok()
        });
        if changed {
            crate::inventory::allocations::release_source(data, &id);
        }
        let source = data
            .sources
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or("shopping_source_not_found")?;
        if std::mem::discriminant(&source.origin) != std::mem::discriminant(&draft.origin) {
            return Err("shopping_source_invalid");
        }
        source.origin = draft.origin;
        source.requirements = draft.requirements;
        source.active = true;
    } else {
        let id = data.allocate("source")?;
        data.sources.push(Source {
            id,
            active: true,
            origin: draft.origin,
            requirements: draft.requirements,
        });
    }
    Ok(())
}
pub(crate) fn sync(
    data: &mut Data,
    store: &Store,
    selections: Vec<MealSelection>,
) -> Result<(), &'static str> {
    for source in &mut data.sources {
        if matches!(source.origin, Origin::Meal { .. }) {
            source.active = false;
        }
    }
    for selected in selections {
        let mut draft = recipe(store, &selected.recipe)?;
        if let Origin::Recipe {
            recipe_id,
            revision,
            title,
            language,
            servings,
        } = draft.origin
        {
            draft.origin = Origin::Meal {
                date: selected.date.clone(),
                meal: selected.meal.clone(),
                recipe_id,
                revision,
                title,
                language,
                servings,
            };
        }
        let id = data.sources.iter().find_map(|s| match &s.origin {
            Origin::Meal { date, meal, .. } if *date == selected.date && *meal == selected.meal => {
                Some(s.id.clone())
            }
            _ => None,
        });
        upsert(data, id, draft)?;
    }
    Ok(())
}
