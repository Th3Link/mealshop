//! Imported recipe provenance survives edits and controls every shared projection.
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Origin {
    #[default]
    Owned,
    Imported {
        source_url: String,
    },
}
impl Origin {
    pub(crate) fn is_owned(&self) -> bool {
        matches!(self, Self::Owned)
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if let Self::Imported { source_url } = self {
            let valid = reqwest::Url::parse(source_url).is_ok_and(|url| {
                url.scheme() == "https"
                    && url.host_str().is_some()
                    && url.username().is_empty()
                    && url.password().is_none()
            });
            if !valid || source_url.len() > 2048 || source_url.chars().any(char::is_whitespace) {
                return Err(
                    "Provide the original HTTPS recipe URL without embedded credentials".to_owned(),
                );
            }
        }
        Ok(())
    }

    pub(crate) fn follows(&self, previous: &Self) -> bool {
        previous.is_owned() || self == previous
    }
}
