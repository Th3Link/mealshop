//! Defines the units of the supported nutrient vocabulary.

use serde::{Deserialize, Serialize};

macro_rules! nutrients {
    ($($variant:ident => ($id:literal, $label:literal, $unit:literal, $group:literal)),* $(,)?) => {
        #[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
        #[serde(rename_all = "snake_case")]
        pub(crate) enum Nutrient { $($variant),* }
        impl Nutrient {
            pub(crate) const ALL: &'static [Self] = &[$(Self::$variant),*];
            pub(crate) fn info(self) -> NutrientInfo {
                match self { $(Self::$variant => NutrientInfo { id: $id, label: $label, unit: $unit, group: $group }),* }
            }
        }
    };
}

#[derive(Serialize)]
pub(crate) struct NutrientInfo {
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) unit: &'static str,
    pub(crate) group: &'static str,
}

nutrients! {
    EnergyKcal => ("energy_kcal", "Energy", "kcal", "energy"),
    EnergyKj => ("energy_kj", "Energy", "kJ", "energy"),
    Protein => ("protein", "Protein", "g", "macro"),
    Fat => ("fat", "Fat", "g", "macro"),
    Carbohydrate => ("carbohydrate", "Carbohydrate (by difference, includes fibre)", "g", "macro"),
    Fibre => ("fibre", "Dietary fibre", "g", "macro"),
    Calcium => ("calcium", "Calcium", "mg", "micro"),
    Iron => ("iron", "Iron", "mg", "micro"),
    Magnesium => ("magnesium", "Magnesium", "mg", "micro"),
    Potassium => ("potassium", "Potassium", "mg", "micro"),
    Sodium => ("sodium", "Sodium", "mg", "micro"),
    VitaminC => ("vitamin_c", "Vitamin C", "mg", "micro"),
    Folate => ("folate", "Folate (total)", "µg", "micro"),
    BetaGlucan => ("beta_glucan", "Beta-glucan", "g", "component"),
    SolubleFibre => ("soluble_fibre", "Soluble fibre", "g", "component"),
}
