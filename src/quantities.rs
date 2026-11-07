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
