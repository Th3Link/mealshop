//! Recipe-authored preparation targets; never measured device values or mass conversions.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub(crate) struct Celsius(String);
impl TryFrom<String> for Celsius {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let magnitude = value.strip_prefix('-').unwrap_or(&value);
        let amount = crate::quantities::Amount::try_from(magnitude.to_owned())?;
        let limit = if value.starts_with('-') {
            273_150_000
        } else {
            1_000_000_000
        };
        if amount.atoms() > limit {
            return Err("Temperature must be between -273.15 and 1000 Celsius".into());
        }
        Ok(Self(value))
    }
}
impl From<Celsius> for String {
    fn from(value: Celsius) -> Self {
        value.0
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TemperatureUnit {
    Celsius,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Provenance {
    RecipeAuthor,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Extension {
    Temperature {
        value: Celsius,
        unit: TemperatureUnit,
        original_input: String,
        provenance: Provenance,
    },
}
pub(crate) fn validate(items: &[Extension]) -> Result<(), &'static str> {
    if items.len() > 1 {
        return Err("Duplicate ingredient temperature extension");
    }
    for item in items {
        match item {
            Extension::Temperature { original_input, .. } => {
                if original_input.trim().is_empty()
                    || original_input.len() > 100
                    || original_input.chars().any(char::is_control)
                {
                    return Err(
                        "Temperature original input must contain 1–100 bytes without control characters",
                    );
                }
            }
        }
    }
    Ok(())
}
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn exact_targets_preserve_author_input_and_reject_invalid_dimensions() {
        let target = json!({"type":"temperature","value":"60.00","unit":"celsius","original_input":"60,00 °C","provenance":"recipe_author"});
        let parsed: Extension = serde_json::from_value(target.clone()).unwrap();
        validate(std::slice::from_ref(&parsed)).unwrap();
        assert_eq!(serde_json::to_value(&parsed).unwrap(), target);
        assert!(validate(&[parsed.clone(), parsed]).is_err());
        for value in [
            json!(60),
            json!("NaN"),
            json!("60 °C"),
            json!("-274"),
            json!("1001"),
            json!("60.0000001"),
        ] {
            let mut bad = target.clone();
            bad["value"] = value;
            assert!(serde_json::from_value::<Extension>(bad).is_err());
        }
        for key in ["unit", "type", "provenance"] {
            let mut bad = target.clone();
            bad[key] = json!("unsupported");
            assert!(serde_json::from_value::<Extension>(bad).is_err());
        }
        let mut bad = target;
        bad["original_input"] = json!("");
        assert!(validate(&[serde_json::from_value(bad).unwrap()]).is_err());
    }
}
