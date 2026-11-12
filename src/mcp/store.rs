use super::Scope;
use crate::recipes::Recipe;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Grants {
    pub(crate) schema_version: u32,
    pub(crate) revision: u64,
    pub(crate) users: BTreeMap<String, Scope>,
}
impl Grants {
    fn valid(&self) -> bool {
        self.schema_version == 1
            && self.users.len() <= 100
            && self.users.keys().all(|u| crate::membership::valid_user(u))
    }
}
pub(crate) struct Preview {
    pub(crate) actor: String,
    pub(crate) grant_revision: u64,
    pub(crate) ingredient_revision: u64,
    pub(crate) deadline: std::time::Instant,
    pub(crate) recipe: Recipe,
}
pub(crate) struct Store {
    path: PathBuf,
    pub(crate) grants: Grants,
    pub(crate) previews: BTreeMap<String, Preview>,
    writable: bool,
    pub(super) expiry_started: bool,
}
impl Store {
    pub(crate) fn open(directory: &Path) -> Result<Self, String> {
        if directory.join(".mcp.pending").exists() {
            return Err(
                "Interrupted MCP settings write; recover verified settings before startup"
                    .to_owned(),
            );
        }
        let path = directory.join("mcp.json");
        let grants = if path.exists() {
            let mut bytes = Vec::new();
            File::open(&path)
                .map_err(|_| "Cannot read MCP settings")?
                .take(65_537)
                .read_to_end(&mut bytes)
                .map_err(|_| "Cannot read MCP settings")?;
            if bytes.len() > 65_536 {
                return Err("MCP settings exceed 64 KiB".to_owned());
            }
            serde_json::from_slice(&bytes).map_err(|_| "Invalid MCP settings")?
        } else {
            Grants {
                schema_version: 1,
                ..Grants::default()
            }
        };
        if !grants.valid() {
            return Err("Invalid MCP scope grants".to_owned());
        }
        Ok(Self {
            path,
            grants,
            previews: BTreeMap::new(),
            writable: true,
            expiry_started: false,
        })
    }
    pub(crate) fn scope(
        &self,
        actor: &str,
        role: crate::membership::Role,
    ) -> Result<Scope, &'static str> {
        if !self.writable {
            return Err("mcp_storage_failed");
        }
        Ok(self
            .grants
            .users
            .get(actor)
            .copied()
            .unwrap_or_default()
            .effective(role))
    }
    pub(crate) fn configure(
        &mut self,
        expected: u64,
        mut users: BTreeMap<String, Scope>,
    ) -> Result<(), &'static str> {
        if !self.writable {
            return Err("mcp_storage_failed");
        }
        if self.grants.revision != expected {
            return Err("mcp_revision_conflict");
        }
        users.retain(|_, s| *s != Scope::Disabled);
        if users == self.grants.users {
            return Ok(());
        }
        let grants = Grants {
            schema_version: 1,
            revision: expected.checked_add(1).ok_or("mcp_capacity")?,
            users,
        };
        if !grants.valid() {
            return Err("mcp_invalid_grants");
        }
        let bytes = serde_json::to_vec(&grants).map_err(|_| "mcp_storage_failed")?;
        if bytes.len() > 65_536 {
            return Err("mcp_capacity");
        }
        let pending = self.path.with_file_name(".mcp.pending");
        let persist = || -> std::io::Result<()> {
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
            fs::rename(&pending, &self.path)?;
            File::open(
                self.path
                    .parent()
                    .ok_or_else(|| std::io::Error::other("parent"))?,
            )?
            .sync_all()
        };
        if persist().is_err() {
            self.writable = false;
            self.previews.clear();
            return Err("mcp_storage_failed");
        }
        self.grants = grants;
        self.previews.clear();
        Ok(())
    }
    pub(crate) fn preview(&mut self, preview: Preview) -> Result<String, &'static str> {
        self.expire(std::time::Instant::now());
        if self.previews.len() >= 16
            || self
                .previews
                .values()
                .filter(|p| p.actor == preview.actor)
                .count()
                >= 4
        {
            return Err("mcp_preview_capacity");
        }
        let mut bytes = [0u8; 32];
        File::open("/dev/urandom")
            .and_then(|mut f| f.read_exact(&mut bytes))
            .map_err(|_| "mcp_storage_failed")?;
        let mut id = String::from("mcp-");
        for b in bytes {
            write!(&mut id, "{b:02x}").map_err(|_| "mcp_storage_failed")?;
        }
        self.previews.insert(id.clone(), preview);
        Ok(id)
    }
    pub(super) fn expire(&mut self, now: std::time::Instant) {
        self.previews.retain(|_, preview| preview.deadline > now);
    }
}
