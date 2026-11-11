//! Household collection metadata; recipes remain immutable in their original store.
mod grants;
mod store;
pub(crate) use grants::{Grant, Recipient, Resource};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub(crate) use store::Store;

#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipeRef {
    pub(crate) recipe_id: String,
    pub(crate) revision: u64,
}
#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Collection {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) entries: Vec<RecipeRef>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Draft {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) description: String,
    pub(crate) entries: Vec<RecipeRef>,
}
fn valid_text(value: &str, max: usize, multiline: bool) -> bool {
    value.len() <= max
        && !value
            .chars()
            .any(|c| c.is_control() && !(multiline && c == '\n'))
}
impl Draft {
    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        let mut seen = BTreeSet::new();
        if self.name.trim().is_empty()
            || !valid_text(&self.name, 200, false)
            || !valid_text(&self.description, 2000, true)
            || self.entries.len() > 200
            || self.entries.iter().any(|r| {
                !crate::recipes::valid_id(&r.recipe_id)
                    || r.revision == 0
                    || !seen.insert(&r.recipe_id)
            })
        {
            return Err("invalid_collection");
        }
        Ok(())
    }
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HomeLibrary {
    pub(crate) revision: u64,
    next_id: u64,
    pub(crate) collections: BTreeMap<String, Collection>,
    #[serde(default)]
    pub(crate) shares: BTreeMap<String, Grant>,
}
impl Default for HomeLibrary {
    fn default() -> Self {
        Self {
            revision: 0,
            next_id: 1,
            collections: BTreeMap::new(),
            shares: BTreeMap::new(),
        }
    }
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema_version: u32,
    homes: BTreeMap<String, HomeLibrary>,
}
impl Document {
    fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != 1 || self.homes.len() > 1000 {
            return Err("invalid_library");
        }
        for (home, data) in &self.homes {
            if !valid_home(home)
                || data.next_id == 0
                || data.collections.len() > 100
                || data.shares.len() > 1000
            {
                return Err("invalid_library");
            }
            let mut grants = BTreeSet::new();
            for (id, share) in &data.shares {
                if !grants.insert((share.resource.object_key(), &share.recipient)) {
                    return Err("invalid_library");
                }
                if &share.id != id
                    || share.version == 0
                    || id
                        .strip_prefix("share-")
                        .and_then(|s| s.parse::<u64>().ok())
                        .is_none_or(|n| n == 0 || n >= data.next_id || id != &format!("share-{n}"))
                {
                    return Err("invalid_library");
                }
                share.resource.validate(data)?;
                share.recipient.validate(home)?;
            }
            for (id, collection) in &data.collections {
                if &collection.id != id
                    || id
                        .strip_prefix("collection-")
                        .and_then(|s| s.parse::<u64>().ok())
                        .is_none_or(|n| {
                            n == 0 || n >= data.next_id || id != &format!("collection-{n}")
                        })
                {
                    return Err("invalid_library");
                }
                Draft {
                    name: collection.name.clone(),
                    description: collection.description.clone(),
                    entries: collection.entries.clone(),
                }
                .validate()?;
            }
        }
        Ok(())
    }
}
fn valid_home(home: &str) -> bool {
    home.strip_prefix("home-")
        .and_then(|s| s.parse::<u64>().ok())
        .is_some_and(|n| n > 0 && home == format!("home-{n}"))
}
