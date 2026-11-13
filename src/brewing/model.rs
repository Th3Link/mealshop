use super::templates::{Evidence, Heat, Layout, PhaseKind, Sparge, Strategy};
use crate::{
    cooking::{LocalTime, Stage},
    shopping::model::Quantity,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Mash {
    None,
    Infusion,
    Step,
    Decoction,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Method {
    pub(crate) strategy: Strategy,
    pub(crate) layout: Layout,
    pub(crate) heat: Heat,
    pub(crate) sparge: Sparge,
    pub(crate) mash: Mash,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TemplateRef {
    pub(crate) id: String,
    pub(crate) version: u32,
    pub(crate) sources: Vec<Evidence>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Beverage {
    Beer,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InputLine {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) ingredient_id: Option<String>,
    pub(crate) quantity: Option<Quantity>,
    pub(crate) phase_id: Option<String>,
    pub(crate) note: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReadingUnit {
    Celsius,
    Fahrenheit,
    Sg,
    Plato,
    Brix,
    Ml,
    L,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Measurement {
    pub(crate) value: String,
    pub(crate) unit: ReadingUnit,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Phase {
    pub(crate) kind: PhaseKind,
    pub(crate) timing_known: bool,
    pub(crate) stage: Stage,
    pub(crate) temperature: Option<Measurement>,
    pub(crate) volume: Option<Quantity>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Recipe {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) yeast_preparation: Option<RecipePreparation>,
    pub(crate) beverage: Beverage,
    pub(crate) title: String,
    pub(crate) language: String,
    pub(crate) volume: Quantity,
    pub(crate) template: TemplateRef,
    pub(crate) method: Method,
    pub(crate) ingredients: Vec<InputLine>,
    pub(crate) phases: Vec<Phase>,
    pub(crate) notes: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Record {
    pub(crate) id: String,
    pub(crate) revision: u64,
    pub(crate) recipe: Recipe,
    pub(crate) actor: String,
    pub(crate) recorded_at: i64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EquipmentKind {
    Kettle,
    MashTun,
    HotLiquorTank,
    Pump,
    Chiller,
    Fermenter,
    Bag,
    Other,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EquipmentInput {
    pub(crate) name: String,
    pub(crate) kind: EquipmentKind,
    pub(crate) model: String,
    pub(crate) capacity: Option<Quantity>,
    pub(crate) notes: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Equipment {
    pub(crate) id: String,
    pub(crate) details: EquipmentInput,
    pub(crate) archived: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipePreparation {
    pub(crate) days_before: u8,
    pub(crate) local_time: String,
    pub(crate) fold: crate::cooking::time::Fold,
    pub(crate) instruction: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "choice", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum PreparationChoice {
    Recipe,
    Skip,
    Custom { at: LocalTime, instruction: String },
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct YeastPreparation {
    pub(crate) at: LocalTime,
    pub(crate) instruction: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BatchDraft {
    pub(crate) name: String,
    pub(crate) recipe_id: String,
    pub(crate) recipe_revision: u64,
    pub(crate) brew_day: LocalTime,
    pub(crate) yeast_preparation: PreparationChoice,
    pub(crate) equipment: BTreeMap<String, Vec<String>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Batch {
    pub(crate) draft: BatchDraft,
    pub(crate) equipment: Vec<Equipment>,
    pub(crate) actor: String,
    pub(crate) recorded_at: i64,
    pub(crate) archived: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Acquisition {
    Transcribed,
    Reported,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum ReadingSource {
    Manual,
    Instrument {
        device_id: String,
        name: String,
        acquisition: Acquisition,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Calibration {
    Unknown,
    Declared {
        details: String,
        calibrated_at: Option<LocalTime>,
        reference_temperature: Option<Measurement>,
        source_url: Option<String>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReadingInput {
    pub(crate) phase_id: Option<String>,
    pub(crate) measurement: Measurement,
    pub(crate) measured_at: LocalTime,
    pub(crate) source: ReadingSource,
    pub(crate) calibration: Calibration,
    pub(crate) notes: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Reading {
    pub(crate) id: String,
    pub(crate) run_id: String,
    pub(crate) input: ReadingInput,
    pub(crate) measured_instant: i64,
    pub(crate) actor: String,
    pub(crate) recorded_at: i64,
    pub(crate) corrects: Option<String>,
    pub(crate) reason: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Receipt {
    pub(crate) id: String,
    pub(crate) actor: String,
    pub(crate) key: String,
    pub(crate) expected_revision: u64,
    pub(crate) command: serde_json::Value,
    pub(crate) recorded_at: i64,
    pub(crate) result_id: Option<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Data {
    pub(crate) recipes: Vec<Record>,
    pub(crate) archived_recipes: Vec<String>,
    pub(crate) equipment: Vec<Equipment>,
    pub(crate) batches: BTreeMap<String, Batch>,
    pub(crate) readings: Vec<Reading>,
    pub(crate) operations: Vec<Receipt>,
}
impl Data {
    pub(crate) fn is_empty(&self) -> bool {
        self.recipes.is_empty()
            && self.archived_recipes.is_empty()
            && self.equipment.is_empty()
            && self.batches.is_empty()
            && self.readings.is_empty()
            && self.operations.is_empty()
    }
    pub(crate) fn recipe(&self, id: &str, revision: u64) -> Result<&Record, &'static str> {
        self.recipes
            .iter()
            .find(|r| r.id == id && r.revision == revision)
            .ok_or("brewing_recipe_not_found")
    }
}
