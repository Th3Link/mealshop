use super::model::{Beverage, FermentationProcess};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Strategy {
    Extract,
    ExtractSteep,
    PartialMash,
    AllGrain,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Layout {
    Kettle,
    Bag,
    TopReturn,
    MaltPipe,
    SeparateTun,
    TwoVessel,
    ThreeVessel,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Heat {
    Manual,
    Direct,
    Herms,
    Rims,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Sparge {
    None,
    BasketRinse,
    Batch,
    Fly,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PhaseKind {
    Water,
    Steep,
    Mash,
    Lauter,
    BagLift,
    BasketLift,
    MaltPipeLift,
    Sparge,
    Extract,
    Boil,
    Addition,
    Cool,
    Transfer,
    Pitch,
    Ferment,
    Package,
    Clean,
    YeastPreparation,
    FruitPreparation,
    Press,
    Juice,
    Must,
    Macerate,
    Rack,
    Mature,
    Custom,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Evidence {
    pub(crate) id: String,
    pub(crate) publisher: String,
    pub(crate) title: String,
    pub(crate) url: String,
    pub(crate) document_version: Option<String>,
    pub(crate) retrieved_on: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PhaseSeed {
    pub(crate) kind: PhaseKind,
    pub(crate) name: String,
    pub(crate) translations: BTreeMap<String, String>,
    pub(crate) instruction: String,
    pub(crate) instruction_translations: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Template {
    pub(crate) id: String,
    pub(crate) version: u32,
    pub(crate) beverage: Beverage,
    pub(crate) name: String,
    pub(crate) translations: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) strategy: Option<Strategy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) layout: Option<Layout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) heat: Option<Heat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) sparge: Option<Sparge>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) process: Option<FermentationProcess>,
    pub(crate) sources: Vec<String>,
    pub(crate) phases: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Catalogue {
    pub(crate) schema_version: u32,
    pub(crate) retrieved_on: String,
    pub(crate) sources: Vec<Evidence>,
    pub(crate) phases: BTreeMap<String, PhaseSeed>,
    pub(crate) templates: Vec<Template>,
}
pub(crate) fn catalogue() -> Result<Catalogue, &'static str> {
    let value: Catalogue = serde_json::from_str(include_str!("../../data/brewing-processes.json"))
        .map_err(|_| "brewing_templates_unavailable")?;
    let sources = value.sources.iter().map(|s| &s.id).collect::<BTreeSet<_>>();
    if value.schema_version != 1
        || !crate::planning::date(&value.retrieved_on)
        || sources.len() != value.sources.len()
        || value
            .sources
            .iter()
            .any(|s| !s.url.starts_with("https://") || s.publisher.is_empty() || s.title.is_empty())
        || value
            .templates
            .iter()
            .map(|t| &t.id)
            .collect::<BTreeSet<_>>()
            .len()
            != value.templates.len()
        || value.templates.iter().any(|t| {
            (if t.beverage == Beverage::Beer {
                t.strategy.is_none()
                    || t.layout.is_none()
                    || t.heat.is_none()
                    || t.sparge.is_none()
                    || t.process.is_some()
            } else {
                t.strategy.is_some()
                    || t.layout.is_some()
                    || t.heat.is_some()
                    || t.sparge.is_some()
                    || t.process.is_none_or(|p| p.beverage() != t.beverage)
            }) || t.version != 1
                || t.sources.is_empty()
                || t.sources.iter().any(|s| !sources.contains(s))
                || t.phases.is_empty()
                || t.phases.iter().any(|id| !value.phases.contains_key(id))
        })
    {
        return Err("brewing_templates_unavailable");
    }
    Ok(value)
}
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn researched_templates_keep_distinct_methods_equipment_and_primary_sources() {
        let c = catalogue().unwrap();
        assert_eq!(c.templates.len(), 16);
        let find = |id| c.templates.iter().find(|t| t.id == id).unwrap();
        assert_eq!(find("biab").strategy, Some(Strategy::AllGrain));
        assert_eq!(find("partial-mash").strategy, Some(Strategy::PartialMash));
        assert!(find("partial-mash").phases.contains(&"mash".to_owned()));
        assert!(find("partial-mash").phases.contains(&"extract".to_owned()));
        assert!(!find("extract-steep").phases.contains(&"mash".to_owned()));
        assert_eq!(find("grainfather").layout, Some(Layout::TopReturn));
        assert_eq!(find("braumeister").layout, Some(Layout::MaltPipe));
        assert_eq!(find("diy-herms").heat, Some(Heat::Herms));
        assert_eq!(find("diy-rims").heat, Some(Heat::Rims));
        assert_eq!(
            c.templates
                .iter()
                .filter(|t| t.beverage == Beverage::Beer)
                .count(),
            12
        );
        for id in ["cider-juice", "white-wine", "red-wine", "traditional-mead"] {
            let t = find(id);
            assert_eq!(t.process.unwrap().beverage(), t.beverage);
            assert!(
                t.strategy.is_none()
                    && t.layout.is_none()
                    && t.heat.is_none()
                    && t.sparge.is_none()
            );
        }
        assert!(
            find("white-wine")
                .phases
                .iter()
                .position(|p| p == "wine-press")
                .unwrap()
                < find("white-wine")
                    .phases
                    .iter()
                    .position(|p| p == "wine-ferment")
                    .unwrap()
        );
        assert!(
            find("red-wine")
                .phases
                .iter()
                .position(|p| p == "wine-press")
                .unwrap()
                > find("red-wine")
                    .phases
                    .iter()
                    .position(|p| p == "red-macerate")
                    .unwrap()
        );
    }
}
