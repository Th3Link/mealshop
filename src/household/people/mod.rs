//! Person-specific rules and protected activity records.
mod rules;
pub(crate) use rules::{Rules, diet_review, planning_conflicts};

mod state;
pub(in crate::household) use state::{Document, Private};

mod activity;
pub(crate) use activity::{activity_path, routes};

#[cfg(feature = "device")]
pub(crate) fn check_backup_read(
    path: &std::path::Path,
    readable: &std::collections::BTreeSet<String>,
) -> Result<(), &'static str> {
    use std::io::Read;
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err("device_storage_failed"),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 4_194_304 {
        return Err("device_recovery_required");
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .and_then(|file| file.take(4_194_305).read_to_end(&mut bytes))
        .map_err(|_| "device_storage_failed")?;
    let (_, private) = state::Document::decode(&bytes).map_err(|_| "device_recovery_required")?;
    if private
        .people
        .iter()
        .any(|(id, person)| !person.entries.is_empty() && !readable.contains(id))
    {
        return Err("device_backup_access_required");
    }
    Ok(())
}

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
