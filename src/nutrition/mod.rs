//! Computes nutrient estimates from validated ingredient records.

pub(crate) mod catalogue;
pub(crate) mod estimates;
pub(crate) mod nutrients;
mod ranges;

use crate::quantities::{Grams, SCALE, rounded};
use catalogue::{Basis, Catalogue, Education, IngredientText, Source};
use nutrients::{Nutrient, NutrientInfo};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Serialize)]
pub(crate) struct RecipeLine {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) prepared_part: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) extensions: Vec<crate::ingredient_extensions::Extension>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) part_id: Option<String>,
    pub(crate) ingredient_id: String,
    /// Absent while the cook has not said what a volume, spoon or piece quantity weighs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) grams: Option<Grams>,
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
            prepared_part: Option<String>,
            #[serde(default)]
            extensions: Vec<crate::ingredient_extensions::Extension>,
            id: Option<String>,
            part_id: Option<String>,
            ingredient_id: String,
            grams: Option<Grams>,
            quantity: Option<crate::quantities::Quantity>,
            conversion: Option<crate::quantities::ConversionEvidence>,
        }
        let input = Input::deserialize(deserializer)?;
        crate::ingredient_extensions::validate(&input.extensions)
            .map_err(serde::de::Error::custom)?;
        let unweighed = input.quantity.as_ref().is_some_and(|q| {
            input.conversion.is_none() && q.unit.dimension() != crate::quantities::Dimension::Mass
        });
        let grams = if unweighed {
            // Unknown stays unknown: the original quantity is kept and no mass is derived.
            if input.grams.is_some() {
                return Err(serde::de::Error::custom("quantity_grams_mismatch"));
            }
            None
        } else if let Some(q) = &input.quantity {
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
            Some(result)
        } else {
            if input.conversion.is_some() {
                return Err(serde::de::Error::custom("conversion_without_quantity"));
            }
            Some(
                input
                    .grams
                    .ok_or_else(|| serde::de::Error::custom("quantity_required"))?,
            )
        };
        Ok(Self {
            prepared_part: input.prepared_part,
            extensions: input.extensions,
            id: input.id,
            part_id: input.part_id,
            ingredient_id: input.ingredient_id,
            grams,
            quantity: input.quantity,
            conversion: input.conversion,
        })
    }
}

/// Where a line's mass comes from; anything but `Weighed` is shown to the cook.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MassBasis {
    Weighed,
    SourcePortion,
    HouseholdWeight,
    RoughEstimate,
    Unknown,
}
impl RecipeLine {
    pub(crate) fn mass_basis(&self) -> MassBasis {
        match (&self.grams, &self.conversion) {
            (None, _) => MassBasis::Unknown,
            (Some(_), None) => MassBasis::Weighed,
            (Some(_), Some(e)) if !e.source_url.is_empty() => MassBasis::SourcePortion,
            (Some(_), Some(e)) if e.version.starts_with("household estimate") => {
                MassBasis::RoughEstimate
            }
            (Some(_), Some(_)) => MassBasis::HouseholdWeight,
        }
    }
    /// The line for another serving count; an unweighed line scales its original quantity.
    pub(crate) fn scaled(&self, target: u32, original: u32) -> Result<Self, String> {
        let mut line = self.clone();
        match &self.grams {
            Some(grams) => {
                line.grams = Some(Grams(grams.0.scaled(target, original)?));
                line.quantity = None;
                line.conversion = None;
            }
            None => {
                if let Some(quantity) = &mut line.quantity {
                    quantity.amount = quantity.amount.scaled(target, original)?;
                }
            }
        }
        Ok(line)
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    nutrition_estimates: Vec<estimates::Profile>,
    nutrient_ranges: BTreeMap<Nutrient, estimates::Bounds>,
    ingredient_id: String,
    name: String,
    form: String,
    basis: Basis,
    #[serde(skip_serializing_if = "Option::is_none")]
    grams: Option<Grams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quantity: Option<crate::quantities::Quantity>,
    mass_basis: MassBasis,
    source: Source,
    uncertainty: String,
    education: Vec<Education>,
    translations: BTreeMap<String, IngredientText>,
}

#[derive(Serialize)]
pub(crate) struct NutrientResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    range: Option<ranges::RangeResult>,
    #[serde(flatten)]
    info: NutrientInfo,
    total: Option<String>,
    per_serving: Option<String>,
    known_total: String,
    known_per_serving: String,
    status: &'static str,
    missing_ingredients: Vec<String>,
    uncertainty: Vec<String>,
    #[serde(skip)]
    known_total_is_empty: bool,
}

