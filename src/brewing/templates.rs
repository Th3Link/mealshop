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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Evidence {
    pub(crate) id: String,
    pub(crate) publisher: String,
    pub(crate) title: String,
    pub(crate) url: String,
    pub(crate) document_version: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PhaseSeed {
    pub(crate) kind: String,
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
    pub(crate) beverage: String,
    pub(crate) name: String,
    pub(crate) translations: BTreeMap<String, String>,
    pub(crate) strategy: Strategy,
    pub(crate) layout: Layout,
    pub(crate) heat: Heat,
    pub(crate) sparge: Sparge,
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
            t.beverage != "beer"
                || t.version != 1
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
        assert_eq!(c.templates.len(), 12);
        let find = |id| c.templates.iter().find(|t| t.id == id).unwrap();
        assert_eq!(find("biab").strategy, Strategy::AllGrain);
        assert_eq!(find("partial-mash").strategy, Strategy::PartialMash);
        assert!(find("partial-mash").phases.contains(&"mash".to_owned()));
        assert!(find("partial-mash").phases.contains(&"extract".to_owned()));
        assert!(!find("extract-steep").phases.contains(&"mash".to_owned()));
        assert_eq!(find("grainfather").layout, Layout::TopReturn);
        assert_eq!(find("braumeister").layout, Layout::MaltPipe);
        assert_eq!(find("diy-herms").heat, Heat::Herms);
        assert_eq!(find("diy-rims").heat, Heat::Rims);
        assert!(c.templates.iter().all(|t| t.beverage == "beer"));
    }
}
