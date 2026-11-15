//! Reviewed local capture is evidence, never an ingredient match or a health assertion.
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Photo {
    #[serde(default)]
    pub(crate) rotation: u16,
    pub(crate) image_id: String,
    pub(crate) text: String,
    pub(crate) engine: String,
    pub(crate) score: Option<u8>,
}
fn text(value: &str, max: usize) -> bool {
    value.len() <= max
        && !value
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
}
impl Photo {
    pub(crate) fn valid(&self) -> bool {
        matches!(self.rotation, 0 | 90 | 180 | 270)
            && crate::recipe_images::valid_id(&self.image_id)
            && text(&self.text, 32_768)
            && match self.engine.as_str() {
                "tesseract-5.5.1-fast-eng-deu" => self.score.is_none_or(|v| v <= 100),
                "manual_transcription" => self.score.is_none(),
                _ => false,
            }
    }
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Qualifier {
    Exact,
    LessThan,
    Approximate,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Declaration {
    pub(crate) qualifier: Qualifier,
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) unit: String,
    /// Original printed definition, e.g. available carbohydrate. None means unknown.
    pub(crate) definition: Option<String>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Label {
    pub(crate) photo: Photo,
    /// Exact printed basis, not an assumed conversion to 100 g.
    pub(crate) basis: String,
    pub(crate) values: Vec<Declaration>,
    pub(crate) reviewed: bool,
}
impl Label {
    pub(crate) fn valid(&self) -> bool {
        self.reviewed
            && self.photo.valid()
            && self.photo.text.len() <= 4096
            && !self.basis.trim().is_empty()
            && text(&self.basis, 200)
            && self.values.len() <= 20
            && self.values.iter().all(|v| {
                !v.name.trim().is_empty()
                    && text(&v.name, 120)
                    && matches!(v.unit.as_str(), "g" | "mg" | "µg" | "kJ" | "kcal" | "%")
                    && crate::quantities::Amount::try_from(v.value.clone()).is_ok()
                    && v.definition.as_ref().is_none_or(|s| text(s, 400))
            })
    }
}