#[derive(Serialize)]
pub(crate) struct Estimate {
    constituents: Vec<crate::constituents::ResultRow>,
    catalogue_version: String,
    servings: u32,
    preparation: Preparation,
    rounding: &'static str,
    nutrients: Vec<NutrientResult>,
    ingredients: Vec<Contribution>,
    warnings: Vec<&'static str>,
    warning_codes: Vec<&'static str>,
}

pub(crate) fn validate_line<'a>(
    catalogue: &'a Catalogue,
    line: &RecipeLine,
) -> Result<&'a catalogue::Ingredient, String> {
    if line.grams.as_ref().is_some_and(|g| g.0.atoms() == 0)
        || (line.grams.is_none() && line.quantity.as_ref().is_none_or(|q| q.amount.atoms() == 0))
    {
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
    Ok(ingredient)
}
pub(crate) fn calculate(catalogue: &Catalogue, recipe: &Calculation) -> Result<Estimate, String> {
    if recipe.lines.is_empty()
        || recipe.lines.len() > 100
        || recipe.servings == 0
        || recipe.servings > 1_000_000
    {
        return Err("Use 1–100 ingredient lines and 1–1000000 integer servings".to_owned());
    }
    if recipe.lines.iter().any(|line| line.prepared_part.is_some()) {
        return Err(
            "Resolve homemade ingredient parts through a saved recipe before calculating nutrition"
                .to_owned(),
        );
    }
    let mut ingredients = Vec::with_capacity(recipe.lines.len());
    let mut all = Vec::with_capacity(recipe.lines.len());
    let mut unweighed = Vec::new();
    for line in &recipe.lines {
        let ingredient = validate_line(catalogue, line)?;
        all.push((ingredient, line));
        match &line.grams {
            Some(grams) => ingredients.push((ingredient, grams)),
            None if !unweighed.contains(&ingredient.id) => unweighed.push(ingredient.id.clone()),
            None => {}
        }
    }
    let mut nutrients = Nutrient::ALL
        .iter()
        .map(|&nutrient| calculate_nutrient(nutrient, &ingredients, recipe.servings, 1, 1))
        .collect::<Result<Vec<_>, _>>()?;
    leave_unweighed_unknown(&mut nutrients, &unweighed);
    let contributions = all
        .iter()
        .map(|(i, line)| Contribution {
            nutrition_estimates: i.nutrition_estimates.clone(),
            nutrient_ranges: i
                .nutrients
                .iter()
                .filter_map(|(id, r)| r.range.clone().map(|bounds| (*id, bounds)))
                .collect(),
            ingredient_id: i.id.clone(),
            name: i.name.clone(),
            form: i.form.clone(),
            basis: i.basis,
            grams: line.grams.clone(),
            quantity: line.quantity.clone().filter(|_| line.grams.is_none()),
            mass_basis: line.mass_basis(),
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
    if ingredients.iter().any(|(i, _)| estimates::is_estimated(i)) {
        warning_codes.push("comparison_nutrition");
        warnings.push("Nutrition includes explicit estimated ranges. Reference envelopes describe selected observations, not guaranteed bounds or statistical confidence intervals.");
    }
    if recipe.lines.iter().any(|line| line.conversion.is_some()) {
        warning_codes.push("conversion_estimate");
        warnings.push("Ingredient masses include source-based or user-declared conversion estimates; original quantities and evidence remain in the saved recipe revision.");
    }
    mass_warnings(recipe, &unweighed, &mut warning_codes, &mut warnings);
    if matches!(recipe.preparation, Preparation::CookingUnmodelled) {
        warning_codes.push("cooking_unmodelled");
        warnings.push("Cooking is unmodelled: no nutrient retention, cooking-loss or water/yield correction has been applied. These are input-composition estimates, not measured cooked-food values.");
    }
    if ["carbohydrate", "available_carbohydrate"].iter().all(|id| {
        nutrients
            .iter()
            .any(|n| n.info.id == *id && (n.status != "unknown" || n.range.is_some()))
    }) {
        warning_codes.push("carbohydrate_definitions");
        warnings.push("Sources use different carbohydrate definitions. By-difference and available carbohydrate totals stay separate and may be partial; no inferred conversion is applied.");
    }
    let mut constituents = crate::constituents::calculate(&ingredients, recipe.servings, 1, 1)?;
    for row in &mut constituents {
        row.leave_unknown(&unweighed);
    }
    Ok(Estimate {
        constituents,
        catalogue_version: catalogue.version.clone(),
        servings: recipe.servings,
        preparation: recipe.preparation,
        rounding: "2 decimal places; reported totals: half away from zero; ranges: lower down, upper up; scale unrounded values for portions",
        nutrients,
        ingredients: contributions,
        warnings,
        warning_codes,
    })
}

/// Says where ingredient masses are estimated, household-declared or missing.
fn mass_warnings(
    recipe: &Calculation,
    unweighed: &[String],
    warning_codes: &mut Vec<&'static str>,
    warnings: &mut Vec<&'static str>,
) {
    let basis = |wanted| recipe.lines.iter().any(|line| line.mass_basis() == wanted);
    if basis(MassBasis::RoughEstimate) {
        warning_codes.push("rough_mass_estimate");
        warnings.push("Some ingredient masses are rough general estimates accepted by the household (as if the ingredient weighed like water), not weighed or source-backed values.");
    }
    if basis(MassBasis::HouseholdWeight) {
        warning_codes.push("household_mass");
        warnings.push("Some ingredient masses use a weight per unit declared by the household without an external source.");
    }
    if !unweighed.is_empty() {
        warning_codes.push("unweighed_ingredients");
        warnings.push("Nutrition is incomplete: some ingredients have a quantity but no known mass. Their contribution is unknown, not zero, and no total is reported.");
    }
}

/// An ingredient without a known mass makes every total unknown; only the known part remains.
fn leave_unweighed_unknown(nutrients: &mut [NutrientResult], unweighed: &[String]) {
    if unweighed.is_empty() {
        return;
    }
    for result in nutrients {
        for id in unweighed {
            if !result.missing_ingredients.contains(id) {
                result.missing_ingredients.push(id.clone());
            }
        }
        result.status = if result.status == "unknown" || result.known_total_is_empty {
            "unknown"
        } else {
            "partial"
        };
        result.total = None;
        result.per_serving = None;
        result.range = None;
    }
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
    let mut range = ranges::Accumulator::default();
    for (ingredient, grams) in ingredients {
        range.ingredient(ingredient, grams, nutrient)?;
        let reading = ingredient.nutrients.get(&nutrient);
        if let estimates::Value::Reported(amount) = estimates::value(ingredient, nutrient) {
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
        range: range.finish(servings, factor_numerator, factor_denominator)?,
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
        known_total_is_empty: known == 0,
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
    let mut ingredients = Vec::new();
    let mut unweighed = Vec::new();
    for line in &original.lines {
        let ingredient = catalogue
            .find(&line.ingredient_id)
            .ok_or_else(|| "Unknown ingredient".to_owned())?;
        match &line.grams {
            Some(grams) => ingredients.push((ingredient, grams)),
            None if !unweighed.contains(&ingredient.id) => unweighed.push(ingredient.id.clone()),
            None => {}
        }
    }
    result.nutrients = Nutrient::ALL
        .iter()
        .map(|&n| calculate_nutrient(n, &ingredients, target, target, original.servings))
        .collect::<Result<Vec<_>, _>>()?;
    leave_unweighed_unknown(&mut result.nutrients, &unweighed);
    result.constituents =
        crate::constituents::calculate(&ingredients, target, target, original.servings)?;
    for row in &mut result.constituents {
        row.leave_unknown(&unweighed);
    }
    result.servings = target;
    for ingredient in &mut result.ingredients {
        if let Some(grams) = &mut ingredient.grams {
            grams.0 = grams.0.scaled(target, original.servings)?;
        }
        if let Some(quantity) = &mut ingredient.quantity {
            quantity.amount = quantity.amount.scaled(target, original.servings)?;
        }
    }
    Ok(result)
}
