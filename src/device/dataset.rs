use super::{Result, random_id};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Format {
    schema_version: u32,
    kind: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Active {
    schema_version: u32,
    id: String,
}

pub(super) struct Root {
    pub path: PathBuf,
    // The OS lock is never unlinked; closing the file releases lifetime ownership after a crash.
    _owner: File,
}
impl Root {
    pub fn open(path: &Path) -> Result<Self> {
        private_directory(path)?;
        let entries = fs::read_dir(path)
            .map_err(|_| "device_storage_failed")?
            .map(|entry| entry.map(|e| e.file_name()))
            .collect::<std::io::Result<Vec<_>>>()
            .map_err(|_| "device_storage_failed")?;
        let new = !path.join("format.json").exists()
            && entries
                .iter()
                .all(|name| name == ".mealshop-device.owner" || name == ".format.json.pending");
        if !new {
            let format: Format = read_json(&path.join("format.json"))?;
            if format.schema_version != 1 || format.kind != "mealshop-app-only" {
                return Err("device_format_invalid");
            }
        }
        let lock = path.join(".mealshop-device.owner");
        if fs::symlink_metadata(&lock).is_ok_and(|m| !m.is_file() || m.file_type().is_symlink()) {
            return Err("device_storage_failed");
        }
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let owner = options.open(lock).map_err(|_| "device_storage_failed")?;
        owner.try_lock().map_err(|_| "device_in_use")?;
        let root = Self {
            path: path.to_owned(),
            _owner: owner,
        };
        if new {
            atomic_json(
                &root.path,
                "format.json",
                &Format {
                    schema_version: 1,
                    kind: "mealshop-app-only".to_owned(),
                },
            )?;
        }
        for name in ["datasets", "imports", "exports"] {
            private_directory(&root.path.join(name))?;
        }
        Ok(root)
    }

    pub fn active(&self) -> Result<String> {
        let path = self.path.join("active.json");
        if !path.exists() {
            let entries = fs::read_dir(self.path.join("datasets"))
                .map_err(|_| "device_storage_failed")?
                .map(|entry| entry.map(|e| e.file_name()))
                .collect::<std::io::Result<Vec<_>>>()
                .map_err(|_| "device_storage_failed")?;
            let id = if entries.is_empty() {
                self.new_dataset()?
            } else if entries.len() == 1 {
                let id = entries[0].to_str().ok_or("device_recovery_required")?;
                if fs::read_dir(self.dataset(id)?)
                    .map_err(|_| "device_storage_failed")?
                    .next()
                    .is_some()
                {
                    return Err("device_recovery_required");
                }
                id.to_owned()
            } else {
                return Err("device_recovery_required");
            };
            self.publish(&id)?;
            return Ok(id);
        }
        let active: Active = read_json(&path)?;
        if active.schema_version != 1
            || !valid_id(&active.id)
            || !self.dataset(&active.id)?.is_dir()
        {
            return Err("device_recovery_required");
        }
        Ok(active.id)
    }

    pub fn dataset(&self, id: &str) -> Result<PathBuf> {
        if !valid_id(id) {
            return Err("device_storage_failed");
        }
        let path = self.path.join("datasets").join(id);
        if let Ok(metadata) = fs::symlink_metadata(&path)
            && (!metadata.is_dir() || metadata.file_type().is_symlink())
        {
            return Err("device_storage_failed");
        }
        Ok(path)
    }
    pub fn new_dataset(&self) -> Result<String> {
        let id = random_id()?;
        let path = self.dataset(&id)?;
        fs::create_dir(&path).map_err(|_| "device_storage_failed")?;
        set_private_directory(&path)?;
        sync_directory(&self.path.join("datasets"))?;
        Ok(id)
    }
    pub fn publish(&self, id: &str) -> Result<()> {
        if !self.dataset(id)?.is_dir() {
            return Err("device_storage_failed");
        }
        atomic_json(
            &self.path,
            "active.json",
            &Active {
                schema_version: 1,
                id: id.to_owned(),
            },
        )
    }
    pub fn remove_dataset(&self, id: &str) -> Result<()> {
        let path = self.dataset(id)?;
        let metadata = fs::symlink_metadata(&path).map_err(|_| "device_storage_failed")?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err("device_storage_failed");
        }
        fs::remove_dir_all(path).map_err(|_| "device_cleanup_failed")
    }
    pub fn cleanup_inactive(&self, active: &str) -> Result<()> {
        for entry in
            fs::read_dir(self.path.join("datasets")).map_err(|_| "device_storage_failed")?
        {
            let entry = entry.map_err(|_| "device_storage_failed")?;
            let id = entry
                .file_name()
                .into_string()
                .map_err(|_| "device_storage_failed")?;
            if !valid_id(&id) {
                return Err("device_storage_failed");
            }
            if id != active {
                self.remove_dataset(&id)?;
            }
        }
        Ok(())
    }
}

