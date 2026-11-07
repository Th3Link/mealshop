//! Computes nutrient estimates from validated ingredient records.

pub(crate) mod catalogue;
pub(crate) mod nutrients;

use crate::quantities::{Grams, SCALE, rounded};
use catalogue::{Basis, Catalogue, Education, IngredientText, Source};
use nutrients::{Nutrient, NutrientInfo};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Serialize)]
pub(crate) struct RecipeLine {
    pub(crate) ingredient_id: String,
    pub(crate) grams: Grams,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) quantity: Option<crate::quantities::Quantity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) conversion: Option<crate::quantities::ConversionEvidence>,
}
impl<'de> Deserialize<'de> for RecipeLine {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Input {
            ingredient_id: String,
            grams: Option<Grams>,
            quantity: Option<crate::quantities::Quantity>,
            conversion: Option<crate::quantities::ConversionEvidence>,
        }
        let input = Input::deserialize(deserializer)?;
        let grams = if let Some(q) = &input.quantity {
            let result = q
                .grams(&input.ingredient_id, input.conversion.as_ref())
                .map_err(serde::de::Error::custom)?;
            if input
                .grams
                .as_ref()
                .is_some_and(|g| g.0.atoms() != result.0.atoms())
            {
                return Err(serde::de::Error::custom("quantity_grams_mismatch"));
            }
            result
        } else {
            if input.conversion.is_some() {
                return Err(serde::de::Error::custom("conversion_without_quantity"));
            }
            input
                .grams
                .ok_or_else(|| serde::de::Error::custom("quantity_required"))?
        };
        Ok(Self {
            ingredient_id: input.ingredient_id,
            grams,
            quantity: input.quantity,
            conversion: input.conversion,
        })
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Preparation {
    AsRecorded,
    CookingUnmodelled,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Calculation {
    pub(crate) lines: Vec<RecipeLine>,
    pub(crate) servings: u32,
    pub(crate) preparation: Preparation,
}

#[derive(Serialize)]
pub(crate) struct Contribution {
    ingredient_id: String,
    name: String,
    form: String,
    basis: Basis,
    grams: Grams,
    source: Source,
    uncertainty: String,
    education: Vec<Education>,
    translations: BTreeMap<String, IngredientText>,
}

#[derive(Serialize)]
pub(crate) struct NutrientResult {
    #[serde(flatten)]
    info: NutrientInfo,
    total: Option<String>,
    per_serving: Option<String>,
    known_total: String,
    known_per_serving: String,
    status: &'static str,
    missing_ingredients: Vec<String>,
    uncertainty: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct Estimate {
    catalogue_version: String,
    servings: u32,
    preparation: Preparation,
    rounding: &'static str,
    nutrients: Vec<NutrientResult>,
    ingredients: Vec<Contribution>,
    warnings: Vec<&'static str>,
    warning_codes: Vec<&'static str>,
}

pub(crate) fn calculate(catalogue: &Catalogue, recipe: &Calculation) -> Result<Estimate, String> {
    if recipe.lines.is_empty()
        || recipe.lines.len() > 100
        || recipe.servings == 0
        || recipe.servings > 1_000_000
    {
        return Err("Use 1–100 ingredient lines and 1–1000000 integer servings".to_owned());
    }
    let mut ingredients = Vec::with_capacity(recipe.lines.len());
    for line in &recipe.lines {
        if line.grams.0.atoms() == 0 {
            return Err("Ingredient mass must be positive".to_owned());
        }
        let ingredient = catalogue
            .find(&line.ingredient_id)
            .ok_or_else(|| format!("Unknown ingredient: {}", line.ingredient_id))?;
        if line.conversion.as_ref().is_some_and(|e| {
            e.form != ingredient.form
                || e.source_record_id
                    .as_ref()
                    .is_some_and(|id| id != &ingredient.source.record_id)
        }) {
            return Err("conversion_state_mismatch".to_owned());
        }
        ingredients.push((ingredient, &line.grams));
    }
    let nutrients = Nutrient::ALL
        .iter()
        .map(|&nutrient| calculate_nutrient(nutrient, &ingredients, recipe.servings, 1, 1))
        .collect::<Result<Vec<_>, _>>()?;
    let contributions = ingredients
        .iter()
        .map(|(i, grams)| Contribution {
            ingredient_id: i.id.clone(),
            name: i.name.clone(),
            form: i.form.clone(),
            basis: i.basis,
            grams: (*grams).clone(),
            source: i.source.clone(),
            uncertainty: i.uncertainty.clone(),
            education: i.education.clone(),
            translations: i.translations.clone(),
        })
        .collect();
    let mut warning_codes = vec!["composition_estimate"];
    let mut warnings = vec![
        "Composition is a source-based estimate; missing values are unknown, not zero. No individual benefit or allergen safety is established.",
    ];
    if recipe.lines.iter().any(|line| line.conversion.is_some()) {
        warning_codes.push("conversion_estimate");
        warnings.push("Ingredient masses include source-based or user-declared conversion estimates; original quantities and evidence remain in the saved recipe revision.");
    }
    if matches!(recipe.preparation, Preparation::CookingUnmodelled) {
        warning_codes.push("cooking_unmodelled");
        warnings.push("Cooking is unmodelled: no nutrient retention, cooking-loss or water/yield correction has been applied. These are input-composition estimates, not measured cooked-food values.");
    }
    if ["carbohydrate", "available_carbohydrate"].iter().all(|id| {
        nutrients
            .iter()
            .any(|n| n.info.id == *id && n.status == "partial")
    }) {
        warning_codes.push("carbohydrate_definitions");
        warnings.push("Sources use different carbohydrate definitions. By-difference and available carbohydrate totals stay separate and may be partial; no inferred conversion is applied.");
    }
    Ok(Estimate {
        catalogue_version: catalogue.version.clone(),
        servings: recipe.servings,
        preparation: recipe.preparation,
        rounding: "2 decimal places, half away from zero; scale unrounded totals for portions",
        nutrients,
        ingredients: contributions,
        warnings,
        warning_codes,
    })
}

fn calculate_nutrient(
    nutrient: Nutrient,
    ingredients: &[(&catalogue::Ingredient, &Grams)],
    servings: u32,
    factor_numerator: u32,
    factor_denominator: u32,
) -> Result<NutrientResult, String> {
    // Each source value is per 100 grams. Keep the entire rational numerator until display.
    let denominator = 100 * SCALE * SCALE;
    let mut numerator = 0_u128;
    let mut missing = Vec::new();
    let mut uncertainty = Vec::new();
    let mut known = 0;
    for (ingredient, grams) in ingredients {
        let reading = ingredient.nutrients.get(&nutrient);
        if let Some(amount) = reading.and_then(|r| r.amount.as_ref()) {
            numerator = amount
                .atoms()
                .checked_mul(grams.0.atoms())
                .and_then(|v| numerator.checked_add(v))
                .ok_or_else(|| "Quantity exceeds calculation precision".to_owned())?;
            known += 1;
        } else if !missing.contains(&ingredient.id) {
            missing.push(ingredient.id.clone());
        }
        if let Some(reading) = reading {
            uncertainty.push(format!("{}: {}", ingredient.id, reading.uncertainty));
        }
    }
    let numerator = numerator
        .checked_mul(u128::from(factor_numerator))
        .ok_or("Quantity exceeds calculation precision")?;
    let denominator = denominator * u128::from(factor_denominator);
    let total = rounded(numerator, denominator)?;
    let portion = rounded(numerator, denominator * u128::from(servings))?;
    let complete = missing.is_empty();
    Ok(NutrientResult {
        info: nutrient.info(),
        total: complete.then(|| total.clone()),
        per_serving: complete.then(|| portion.clone()),
        known_total: total,
        known_per_serving: portion,
        status: if complete {
            "complete"
        } else if known == 0 {
            "unknown"
        } else {
            "partial"
        },
        missing_ingredients: missing,
        uncertainty,
    })
}

// Scale source composition rationally, before either gram or nutrient display rounding.
pub(crate) fn scaled_nutrition(
    catalogue: &Catalogue,
    original: &Calculation,
    target: u32,
) -> Result<Estimate, String> {
    let mut result = calculate(catalogue, original)?;
    if target == 0 || target > 1_000_000 {
        return Err("Invalid serving count".to_owned());
    }
    let ingredients = original
        .lines
        .iter()
        .map(|line| {
            catalogue
                .find(&line.ingredient_id)
                .map(|i| (i, &line.grams))
                .ok_or_else(|| "Unknown ingredient".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    result.nutrients = Nutrient::ALL
        .iter()
        .map(|&n| calculate_nutrient(n, &ingredients, target, target, original.servings))
        .collect::<Result<Vec<_>, _>>()?;
    result.servings = target;
    for ingredient in &mut result.ingredients {
        ingredient.grams.0 = ingredient.grams.0.scaled(target, original.servings)?;
    }
    Ok(result)
}
