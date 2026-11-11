use super::{HomeLibrary, RecipeRef, Store, valid_home};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Resource {
    Recipe { recipe_id: String, revision: u64 },
    Collection { collection_id: String },
}
impl Resource {
    pub(crate) fn reference(&self) -> Option<RecipeRef> {
        match self {
            Self::Recipe {
                recipe_id,
                revision,
            } => Some(RecipeRef {
                recipe_id: recipe_id.clone(),
                revision: *revision,
            }),
            Self::Collection { .. } => None,
        }
    }
    fn same_object(&self, other: &Self) -> bool {
        self.object_key() == other.object_key()
    }
    pub(super) fn object_key(&self) -> (&str, &str) {
        match self {
            Self::Recipe { recipe_id, .. } => ("recipe", recipe_id),
            Self::Collection { collection_id } => ("collection", collection_id),
        }
    }
    pub(super) fn validate(&self, home: &HomeLibrary) -> Result<(), &'static str> {
        match self {
            Self::Recipe {
                recipe_id,
                revision,
            } if crate::recipes::valid_id(recipe_id) && *revision > 0 => Ok(()),
            Self::Collection { collection_id } if home.collections.contains_key(collection_id) => {
                Ok(())
            }
            _ => Err("invalid_share_resource"),
        }
    }
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Recipient {
    Account { user: String },
    HouseholdInvitation { user: String },
    Household { home_id: String },
}
impl Recipient {
    pub(super) fn validate(&self, source_home: &str) -> Result<(), &'static str> {
        let valid = match self {
            Self::Account { user } | Self::HouseholdInvitation { user } => {
                crate::membership::valid_user(user)
            }
            Self::Household { home_id } => valid_home(home_id) && home_id != source_home,
        };
        if valid {
            Ok(())
        } else {
            Err("invalid_share_recipient")
        }
    }
}
#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Grant {
    pub(crate) id: String,
    pub(crate) version: u64,
    pub(crate) resource: Resource,
    pub(crate) recipient: Recipient,
}
impl Store {
    pub(crate) fn grant(
        &mut self,
        home: &str,
        expected: u64,
        resource: Resource,
        recipient: Recipient,
    ) -> Result<String, &'static str> {
        recipient.validate(home)?;
        self.commit(home, expected, |data| {
            resource.validate(data)?;
            if data
                .shares
                .values()
                .any(|s| s.resource.same_object(&resource) && s.recipient == recipient)
            {
                return Err("share_exists");
            }
            if data.shares.len() >= 1000 {
                return Err("library_capacity");
            }
            let id = format!("share-{}", data.next_id);
            data.next_id = data.next_id.checked_add(1).ok_or("library_capacity")?;
            data.shares.insert(
                id.clone(),
                Grant {
                    id: id.clone(),
                    version: 1,
                    resource,
                    recipient,
                },
            );
            Ok(id)
        })
    }
    pub(crate) fn publish(
        &mut self,
        home: &str,
        expected: u64,
        id: &str,
        resource: Resource,
    ) -> Result<(), &'static str> {
        self.commit(home, expected, |data| {
            resource.validate(data)?;
            let share = data.shares.get_mut(id).ok_or("share_not_found")?;
            if !share.resource.same_object(&resource) || resource.reference().is_none() {
                return Err("invalid_share_resource");
            }
            share.resource = resource;
            share.version = share.version.checked_add(1).ok_or("library_capacity")?;
            Ok(())
        })
    }
    pub(crate) fn revoke(
        &mut self,
        home: &str,
        expected: u64,
        id: &str,
    ) -> Result<(), &'static str> {
        self.commit(home, expected, |data| {
            data.shares.remove(id).map(|_| ()).ok_or("share_not_found")
        })
    }
    pub(crate) fn accept(
        &mut self,
        home: &str,
        id: &str,
        version: u64,
        destination: String,
    ) -> Result<(), &'static str> {
        let expected = self.home(home).revision;
        let recipient = Recipient::Household {
            home_id: destination,
        };
        recipient.validate(home)?;
        self.commit(home, expected, |data| {
            let mut share = data.shares.get(id).cloned().ok_or("share_not_found")?;
            if share.version != version {
                return Err("revision_conflict");
            }
            if !matches!(share.recipient, Recipient::HouseholdInvitation { .. }) {
                return Err("invalid_share_recipient");
            }
            if data
                .shares
                .values()
                .any(|s| s.recipient == recipient && s.resource.same_object(&share.resource))
            {
                return Err("share_exists");
            }
            share.recipient = recipient;
            share.version = share.version.checked_add(1).ok_or("library_capacity")?;
            data.shares.insert(id.to_owned(), share);
            Ok(())
        })
    }
}
