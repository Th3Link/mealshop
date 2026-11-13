//! Person-specific rules and protected activity records.
mod rules;
pub(crate) use rules::{Rules, diet_review, planning_conflicts};

mod state;
pub(in crate::household) use state::{Document, Private};

mod activity;
pub(crate) use activity::{activity_path, routes};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Avatar {
    pub(crate) image_id: Option<String>,
}
impl Avatar {
    pub(crate) fn valid(&self) -> bool {
        self.image_id
            .as_deref()
            .is_none_or(crate::recipe_images::valid_id)
    }
}
