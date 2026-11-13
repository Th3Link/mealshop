//! Durable, addressed household invitations and current-role authorization.
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Role {
    Owner,
    Editor,
    Reader,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Home {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) members: BTreeMap<String, Role>,
    pub(crate) invitations: BTreeMap<String, Role>,
}
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    version: u32,
    next_id: u64,
    homes: Vec<Home>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    activity_epochs: BTreeMap<String, BTreeMap<String, u64>>,
}
pub(crate) fn valid_user(user: &str) -> bool {
    !user.is_empty() && user.len() <= 255 && !user.chars().any(char::is_control)
}
impl Registry {
    fn validate(&self) -> Result<(), String> {
        let mut ids = BTreeSet::new();
        if self.version != 1 || self.homes.len() > 1000 || self.next_id == 0 {
            return Err("Invalid membership registry version or capacity".to_owned());
        }
        if self.activity_epochs.iter().any(|(home, epochs)| {
            !self.homes.iter().any(|h| &h.id == home)
                || epochs.len() > 10000
                || epochs
                    .iter()
                    .any(|(user, epoch)| !valid_user(user) || *epoch == 0)
        }) {
            return Err("Invalid activity membership epochs".to_owned());
        }
        for home in &self.homes {
            let number = home
                .id
                .strip_prefix("home-")
                .and_then(|s| s.parse::<u64>().ok());
            if !ids.insert(&home.id)
                || number
                    .is_none_or(|n| n == 0 || n >= self.next_id || home.id != format!("home-{n}"))
                || home.name.trim().is_empty()
                || home.name.len() > 200
                || home.name.chars().any(char::is_control)
                || home.members.len() > 100
                || home.invitations.len() > 100
                || home
                    .members
                    .values()
                    .filter(|role| **role == Role::Owner)
                    .count()
                    != 1
                || home.members.keys().any(|user| !valid_user(user))
                || home.invitations.iter().any(|(user, role)| {
                    !valid_user(user) || *role == Role::Owner || home.members.contains_key(user)
                })
            {
                return Err(
                    "Invalid membership invariants; restore a verified membership backup"
                        .to_owned(),
                );
            }
        }
        Ok(())
    }
}
pub(crate) struct Membership {
    root: PathBuf,
    data: Registry,
    writable: bool,
}
impl Membership {
    pub(crate) fn owner(&self, id: &str) -> Option<&str> {
        self.data
            .homes
            .iter()
            .find(|h| h.id == id)?
            .members
            .iter()
            .find(|(_, role)| **role == Role::Owner)
            .map(|(user, _)| user.as_str())
    }
    pub(crate) fn name(&self, id: &str) -> Option<&str> {
        self.data
            .homes
            .iter()
            .find(|h| h.id == id)
            .map(|h| h.name.as_str())
    }
    pub(crate) fn household_ids(&self) -> Vec<String> {
        self.data.homes.iter().map(|h| h.id.clone()).collect()
    }

