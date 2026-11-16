use super::{Result, dataset, random_id};
use age::secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

pub(super) const MAX_ARCHIVE: u64 = 512 * 1024 * 1024;
const MAX_FILE: u64 = 64 * 1024 * 1024;
const MAX_FILES: usize = 20_000;
pub(super) const MAX_TIMESTAMP: u64 = 253_402_300_799;
#[cfg(not(test))]
const WORK_FACTOR: u8 = 17;
#[cfg(test)]
const WORK_FACTOR: u8 = 2;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Manifest {
    pub schema_version: u32,
    pub kind: String,
    pub engine_version: String,
    pub created_at: u64,
    pub households: usize,
    pub files: usize,
    pub bytes: u64,
}
fn passphrase(value: String) -> Result<SecretString> {
    if !(12..=256).contains(&value.chars().count()) || value.len() > 1024 {
        return Err("device_passphrase_invalid");
    }
    Ok(SecretString::from(value))
}
pub(super) fn transfer(root: &dataset::Root, kind: &str, id: &str) -> Result<PathBuf> {
    if !dataset::valid_id(id) || !matches!(kind, "imports" | "exports") {
        return Err("device_request_invalid");
    }
    Ok(root.path.join(kind).join(format!("{id}.age")))
}
pub(super) fn discard(root: &dataset::Root, kind: &str, id: &str) -> Result<()> {
    let path = transfer(root, kind, id)?;
    if path.exists() {
        fs::remove_file(path).map_err(|_| "device_cleanup_failed")?;
    }
    Ok(())
}
pub(super) fn cleanup_transfers(root: &dataset::Root) -> Result<()> {
    for name in ["imports", "exports"] {
        for file in fs::read_dir(root.path.join(name)).map_err(|_| "device_storage_failed")? {
            let file = file.map_err(|_| "device_storage_failed")?;
            if !file
                .file_type()
                .map_err(|_| "device_storage_failed")?
                .is_file()
            {
                return Err("device_storage_failed");
            }
            fs::remove_file(file.path()).map_err(|_| "device_cleanup_failed")?;
        }
    }
    Ok(())
}
fn data_files(path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = dataset::files(path)?;
    if files.iter().any(|p| {
        p.file_name()
            .is_some_and(|n| n.to_string_lossy().ends_with(".pending"))
    }) {
        return Err("device_recovery_required");
    }
    files.retain(|p| {
        !p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| [".membership.lock", ".household.lock", ".mealshop.lock"].contains(&n))
    });
    if files.len() > MAX_FILES {
        return Err("device_backup_capacity");
    }
    Ok(files)
}
pub(super) fn export(
    root: &dataset::Root,
    active: &str,
    households: usize,
    password: String,
) -> Result<(String, Manifest)> {
    let secret = passphrase(password)?;
    let directory = root.dataset(active)?;
    let files = data_files(&directory)?;
    let sizes = files
        .iter()
        .map(|path| {
            fs::metadata(path)
                .map(|m| m.len())
                .map_err(|_| "device_storage_failed")
        })
        .collect::<Result<Vec<_>>>()?;
    let total = sizes
        .iter()
        .try_fold(0u64, |total, size| total.checked_add(*size))
        .ok_or("device_backup_capacity")?;
    if total > MAX_ARCHIVE || sizes.iter().any(|size| *size > MAX_FILE) {
        return Err("device_backup_capacity");
    }
    let manifest = Manifest {
        schema_version: 1,
        kind: "mealshop-app-only".to_owned(),
        engine_version: env!("CARGO_PKG_VERSION").to_owned(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "device_clock_invalid")?
            .as_secs(),
        households,
        files: files.len(),
        bytes: total,
    };
    if manifest.created_at > MAX_TIMESTAMP {
        return Err("device_clock_invalid");
    }
    let id = random_id()?;
    let output = transfer(root, "exports", &id)?;
    let result = (|| {
        let mut recipient = age::scrypt::Recipient::new(secret);
        recipient.set_work_factor(WORK_FACTOR);
        let encryptor =
            age::Encryptor::with_recipients(std::iter::once(&recipient as &dyn age::Recipient))
                .map_err(|_| "device_backup_failed")?;
        let file = dataset::private_file(&output)?;
        let encrypted = encryptor
            .wrap_output(file)
            .map_err(|_| "device_backup_failed")?;
        let mut archive = tar::Builder::new(encrypted);
        let bytes = serde_json::to_vec(&manifest).map_err(|_| "device_backup_failed")?;
        append(
            &mut archive,
            Path::new("manifest.json"),
            bytes.len() as u64,
            &bytes[..],
        )?;
        for (file, size) in files.iter().zip(sizes) {
            let relative = file
                .strip_prefix(&directory)
                .map_err(|_| "device_backup_failed")?;
            append(
                &mut archive,
                &Path::new("data").join(relative),
                size,
                File::open(file).map_err(|_| "device_storage_failed")?,
            )?;
        }
        let encrypted = archive.into_inner().map_err(|_| "device_backup_failed")?;
        let file = encrypted.finish().map_err(|_| "device_backup_failed")?;
        file.sync_all().map_err(|_| "device_backup_failed")?;
        if file.metadata().map_err(|_| "device_backup_failed")?.len() > MAX_ARCHIVE {
            return Err("device_backup_capacity");
        }
        dataset::sync_directory(&root.path.join("exports"))
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(output);
        return Err(error);
    }
    Ok((id, manifest))
}
fn append<W: Write>(
    archive: &mut tar::Builder<W>,
    path: &Path,
    size: u64,
    reader: impl Read,
) -> Result<()> {
    let mut header = tar::Header::new_ustar();
    header.set_size(size);
    header.set_mode(0o600);
    header.set_entry_type(tar::EntryType::Regular);
    header.set_cksum();
    archive
        .append_data(&mut header, path, reader)
        .map_err(|_| "device_backup_failed")
}

pub(super) fn restore(
    root: &dataset::Root,
    id: &str,
    password: String,
) -> Result<(String, Manifest)> {
    let secret = passphrase(password)?;
    let input = transfer(root, "imports", id)?;
    let metadata = fs::symlink_metadata(&input).map_err(|_| "device_restore_invalid")?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > MAX_ARCHIVE {
        return Err("device_restore_invalid");
    }
    let decryptor = age::Decryptor::new_buffered(std::io::BufReader::new(
        File::open(input).map_err(|_| "device_restore_invalid")?,
    ))
    .map_err(|_| "device_restore_invalid")?;
    if !decryptor.is_scrypt() {
        return Err("device_restore_invalid");
    }
    let mut identity = age::scrypt::Identity::new(secret);
    identity.set_max_work_factor(18);
    let reader = decryptor
        .decrypt(std::iter::once(&identity as &dyn age::Identity))
        .map_err(|_| "device_restore_password")?;
    let candidate = root.new_dataset()?;
    let directory = root.dataset(&candidate)?;
    match extract(reader, &directory) {
        Ok(manifest) => Ok((candidate, manifest)),
        Err(code) => {
            root.remove_dataset(&candidate)?;
            Err(code)
        }
    }
}
fn extract(reader: impl Read, directory: &Path) -> Result<Manifest> {
    // Include tar headers/padding in the decompressed limit; the input is never decompressed.
    let mut archive = tar::Archive::new(reader.take(MAX_ARCHIVE + 1));
    let mut names = BTreeSet::new();
    let mut total = 0u64;
    let mut manifest = None;
    for entry in archive
        .entries()
        .map_err(|_| "device_restore_invalid")?
        .raw(true)
    {
        let mut entry = entry.map_err(|_| "device_restore_invalid")?;
        if !entry.header().entry_type().is_file() {
            return Err("device_restore_invalid");
        }
        let path = entry
            .path()
            .map_err(|_| "device_restore_invalid")?
            .into_owned();
        let name = path.to_str().ok_or("device_restore_invalid")?;
        if name.len() > 240
            || name.contains('\\')
            || path
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
            || !names.insert(path.components().collect::<PathBuf>())
            || names.len() > MAX_FILES + 1
        {
            return Err("device_restore_invalid");
        }
        let size = entry.size();
        if size > MAX_FILE {
            return Err("device_restore_invalid");
        }
        if name == "manifest.json" && manifest.is_none() && names.len() == 1 {
            if size > 4096 {
                return Err("device_restore_invalid");
            }
            let mut bytes = Vec::new();
            entry
                .read_to_end(&mut bytes)
                .map_err(|_| "device_restore_invalid")?;
            let value: Manifest =
                serde_json::from_slice(&bytes).map_err(|_| "device_restore_invalid")?;
            if value.schema_version != 1 || value.kind != "mealshop-app-only" {
                return Err("device_restore_version");
            }
            if value.files > MAX_FILES
                || value.bytes > MAX_ARCHIVE
                || value.engine_version.len() > 80
                || value.created_at > MAX_TIMESTAMP
            {
                return Err("device_restore_invalid");
            }
            manifest = Some(value);
        } else {
            if manifest.is_none() {
                return Err("device_restore_invalid");
            }
            let relative = path
                .strip_prefix("data")
                .map_err(|_| "device_restore_invalid")?;
            if relative.as_os_str().is_empty()
                || relative.components().any(|c| {
                    let s = c.as_os_str().to_string_lossy();
                    s.starts_with('.')
                })
            {
                return Err("device_restore_invalid");
            }
            let destination = directory.join(relative);
            dataset::private_directory(destination.parent().ok_or("device_restore_invalid")?)?;
            let mut file = dataset::private_file(&destination)?;
            let copied =
                std::io::copy(&mut entry, &mut file).map_err(|_| "device_restore_invalid")?;
            if copied != size {
                return Err("device_restore_invalid");
            }
            file.sync_all().map_err(|_| "device_storage_failed")?;
            total = total.checked_add(size).ok_or("device_restore_invalid")?;
            if total > MAX_ARCHIVE {
                return Err("device_restore_invalid");
            }
        }
    }
    authenticate_tail(archive.into_inner())?;
    let manifest = manifest.ok_or("device_restore_invalid")?;
    if names.len() != manifest.files + 1 || total != manifest.bytes {
        return Err("device_restore_invalid");
    }
    // Sync all directory entries before the root pointer can publish this generation.
    sync_tree(directory)?;
    Ok(manifest)
}
fn authenticate_tail(mut tail: std::io::Take<impl Read>) -> Result<()> {
    // tar stops at its end marker. Authenticate the age final chunk as well, including truncation.
    let mut buffer = [0u8; 8192];
    loop {
        let count = tail
            .read(&mut buffer)
            .map_err(|_| "device_restore_invalid")?;
        if count == 0 {
            break;
        }
        if buffer[..count].iter().any(|b| *b != 0) {
            return Err("device_restore_invalid");
        }
    }
    if tail.limit() == 0 {
        return Err("device_restore_invalid");
    }
    Ok(())
}
fn sync_tree(directory: &Path) -> Result<()> {
    for entry in fs::read_dir(directory).map_err(|_| "device_storage_failed")? {
        let entry = entry.map_err(|_| "device_storage_failed")?;
        if entry
            .file_type()
            .map_err(|_| "device_storage_failed")?
            .is_dir()
        {
            sync_tree(&entry.path())?;
        }
    }
    dataset::sync_directory(directory)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
