use super::{Collection, Document, Draft, HomeLibrary};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub(crate) struct Store {
    root: PathBuf,
    data: Document,
    writable: bool,
}
impl Store {
    pub(crate) fn open(root: &Path) -> Result<Self, String> {
        if root.join(".library.pending").exists() {
            return Err("Interrupted library write; recover before startup".to_owned());
        }
        let path = root.join("library.json");
        let data = if path.exists() {
            let mut bytes = Vec::new();
            File::open(path)
                .and_then(|f| f.take(2_097_153).read_to_end(&mut bytes))
                .map_err(|_| "Cannot read recipe library")?;
            if bytes.len() > 2_097_152 {
                return Err("Recipe library exceeds 2 MiB".to_owned());
            }
            serde_json::from_slice(&bytes).map_err(|_| "Invalid recipe library")?
        } else {
            Document {
                schema_version: 1,
                homes: BTreeMap::new(),
            }
        };
        data.validate().map_err(str::to_owned)?;
        Ok(Self {
            root: root.to_owned(),
            data,
            writable: true,
        })
    }
    pub(crate) fn home(&self, home: &str) -> HomeLibrary {
        self.data.homes.get(home).cloned().unwrap_or_default()
    }
    pub(crate) fn homes(&self) -> &BTreeMap<String, HomeLibrary> {
        &self.data.homes
    }
    pub(super) fn commit<T>(
        &mut self,
        home: &str,
        expected: u64,
        action: impl FnOnce(&mut HomeLibrary) -> Result<T, &'static str>,
    ) -> Result<T, &'static str> {
        if !self.writable {
            return Err("library_storage_failed");
        }
        let mut document = self.data.clone();
        let target = document.homes.entry(home.to_owned()).or_default();
        if target.revision != expected {
            return Err("revision_conflict");
        }
        let result = action(target)?;
        target.revision = target.revision.checked_add(1).ok_or("library_capacity")?;
        document.validate()?;
        let bytes = serde_json::to_vec(&document).map_err(|_| "library_storage_failed")?;
        if bytes.len() > 2_097_152 {
            return Err("library_capacity");
        }
        let persist = || -> std::io::Result<()> {
            let pending = self.root.join(".library.pending");
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
            fs::rename(pending, self.root.join("library.json"))?;
            #[cfg(unix)]
            File::open(&self.root)?.sync_all()?;
            Ok(())
        };
        if persist().is_err() {
            self.writable = false;
            return Err("library_storage_failed");
        }
        self.data = document;
        Ok(result)
    }
    pub(crate) fn save(
        &mut self,
        home: &str,
        expected: u64,
        id: Option<&str>,
        draft: Draft,
    ) -> Result<String, &'static str> {
        draft.validate()?;
        self.commit(home, expected, |data| {
            let id = if let Some(id) = id {
                if !data.collections.contains_key(id) {
                    return Err("collection_not_found");
                }
                id.to_owned()
            } else {
                if data.collections.len() >= 100 {
                    return Err("library_capacity");
                }
                let id = format!("collection-{}", data.next_id);
                data.next_id = data.next_id.checked_add(1).ok_or("library_capacity")?;
                id
            };
            data.collections.insert(
                id.clone(),
                Collection {
                    id: id.clone(),
                    name: draft.name,
                    description: draft.description,
                    entries: draft.entries,
                },
            );
            Ok(id)
        })
    }
    pub(crate) fn remove(
        &mut self,
        home: &str,
        expected: u64,
        id: &str,
    ) -> Result<(), &'static str> {
        self.commit(home, expected, |data| {
            data.collections.remove(id).ok_or("collection_not_found")?;
            data.shares.retain(|_, share| {
                !matches!(&share.resource, super::Resource::Collection {collection_id} if collection_id == id)
            });
            Ok(())
        })
    }
}
