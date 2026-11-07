//! Computes nutrient estimates from validated ingredient records.

pub(crate) mod catalogue;
pub(crate) mod nutrients;

use crate::quantities::{Grams, SCALE, rounded};
use catalogue::{Basis, Catalogue, Education, Source};
use nutrients::{Nutrient, NutrientInfo};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipeLine {
    pub(crate) ingredient_id: String,
    pub(crate) grams: Grams,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Preparation {
    AsRecorded,
    CookingUnmodelled,
}

#[derive(Deserialize)]
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
        ingredients.push((ingredient, &line.grams));
    }
    let nutrients = Nutrient::ALL
        .iter()
        .map(|&nutrient| calculate_nutrient(nutrient, &ingredients, recipe.servings))
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
        })
        .collect();
    let mut warnings = vec![
        "Composition is a source-based estimate; missing values are unknown, not zero. No individual benefit or allergen safety is established.",
    ];
    if matches!(recipe.preparation, Preparation::CookingUnmodelled) {
        warnings.push("Cooking is unmodelled: no nutrient retention, cooking-loss or water/yield correction has been applied. These are input-composition estimates, not measured cooked-food values.");
    }
    Ok(Estimate {
        catalogue_version: catalogue.version.clone(),
        servings: recipe.servings,
        preparation: recipe.preparation,
        rounding: "2 decimal places, half away from zero; scale unrounded totals for portions",
        nutrients,
        ingredients: contributions,
        warnings,
    })
}

fn calculate_nutrient(
    nutrient: Nutrient,
    ingredients: &[(&catalogue::Ingredient, &Grams)],
    servings: u32,
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