    pub(crate) fn open(root: &Path) -> Result<Self, String> {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder
            .create(root)
            .map_err(|_| "Cannot create membership directory")?;
        let lock = root.join(".membership.lock");
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&lock)
            .map_err(
                |_| "Membership directory locked; stop writers before recovering a stale lock",
            )?;
        let result = (|| {
            file.sync_all().map_err(|_| "Cannot sync membership lock")?;
            if root.join(".membership.pending").exists() {
                return Err("Interrupted membership write; recover before startup".to_owned());
            }
            let path = root.join("membership.json");
            let data = if path.exists() {
                let mut bytes = Vec::new();
                File::open(path)
                    .map_err(|_| "Cannot read membership")?
                    .take(1_048_577)
                    .read_to_end(&mut bytes)
                    .map_err(|_| "Cannot read membership")?;
                if bytes.len() > 1_048_576 {
                    return Err("Membership exceeds 1 MiB".to_owned());
                }
                serde_json::from_slice::<Registry>(&bytes)
                    .map_err(|_| "Invalid membership registry")?
            } else {
                Registry {
                    version: 1,
                    next_id: 1,
                    homes: vec![],
                    activity_epochs: BTreeMap::new(),
                }
            };
            data.validate()?;
            Ok(Self {
                root: root.to_owned(),
                data,
                writable: true,
            })
        })();
        if result.is_err() {
            let _ = fs::remove_file(lock);
        }
        result
    }
    pub(crate) fn list(&self, user: &str) -> serde_json::Value {
        let homes: Vec<_> = self
            .data
            .homes
            .iter()
            .filter_map(|h| {
                h.members
                    .get(user)
                    .map(|role| serde_json::json!({"id":h.id,"name":h.name,"role":role}))
            })
            .collect();
        let invitations: Vec<_> = self
            .data
            .homes
            .iter()
            .filter_map(|h| {
                h.invitations
                    .get(user)
                    .map(|role| serde_json::json!({"id":h.id,"name":h.name,"role":role}))
            })
            .collect();
        serde_json::json!({"households":homes,"invitations":invitations,"user":user})
    }
    pub(crate) fn activity_epochs(&self, id: &str) -> BTreeMap<String, u64> {
        self.data
            .activity_epochs
            .get(id)
            .cloned()
            .unwrap_or_default()
    }
    pub(crate) fn members(&self, id: &str) -> Option<BTreeMap<String, Role>> {
        Some(
            self.data
                .homes
                .iter()
                .find(|home| home.id == id)?
                .members
                .clone(),
        )
    }
    pub(crate) fn role(&self, id: &str, user: &str) -> Option<Role> {
        self.data
            .homes
            .iter()
            .find(|h| h.id == id)?
            .members
            .get(user)
            .copied()
    }
    pub(crate) fn detail(&self, id: &str, user: &str) -> Result<serde_json::Value, &'static str> {
        let h = self
            .data
            .homes
            .iter()
            .find(|h| h.id == id && h.members.contains_key(user))
            .ok_or("household_access_denied")?;
        let mut value = serde_json::json!({"id":h.id,"name":h.name,"members":h.members});
        if h.members.get(user) == Some(&Role::Owner) {
            value["invitations"] = serde_json::json!(h.invitations);
        }
        Ok(value)
    }
    fn commit(&mut self, data: Registry) -> Result<(), &'static str> {
        if !self.writable {
            return Err("membership_storage_failed");
        }
        data.validate().map_err(|_| "membership_storage_failed")?;
        let bytes = serde_json::to_vec(&data).map_err(|_| "membership_storage_failed")?;
        if bytes.len() > 1_048_576 {
            return Err("membership_capacity");
        }
        let persist = || -> std::io::Result<()> {
            let pending = self.root.join(".membership.pending");
            let mut options = OpenOptions::new();
            options.create_new(true).write(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&pending)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            fs::rename(pending, self.root.join("membership.json"))?;
            #[cfg(unix)]
            File::open(&self.root)?.sync_all()?;
            Ok(())
        };
        if persist().is_err() {
            self.writable = false;
            return Err("membership_storage_failed");
        }
        self.data = data;
        Ok(())
    }
    pub(crate) fn create(&mut self, user: &str, name: &str) -> Result<String, &'static str> {
        if name.trim().is_empty() || name.len() > 200 || name.chars().any(char::is_control) {
            return Err("invalid_household_name");
        }
        if self.data.homes.len() >= 1000
            || self
                .data
                .homes
                .iter()
                .filter(|h| h.members.contains_key(user))
                .count()
                >= 20
        {
            return Err("membership_capacity");
        }
        let mut data = self.data.clone();
        let id = format!("home-{}", data.next_id);
        data.next_id = data.next_id.checked_add(1).ok_or("membership_capacity")?;
        data.homes.push(Home {
            id: id.clone(),
            name: name.trim().to_owned(),
            members: BTreeMap::from([(user.to_owned(), Role::Owner)]),
            invitations: BTreeMap::new(),
        });
        self.commit(data)?;
        Ok(id)
    }
    pub(crate) fn invite(
        &mut self,
        id: &str,
        actor: &str,
        user: &str,
        role: Role,
    ) -> Result<(), &'static str> {
        if !valid_user(user) || role == Role::Owner {
            return Err("invalid_invitation");
        }
        let mut data = self.data.clone();
        let h = data
            .homes
            .iter_mut()
            .find(|h| h.id == id && h.members.get(actor) == Some(&Role::Owner))
            .ok_or("household_access_denied")?;
        if h.members.contains_key(user) {
            return Err("already_member");
        }
        if h.invitations.len() >= 100 || h.members.len() >= 100 {
            return Err("membership_capacity");
        }
        h.invitations.insert(user.to_owned(), role);
        self.commit(data)
    }
    pub(crate) fn accept(
        &mut self,
        id: &str,
        user: &str,
        accept: bool,
    ) -> Result<(), &'static str> {
        let mut data = self.data.clone();
        let h = data
            .homes
            .iter_mut()
            .find(|h| h.id == id)
            .ok_or("household_access_denied")?;
        let role = h
            .invitations
            .remove(user)
            .ok_or("household_access_denied")?;
        if accept {
            if h.members.len() >= 100 {
                return Err("membership_capacity");
            }
            h.members.insert(user.to_owned(), role);
        }
        self.commit(data)
    }
    pub(crate) fn change(
        &mut self,
        id: &str,
        actor: &str,
        user: &str,
        role: Option<Role>,
    ) -> Result<(), &'static str> {
        let mut data = self.data.clone();
        let h = data
            .homes
            .iter_mut()
            .find(|h| h.id == id)
            .ok_or("household_access_denied")?;
        if h.members.get(actor) != Some(&Role::Owner)
            && !(actor == user && role.is_none() && h.members.contains_key(actor))
        {
            return Err("household_access_denied");
        }
        if h.members.get(user) == Some(&Role::Owner) {
            return Err("owner_transfer_required");
        }
        if let Some(role) = role {
            if !h.members.contains_key(user) {
                return Err("household_access_denied");
            }
            if role == Role::Owner {
                h.members.insert(actor.to_owned(), Role::Editor);
            }
            h.members.insert(user.to_owned(), role);
        } else {
            h.members.remove(user);
            h.invitations.remove(user);
            let epochs = data.activity_epochs.entry(id.to_owned()).or_default();
            let epoch = epochs
                .get(user)
                .copied()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or("membership_capacity")?;
            epochs.insert(user.to_owned(), epoch);
        }
        self.commit(data)
    }
}
impl Drop for Membership {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.root.join(".membership.lock"));
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn membership_survives_restart_and_rejects_corruption_and_second_writers() {
        let root = std::env::temp_dir().join(format!("mealshop-membership-{}", std::process::id()));
        let mut store = Membership::open(&root).unwrap();
        let id = store.create("alice", "Family").unwrap();
        assert!(Membership::open(&root).is_err());
        assert!(root.join(".membership.lock").is_file());
        store.invite(&id, "alice", "bob", Role::Editor).unwrap();
        assert!(store.accept(&id, "mallory", true).is_err());
        store.accept(&id, "bob", true).unwrap();
        assert!(
            store
                .change(&id, "bob", "alice", Some(Role::Reader))
                .is_err()
        );
        drop(store);
        let mut store = Membership::open(&root).unwrap();
        assert!(store.role(&id, "bob") == Some(Role::Editor));
        store
            .change(&id, "alice", "bob", Some(Role::Owner))
            .unwrap();
        assert!(store.role(&id, "alice") == Some(Role::Editor));
        store.change(&id, "alice", "alice", None).unwrap();
        assert!(store.role(&id, "alice").is_none());
        drop(store);
        let path = root.join("membership.json");
        let bytes = fs::read(&path).unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        value["homes"][0]["id"] = serde_json::json!("../escaped");
        let corrupt = serde_json::to_vec(&value).unwrap();
        fs::write(&path, &corrupt).unwrap();
        assert!(Membership::open(&root).is_err());
        assert_eq!(fs::read(&path).unwrap(), corrupt);
        assert!(!root.join(".membership.lock").exists());
        fs::remove_dir_all(root).unwrap();
    }
}
