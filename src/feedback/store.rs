use super::{Data, Entry, Input, Policy, Recipe, Source};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub(crate) struct Draft {
    pub(crate) source: Source,
    pub(crate) recipe: Recipe,
    pub(crate) servings: u32,
    pub(crate) input: Input,
}
pub(crate) struct Store {
    path: PathBuf,
    pub(crate) data: Data,
    writable: bool,
}
impl Store {
    pub(crate) fn open(directory: &Path) -> Result<Self, String> {
        if directory.join(".history.pending").exists() {
            return Err(
                "Interrupted cooking-history write; recover verified history before startup"
                    .to_owned(),
            );
        }
        let path = directory.join("history.json");
        let data = if path.exists() {
            let mut bytes = Vec::new();
            File::open(&path)
                .and_then(|f| f.take(4_194_305).read_to_end(&mut bytes))
                .map_err(|_| "Cannot read cooking history")?;
            if bytes.len() > 4_194_304 {
                return Err("Cooking history exceeds 4 MiB".to_owned());
            }
            serde_json::from_slice(&bytes)
                .map_err(|_| "Invalid cooking history; restore verified history")?
        } else {
            Data::default()
        };
        data.validate().map_err(
            |_| "Invalid cooking history or preference aggregates; restore verified history",
        )?;
        Ok(Self {
            path,
            data,
            writable: true,
        })
    }
    fn change<T>(
        &mut self,
        expected: u64,
        action: impl FnOnce(&mut Data) -> Result<T, &'static str>,
    ) -> Result<T, &'static str> {
        if !self.writable {
            return Err("history_storage_failed");
        }
        if self.data.revision != expected {
            return Err("history_revision_conflict");
        }
        let mut next = self.data.clone();
        let result = action(&mut next)?;
        next.revision = next.revision.checked_add(1).ok_or("history_capacity")?;
        next.validate()?;
        let bytes = serde_json::to_vec(&next).map_err(|_| "history_storage_failed")?;
        if bytes.len() > 4_194_304 {
            return Err("history_capacity");
        }
        let pending = self.path.with_file_name(".history.pending");
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
            return Err("history_storage_failed");
        }
        self.data = next;
        Ok(result)
    }
    pub(crate) fn save(
        &mut self,
        expected: u64,
        id: Option<&str>,
        draft: Draft,
        actor: &str,
        now: i64,
    ) -> Result<String, &'static str> {
        draft.input.validate()?;
        self.change(expected, |data| {
            let previous = id
                .map(|id| data.entries.get(id).cloned().ok_or("history_not_found"))
                .transpose()?;
            if let Some(previous) = &previous {
                if matches!(previous.source, Source::Session { .. })
                    && (previous.source != draft.source
                        || previous.recipe != draft.recipe
                        || previous.servings != draft.servings)
                {
                    return Err("history_source_changed");
                }
                data.delta(previous, -1)?;
            } else if data.entries.len() >= 1000 {
                return Err("history_capacity");
            }
            let id = if let Some(id) = id {
                id.to_owned()
            } else {
                let id = format!("cook-{}", data.next_id);
                data.next_id = data.next_id.checked_add(1).ok_or("history_capacity")?;
                id
            };
            let entry = Entry {
                id: id.clone(),
                source: draft.source,
                recipe: draft.recipe,
                servings: draft.servings,
                input: draft.input,
                recorded_by: previous
                    .as_ref()
                    .map_or_else(|| actor.to_owned(), |p| p.recorded_by.clone()),
                recorded_at: previous.as_ref().map_or(now, |p| p.recorded_at),
                updated_by: actor.to_owned(),
                updated_at: now,
                epoch: data.policy.enabled.then_some(data.epoch),
            };
            data.delta(&entry, 1)?;
            data.entries.insert(id.clone(), entry);
            Ok(id)
        })
    }
    pub(crate) fn remove(&mut self, expected: u64, id: &str) -> Result<(), &'static str> {
        self.change(expected, |data| {
            let entry = data.entries.remove(id).ok_or("history_not_found")?;
            data.delta(&entry, -1)
        })
    }
    pub(crate) fn clear(&mut self, expected: u64) -> Result<(), &'static str> {
        self.change(expected, |data| {
            data.entries.clear();
            data.stats.clear();
            Ok(())
        })
    }
    pub(crate) fn policy(&mut self, expected: u64, policy: Policy) -> Result<(), &'static str> {
        self.change(expected, |data| {
            data.epoch = data.epoch.checked_add(1).ok_or("history_capacity")?;
            data.policy = policy;
            data.stats.clear();
            Ok(())
        })
    }
}
