//! Exact outward-rounded nutrient envelopes; incomplete inputs remain explicit.
use super::{
    catalogue::Ingredient,
    estimates::{self, Value},
    nutrients::Nutrient,
};
use crate::quantities::{Amount, Grams, SCALE};
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct DisplayBounds {
    pub(crate) lower: String,
    pub(crate) upper: String,
}
#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub(crate) enum RangeResult {
    Complete {
        total: DisplayBounds,
        per_serving: DisplayBounds,
        estimated_ingredients: Vec<String>,
    },
    Partial {
        known_total: DisplayBounds,
        known_per_serving: DisplayBounds,
        estimated_ingredients: Vec<String>,
        missing_ingredients: Vec<String>,
    },
}
#[derive(Default)]
pub(crate) struct Accumulator {
    lower: u128,
    upper: u128,
    estimated: Vec<String>,
    missing: Vec<String>,
}
fn precision() -> String {
    "Quantity exceeds calculation precision".to_owned()
}
impl Accumulator {
    fn add(&mut self, lower: &Amount, upper: &Amount, grams: &Grams) -> Result<(), String> {
        self.lower = lower
            .atoms()
            .checked_mul(grams.0.atoms())
            .and_then(|v| self.lower.checked_add(v))
            .ok_or_else(precision)?;
        self.upper = upper
            .atoms()
            .checked_mul(grams.0.atoms())
            .and_then(|v| self.upper.checked_add(v))
            .ok_or_else(precision)?;
        Ok(())
    }
    pub(crate) fn ingredient(
        &mut self,
        ingredient: &Ingredient,
        grams: &Grams,
        nutrient: Nutrient,
    ) -> Result<(), String> {
        match estimates::value(ingredient, nutrient) {
            Value::Reported(value) => self.add(value, value, grams)?,
            Value::Estimated(range) => {
                self.add(&range.lower, &range.upper, grams)?;
                if !self.estimated.contains(&ingredient.id) {
                    self.estimated.push(ingredient.id.clone());
                }
            }
            Value::Unknown => {
                if !self.missing.contains(&ingredient.id) {
                    self.missing.push(ingredient.id.clone());
                }
            }
        }
        Ok(())
    }
    pub(crate) fn finish(
        self,
        servings: u32,
        numerator: u32,
        denominator: u32,
    ) -> Result<Option<RangeResult>, String> {
        if self.estimated.is_empty() {
            return Ok(None);
        }
        let lower = self
            .lower
            .checked_mul(u128::from(numerator))
            .ok_or_else(precision)?;
        let upper = self
            .upper
            .checked_mul(u128::from(numerator))
            .ok_or_else(precision)?;
        let divisor = 100 * SCALE * SCALE * u128::from(denominator);
        let total = display(lower, upper, divisor)?;
        let per_serving = display(lower, upper, divisor * u128::from(servings))?;
        Ok(Some(if self.missing.is_empty() {
            RangeResult::Complete {
                total,
                per_serving,
                estimated_ingredients: self.estimated,
            }
        } else {
            RangeResult::Partial {
                known_total: total,
                known_per_serving: per_serving,
                estimated_ingredients: self.estimated,
                missing_ingredients: self.missing,
            }
        }))
    }
}
fn display(lower: u128, upper: u128, divisor: u128) -> Result<DisplayBounds, String> {
    let low = lower.checked_mul(100).ok_or_else(precision)? / divisor;
    let high = upper
        .checked_mul(100)
        .and_then(|v| v.checked_add(divisor - 1))
        .ok_or_else(precision)?
        / divisor;
    Ok(DisplayBounds {
        lower: format!("{}.{:02}", low / 100, low % 100),
        upper: format!("{}.{:02}", high / 100, high % 100),
    })
}
