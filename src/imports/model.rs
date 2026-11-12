use serde::{Deserialize, Serialize};
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Candidate {
    pub(crate) title: Option<String>,
    pub(crate) author: Option<String>,
    pub(crate) language: Option<String>,
    pub(crate) yield_text: Option<String>,
    pub(crate) ingredients: Vec<String>,
    pub(crate) steps: Vec<SourceStep>,
    pub(crate) prep_time: Option<String>,
    pub(crate) cook_time: Option<String>,
    pub(crate) total_time: Option<String>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceStep {
    pub(crate) text: String,
    pub(crate) section: Option<String>,
}
#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Method {
    JsonLd,
    Manual,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Source {
    pub(crate) requested_url: String,
    pub(crate) final_url: Option<String>,
    pub(crate) method: Method,
    pub(crate) retrieved_at: Option<i64>,
    pub(crate) importer_version: String,
    pub(crate) original: Candidate,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Receipt {
    pub(crate) actor: String,
    pub(crate) key: String,
    pub(crate) preview_id: String,
    pub(crate) candidate: usize,
    pub(crate) recorded_at: i64,
    pub(crate) source: Source,
}
fn text(value: &str, max: usize) -> bool {
    value.len() <= max
        && !value
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
}
impl Candidate {
    pub(crate) fn valid(&self) -> bool {
        [
            &self.title,
            &self.author,
            &self.language,
            &self.yield_text,
            &self.prep_time,
            &self.cook_time,
            &self.total_time,
        ]
        .iter()
        .all(|s| s.as_ref().is_none_or(|s| text(s, 200)))
            && self.ingredients.len() <= 100
            && self
                .ingredients
                .iter()
                .all(|s| !s.trim().is_empty() && text(s, 4000))
            && self.steps.len() <= 100
            && self.steps.iter().all(|s| {
                !s.text.trim().is_empty()
                    && text(&s.text, 4000)
                    && s.section.as_ref().is_none_or(|s| text(s, 200))
            })
    }
}
impl Source {
    pub(crate) fn valid(&self) -> bool {
        crate::recipe_origin::Origin::Imported {
            source_url: self.requested_url.clone(),
        }
        .validate()
        .is_ok()
            && self.final_url.as_ref().is_none_or(|u| {
                crate::recipe_origin::Origin::Imported {
                    source_url: u.clone(),
                }
                .validate()
                .is_ok()
            })
            && self.importer_version == "schema-recipe-1"
            && self.retrieved_at.is_none_or(|n| n >= 0)
            && match self.method {
                Method::JsonLd => self.final_url.is_some() && self.retrieved_at.is_some(),
                Method::Manual => self.final_url.is_none() && self.retrieved_at.is_none(),
            }
            && self.original.valid()
            && serde_json::to_vec(self).is_ok_and(|v| v.len() <= 98_304)
    }
}
impl Receipt {
    pub(crate) fn valid(&self) -> bool {
        crate::membership::valid_user(&self.actor)
            && crate::recipes::Creation::valid_key(&self.key)
            && self.preview_id.starts_with("import-")
            && self.preview_id.len() == 71
            && self.preview_id[7..].bytes().all(|b| b.is_ascii_hexdigit())
            && self.candidate < 16
            && self.recorded_at >= 0
            && self.source.valid()
    }
}
