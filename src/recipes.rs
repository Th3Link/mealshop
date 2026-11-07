//! Versioned local recipe records and an exclusive, append-only file store.

use crate::nutrition::{self, Calculation, catalogue::Catalogue};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

const MAX_REVISIONS: usize = 1000;
const MAX_RECORD_BYTES: u64 = 2_097_152;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Step {
    instruction: String,
    active_seconds: u32,
    passive_seconds: u32,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Recipe {
    pub(crate) title: String,
    pub(crate) language: String,
    author: String,
    pub(crate) lines: Vec<nutrition::RecipeLine>,
    pub(crate) servings: u32,
    steps: Vec<Step>,
    preparation: nutrition::Preparation,
}

fn text(value: &str, max: usize) -> bool {
    !value.trim().is_empty()
        && value.len() <= max
        && !value.chars().any(|c| c.is_control() && c != '\n')
}

impl Recipe {
    pub(crate) fn calculation(&self) -> Calculation {
        Calculation {
            lines: self.lines.clone(),
            servings: self.servings,
            preparation: self.preparation,
        }
    }

    fn validate(&self, catalogue: &Catalogue) -> Result<(), String> {
        if !text(&self.title, 200)
            || !text(&self.author, 200)
            || !matches!(self.language.as_str(), "en" | "de")
            || self.steps.len() > 100
            || self.steps.iter().any(|s| {
                !text(&s.instruction, 4000)
                    || s.active_seconds > 604_800
                    || s.passive_seconds > 604_800
            })
        {
            return Err("Use a title/author of 1–200 bytes, language en/de and at most 100 nonempty steps of 4000 bytes with durations up to 604800 seconds".to_owned());
        }
        nutrition::calculate(catalogue, &self.calculation()).map(|_| ())
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Record {
    schema_version: u32,
    pub(crate) id: String,
    pub(crate) revision: u64,
    pub(crate) recipe: Recipe,
    pub(crate) catalogue: Catalogue,
}

#[derive(Serialize)]
pub(crate) struct Summary {
    id: String,
    revision: u64,
    title: String,
    language: String,
    servings: u32,
}

pub(crate) enum Error {
    Invalid(String),
    NotFound,
    Conflict,
    Storage,
    Full,
}

pub(crate) struct Store {
    directory: PathBuf,
    // Keep the lock file handle alive until the store is dropped.
    lock: File,
    records: BTreeMap<String, Vec<Record>>,
    count: usize,
    // An ambiguous filesystem failure requires operator recovery, not further writes.
    writable: bool,
}

impl Drop for Store {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.directory.join(".mealshop.lock"));
    }
}

impl Store {
    pub(crate) fn open(directory: &Path) -> Result<Self, String> {
        fs::create_dir_all(directory)
            .map_err(|e| format!("Cannot create recipe directory: {e}"))?;
        let directory = fs::canonicalize(directory).map_err(|e| e.to_string())?;
        let lock = private_file(&directory.join(".mealshop.lock")).map_err(|e| {
            format!("Cannot lock recipe directory (stop other writers; see recovery guide): {e}")
        })?;
        // Construct immediately so any later error also releases this process's lock.
        let mut store = Self {
            directory,
            lock,
            records: BTreeMap::new(),
            count: 0,
            writable: true,
        };
        writeln!(store.lock, "pid={}", std::process::id()).map_err(|e| e.to_string())?;
        store.lock.sync_all().map_err(|e| e.to_string())?;
        store.load()?;
        Ok(store)
    }

    fn load(&mut self) -> Result<(), String> {
        let entries = fs::read_dir(&self.directory).map_err(|e| e.to_string())?;
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let filename = entry.file_name();
            let filename = filename.to_str().ok_or("Invalid recipe filename")?;
            if filename == ".mealshop.lock" {
                continue;
            }
            if filename == ".pending" {
                return Err(
                    "Interrupted recipe write: inspect .pending and follow recovery guide"
                        .to_owned(),
                );
            }
            if entry.path().extension().is_none_or(|ext| ext != "json")
                || !entry.file_type().map_err(|e| e.to_string())?.is_file()
            {
                return Err(
                    "Unexpected entry in recipe directory; use a dedicated directory".to_owned(),
                );
            }
            if self.count >= MAX_REVISIONS {
                return Err("Recipe revision limit exceeded".to_owned());
            }
            let mut bytes = Vec::new();
            File::open(entry.path())
                .map_err(|e| e.to_string())?
                .take(MAX_RECORD_BYTES + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() as u64 > MAX_RECORD_BYTES {
                return Err("Recipe revision exceeds 2 MiB".to_owned());
            }
            let record: Record = serde_json::from_slice(&bytes)
                .map_err(|e| format!("Invalid recipe revision: {e}"))?;
            if record.schema_version != 1
                || !valid_id(&record.id)
                || record.revision == 0
                || filename != format!("{}-{}.json", record.id, record.revision)
            {
                return Err("Invalid recipe identity, revision or schema".to_owned());
            }
            let snapshot = serde_json::to_string(&record.catalogue).map_err(|e| e.to_string())?;
            Catalogue::from_json(&snapshot)?;
            record.recipe.validate(&record.catalogue)?;
            self.records
                .entry(record.id.clone())
                .or_default()
                .push(record);
            self.count += 1;
        }
        for revisions in self.records.values_mut() {
            revisions.sort_by_key(|r| r.revision);
            for (index, record) in revisions.iter().enumerate() {
                if record.revision != index as u64 + 1 {
                    return Err("Recipe history has duplicate or missing revisions".to_owned());
                }
            }
        }
        Ok(())
    }

    pub(crate) fn list(&self) -> Vec<Summary> {
        self.records
            .values()
            .filter_map(|r| r.last())
            .map(|r| Summary {
                id: r.id.clone(),
                revision: r.revision,
                title: r.recipe.title.clone(),
                language: r.recipe.language.clone(),
                servings: r.recipe.servings,
            })
            .collect()
    }

    pub(crate) fn get(&self, id: &str, revision: Option<u64>) -> Result<Record, Error> {
        let records = self.records.get(id).ok_or(Error::NotFound)?;
        revision
            .map_or_else(
                || records.last(),
                |n| records.iter().find(|r| r.revision == n),
            )
            .cloned()
            .ok_or(Error::NotFound)
    }

    pub(crate) fn save(
        &mut self,
        recipe: Recipe,
        update: Option<(&str, u64)>,
        catalogue: &Catalogue,
    ) -> Result<Record, Error> {
        if !self.writable {
            return Err(Error::Storage);
        }
        recipe.validate(catalogue).map_err(Error::Invalid)?;
        if self.count >= MAX_REVISIONS {
            return Err(Error::Full);
        }
        let (id, revision) = if let Some((id, expected)) = update {
            let previous = self.get(id, None)?;
            if previous.revision != expected {
                return Err(Error::Conflict);
            }
            (id.to_owned(), expected + 1)
        } else {
            let next = self
                .records
                .keys()
                .filter_map(|id| id.strip_prefix("recipe-")?.parse::<u64>().ok())
                .max()
                .unwrap_or(0)
                + 1;
            if next > 99_999_999 {
                return Err(Error::Full);
            }
            (format!("recipe-{next:08}"), 1)
        };
        let ids: BTreeSet<_> = recipe
            .lines
            .iter()
            .map(|l| l.ingredient_id.as_str())
            .collect();
        let snapshot = Catalogue {
            schema_version: catalogue.schema_version,
            language: catalogue.language.clone(),
            version: catalogue.version.clone(),
            ingredients: catalogue
                .ingredients
                .iter()
                .filter(|i| ids.contains(i.id.as_str()))
                .cloned()
                .collect(),
        };
        let snapshot_json = serde_json::to_string(&snapshot).map_err(|_| Error::Storage)?;
        Catalogue::from_json(&snapshot_json).map_err(Error::Invalid)?;
        let record = Record {
            schema_version: 1,
            id,
            revision,
            recipe,
            catalogue: snapshot,
        };
        let bytes = serde_json::to_vec_pretty(&record).map_err(|_| Error::Storage)?;
        if bytes.len() as u64 > MAX_RECORD_BYTES {
            return Err(Error::Full);
        }
        let target = self
            .directory
            .join(format!("{}-{}.json", record.id, record.revision));
        let pending = self.directory.join(".pending");
        let persist = || -> std::io::Result<()> {
            let mut file = private_file(&pending)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            // Atomic publication without ever overwriting an existing revision.
            fs::hard_link(&pending, &target)?;
            sync_directory(&self.directory)?;
            fs::remove_file(&pending)?;
            sync_directory(&self.directory)
        };
        if persist().is_err() {
            self.writable = false;
            return Err(Error::Storage);
        }
        self.records
            .entry(record.id.clone())
            .or_default()
            .push(record.clone());
        self.count += 1;
        Ok(record)
    }
}

pub(crate) fn valid_id(id: &str) -> bool {
    id.strip_prefix("recipe-")
        .is_some_and(|n| n.len() == 8 && n.bytes().all(|b| b.is_ascii_digit()) && n != "00000000")
}

fn sync_directory(directory: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    File::open(directory)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = directory; // File data is synced; directory fsync is Unix-specific.
    Ok(())
}

fn private_file(path: &Path) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}
