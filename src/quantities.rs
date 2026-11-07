//! Validates exact bounded decimal input without binary floating point.

use serde::{Deserialize, Serialize};

pub(crate) const SCALE: u128 = 1_000_000;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub(crate) struct Amount {
    original: String,
    atoms: u64,
}

impl Amount {
    pub(crate) fn scaled(&self, target: u32, original: u32) -> Result<Self, String> {
        if target == 0 || target > 1_000_000 || original == 0 {
            return Err("Invalid serving count".to_owned());
        }
        let atoms =
            (self.atoms() * u128::from(target) + u128::from(original) / 2) / u128::from(original);
        if atoms == 0 || atoms > 1_000_000_000_000_000 {
            return Err("Scaled amount is outside supported precision".to_owned());
        }
        Self::try_from(format!("{}.{:06}", atoms / SCALE, atoms % SCALE))
    }

    pub(crate) fn atoms(&self) -> u128 {
        u128::from(self.atoms)
    }
}

impl TryFrom<String> for Amount {
    type Error = String;

    fn try_from(original: String) -> Result<Self, Self::Error> {
        let invalid = || {
            "Expected a decimal string between 0 and 1000000000 with at most six fractional digits"
                .to_owned()
        };
        if original.len() > 32 {
            return Err(invalid());
        }
        let (whole, fraction) = original.split_once('.').unwrap_or((&original, ""));
        if whole.is_empty()
            || !whole.bytes().all(|b| b.is_ascii_digit())
            || fraction.len() > 6
            || !fraction.bytes().all(|b| b.is_ascii_digit())
            || (original.contains('.') && fraction.is_empty())
        {
            return Err(invalid());
        }
        let whole: u64 = whole.parse().map_err(|_| invalid())?;
        let fractional: u64 = format!("{fraction:0<6}").parse().map_err(|_| invalid())?;
        let atoms = whole
            .checked_mul(1_000_000)
            .and_then(|v| v.checked_add(fractional))
            .ok_or_else(invalid)?;
        if atoms > 1_000_000_000_000_000 {
            return Err(invalid());
        }
        Ok(Self { original, atoms })
    }
}

impl From<Amount> for String {
    fn from(amount: Amount) -> Self {
        amount.original
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub(crate) struct Grams(pub(crate) Amount);

pub(crate) fn rounded(numerator: u128, denominator: u128) -> Result<String, String> {
    let cents = numerator
        .checked_mul(100)
        .and_then(|v| v.checked_add(denominator / 2))
        .and_then(|v| v.checked_div(denominator))
        .ok_or_else(|| "Quantity exceeds calculation precision".to_owned())?;
    Ok(format!("{}.{:02}", cents / 100, cents % 100))
}

/// A source quantity retains its original decimal spelling and unit.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Quantity {
    pub(crate) amount: Amount,
    pub(crate) unit: Unit,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Unit {
    G,
    Kg,
    Ml,
    L,
    Tsp,
    Tbsp,
    Piece,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Dimension {
    Mass,
    Volume,
    Count,
}

impl Unit {
    pub(crate) fn dimension(self) -> Dimension {
        match self {
            Self::G | Self::Kg => Dimension::Mass,
            Self::Ml | Self::L | Self::Tsp | Self::Tbsp => Dimension::Volume,
            Self::Piece => Dimension::Count,
        }
    }
    fn factor(self) -> Option<u128> {
        match self {
            Self::G | Self::Ml | Self::Piece => Some(1),
            Self::Kg | Self::L => Some(1000),
            Self::Tsp | Self::Tbsp => None,
        }
    }
}
impl Quantity {
    pub(crate) fn convert(&self, target: Unit) -> Result<Self, &'static str> {
        if self.unit == target {
            return Ok(self.clone());
        }
        if self.unit.dimension() != target.dimension() {
            return Err("ingredient_conversion_required");
        }
        let from = self.unit.factor().ok_or("spoon_standard_required")?;
        let to = target.factor().ok_or("spoon_standard_required")?;
        let numerator = self.amount.atoms() * from;
        if numerator % to != 0 {
            return Err("precision_exceeded");
        }
        let atoms = numerator / to;
        if atoms > 1_000_000_000_000_000 {
            return Err("quantity_out_of_range");
        }
        let amount = Amount::try_from(format!("{}.{:06}", atoms / SCALE, atoms % SCALE))
            .map_err(|_| "quantity_out_of_range")?;
        Ok(Self {
            amount,
            unit: target,
        })
    }
}
