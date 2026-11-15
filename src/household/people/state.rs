use super::super::{Profiles, Store};
use crate::membership::Role;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::Write,
};

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Permission {
    Read,
    Edit,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Grant {
    pub permission: Permission,
    pub epoch: u64,
}
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Entry {
    pub date: String,
    pub kind: Kind,
    pub duration_seconds: u32,
    pub intensity: Option<Intensity>,
    #[serde(default)]
    pub notes: String,
}
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Kind {
    Walking,
    Running,
    Cycling,
    Swimming,
    Strength,
    Other,
}
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Intensity {
    Easy,
    Moderate,
    Hard,
}
impl Entry {
    pub fn valid(&self) -> bool {
        self.date.len() == 10
            && self.date.parse::<jiff::civil::Date>().is_ok()
            && (1..=604_800).contains(&self.duration_seconds)
            && self.notes.len() <= 1000
            && !self.notes.chars().any(|c| c.is_control() && c != '\n')
    }
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Person {
    pub generation: u64,
    pub access_revision: u64,
    pub revision: u64,
    pub grants: BTreeMap<String, Grant>,
    pub entries: BTreeMap<String, Entry>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Private {
    next_generation: u64,
    pub people: BTreeMap<String, Person>,
}
impl Default for Private {
    fn default() -> Self {
        Self {
            next_generation: 1,
            people: BTreeMap::new(),
        }
    }
}
impl Private {
    pub fn empty(&self) -> bool {
        self.people.is_empty() && self.next_generation == 1
    }
    pub fn create(&mut self, id: &str) -> Result<&mut Person, &'static str> {
        if !self.people.contains_key(id) {
            let generation = self.next_generation;
            self.next_generation = generation
                .checked_add(1)
                .ok_or("person_activity_capacity")?;
            self.people.insert(
                id.to_owned(),
                Person {
                    generation,
                    access_revision: 0,
                    revision: 0,
                    grants: BTreeMap::new(),
                    entries: BTreeMap::new(),
                },
            );
        }
        self.people
            .get_mut(id)
            .ok_or("person_activity_storage_failed")
    }
    pub fn valid(&self, public: &Profiles) -> bool {
        let ids: BTreeSet<_> = public.members.iter().map(|m| m.id.clone()).collect();
        let mut generations = BTreeSet::new();
        self.next_generation > 0
            && self.people.len() <= 100
            && self.people.iter().all(|(id, p)| {
                ids.contains(id.as_str())
                    && p.generation > 0
                    && p.generation < self.next_generation
                    && generations.insert(p.generation)
                    && p.grants.len() <= 100
                    && p.grants.keys().all(|u| crate::membership::valid_user(u))
                    && p.entries.len() <= 1000
                    && p.entries.iter().all(|(id, e)| entry_id(id) && e.valid())
            })
    }
}
pub(crate) fn entry_id(id: &str) -> bool {
    id.strip_prefix("activity-").is_some_and(|s| {
        s.len() == 48
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::household) struct Document {
    schema_version: u32,
    revision: u64,
    members: Vec<super::super::Member>,
    #[serde(default, skip_serializing_if = "Private::empty")]
    private_people: Private,
}
impl Document {
    pub fn decode(bytes: &[u8]) -> Result<(Profiles, Private), String> {
        let doc: Self = serde_json::from_slice(bytes).map_err(|_| "Invalid household data")?;
        let profiles = Profiles {
            schema_version: doc.schema_version,
            revision: doc.revision,
            members: doc.members,
        };
        if profiles.schema_version != 1 || !doc.private_people.valid(&profiles) {
            return Err("Invalid person activity state".to_owned());
        }
        Ok((profiles, doc.private_people))
    }
}
impl Store {
    pub(crate) fn publish(&mut self, data: Profiles, private: Private) -> Result<(), &'static str> {
        if !self.writable {
            return Err("household_storage_failed");
        }
        if !private.valid(&data) {
            return Err("invalid_person_activity");
        }
        let doc = Document {
            schema_version: data.schema_version,
            revision: data.revision,
            members: data.members.clone(),
            private_people: private.clone(),
        };
        let bytes = serde_json::to_vec(&doc).map_err(|_| "household_storage_failed")?;
        if bytes.len() > 4_194_304 {
            return Err("person_activity_capacity");
        }
        let pending = self.path.with_file_name(".pending");
        let persist = || -> Result<(), std::io::Error> {
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&pending)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            fs::rename(&pending, &self.path)?;
            #[cfg(unix)]
            File::open(
                self.path
                    .parent()
                    .ok_or_else(|| std::io::Error::other("Missing parent"))?,
            )?
            .sync_all()?;
            Ok(())
        };
        if persist().is_err() {
            self.writable = false;
            return Err("household_storage_failed");
        }
        self.data = data;
        self.private = private;
        Ok(())
    }
    pub(crate) fn replace_profiles(
        &mut self,
        mut members: Vec<super::super::Member>,
        expected: u64,
        role: Role,
    ) -> Result<(), &'static str> {
        if expected != self.data.revision {
            return Err("revision_conflict");
        }
        let ids: BTreeSet<_> = members.iter().map(|m| m.id.clone()).collect();
        if role != Role::Owner
            && self
                .private
                .people
                .keys()
                .any(|id| !ids.contains(id.as_str()))
        {
            return Err("person_owner_required");
        }
        for member in &mut members {
            if member.avatar.is_none() {
                member.avatar = self
                    .data
                    .members
                    .iter()
                    .find(|m| m.id == member.id)
                    .and_then(|m| m.avatar.clone());
            }
            if member.rules.is_none() {
                member.rules = self
                    .data
                    .members
                    .iter()
                    .find(|m| m.id == member.id)
                    .and_then(|m| m.rules.clone());
            }
        }
        let mut private = self.private.clone();
        private.people.retain(|id, _| ids.contains(id.as_str()));
        let data = Profiles {
            schema_version: 1,
            revision: self
                .data
                .revision
                .checked_add(1)
                .ok_or("household_revision_limit")?,
            members,
        };
        self.publish(data, private)
    }
}