pub(super) fn valid_id(id: &str) -> bool {
    id.len() == 32
        && id
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

pub(super) fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "device_storage_failed")?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("device_storage_failed");
    }
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|f| f.take(4097).read_to_end(&mut bytes))
        .map_err(|_| "device_storage_failed")?;
    if bytes.len() > 4096 {
        return Err("device_storage_failed");
    }
    serde_json::from_slice(&bytes).map_err(|_| "device_recovery_required")
}
pub(super) fn private_directory(path: &Path) -> Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path).map_err(|_| "device_storage_failed")?;
    if fs::symlink_metadata(path)
        .map_err(|_| "device_storage_failed")?
        .file_type()
        .is_symlink()
    {
        return Err("device_storage_failed");
    }
    Ok(())
}
fn set_private_directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|_| "device_storage_failed")?;
    }
    Ok(())
}
pub(super) fn private_file(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).map_err(|_| "device_storage_failed")
}
pub(super) fn sync_directory(path: &Path) -> Result<()> {
    File::open(path)
        .and_then(|f| f.sync_all())
        .map_err(|_| "device_storage_failed")
}
pub(super) fn atomic_json<T: Serialize>(root: &Path, name: &str, value: &T) -> Result<()> {
    let pending = root.join(format!(".{name}.pending"));
    if pending.exists() {
        // Root ownership is held; the canonical control file remains the publication authority.
        fs::remove_file(&pending).map_err(|_| "device_storage_failed")?;
    }
    let mut file = private_file(&pending)?;
    serde_json::to_writer(&mut file, value).map_err(|_| "device_storage_failed")?;
    file.flush()
        .and_then(|()| file.sync_all())
        .map_err(|_| "device_storage_failed")?;
    fs::rename(pending, root.join(name)).map_err(|_| "device_storage_failed")?;
    sync_directory(root)
}

/// Reclaim only this device owner's stale markers, after proving no pending data transaction exists.
pub(super) fn recover_markers(path: &Path) -> Result<()> {
    let files = files(path)?;
    if files.iter().any(|p| {
        p.file_name()
            .is_some_and(|n| n.to_string_lossy().ends_with(".pending"))
    }) {
        return Err("device_recovery_required");
    }
    for file in files {
        if file
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| [".membership.lock", ".household.lock", ".mealshop.lock"].contains(&n))
        {
            fs::remove_file(file).map_err(|_| "device_storage_failed")?;
        }
    }
    Ok(())
}
pub(super) fn files(root: &Path) -> Result<Vec<PathBuf>> {
    let metadata = fs::symlink_metadata(root).map_err(|_| "device_storage_failed")?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("device_storage_failed");
    }
    let mut pending = vec![root.to_owned()];
    let mut found = Vec::new();
    let mut visited = 0usize;
    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(path).map_err(|_| "device_storage_failed")? {
            let entry = entry.map_err(|_| "device_storage_failed")?;
            visited += 1;
            let kind = entry.file_type().map_err(|_| "device_storage_failed")?;
            if kind.is_symlink() {
                return Err("device_storage_failed");
            }
            if kind.is_dir() {
                pending.push(entry.path());
            } else if kind.is_file() {
                found.push(entry.path());
            } else {
                return Err("device_storage_failed");
            }
            if visited > 2_000_000 {
                return Err("device_capacity");
            }
        }
    }
    found.sort();
    Ok(found)
}
