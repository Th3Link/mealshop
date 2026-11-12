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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) part_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) ingredient_line_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) prepared_parts: Vec<String>,
    pub(crate) instruction: String,
    pub(crate) active_seconds: u32,
    pub(crate) passive_seconds: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) passive_max_seconds: Option<u32>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Recipe {
    #[serde(
        default,
        skip_serializing_if = "crate::recipe_origin::Origin::is_owned"
    )]
    pub(crate) origin: crate::recipe_origin::Origin,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) parts: Vec<crate::recipe_structure::Part>,
    pub(crate) title: String,
    pub(crate) language: String,
    #[serde(default)]
    author: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) image_id: Option<String>,
    pub(crate) lines: Vec<nutrition::RecipeLine>,
    pub(crate) servings: u32,
    pub(crate) steps: Vec<Step>,
    preparation: nutrition::Preparation,
}

fn text(value: &str, max: usize) -> bool {
    !value.trim().is_empty()
        && value.len() <= max
        && !value.chars().any(|c| c.is_control() && c != '\n')
}

impl Recipe {
    pub(crate) fn input_lines(&self) -> impl Iterator<Item = &nutrition::RecipeLine> {
        self.lines
            .iter()
            .filter(|line| line.prepared_part.is_none())
    }
    pub(crate) fn calculation(&self) -> Calculation {
        Calculation {
            lines: self.input_lines().cloned().collect(),
            servings: self.servings,
            preparation: if self.lines.iter().any(|line| line.prepared_part.is_some()) {
                nutrition::Preparation::CookingUnmodelled
            } else {
                self.preparation
            },
        }
    }

    fn validate(&self, catalogue: &Catalogue) -> Result<(), String> {
        self.origin.validate()?;
        if !text(&self.title, 200)
            || (!self.author.is_empty() && !text(&self.author, 200))
            || self
                .image_id
                .as_ref()
                .is_some_and(|id| !crate::recipe_images::valid_id(id))
            || !matches!(self.language.as_str(), "en" | "de")
            || self.steps.len() > 100
            || self.lines.is_empty()
            || self.lines.len() > 100
            || self.steps.iter().any(|s| {
                !text(&s.instruction, 4000)
                    || s.active_seconds > 604_800
                    || s.passive_seconds > 604_800
                    || s.passive_max_seconds
                        .is_some_and(|max| max < s.passive_seconds || max > 604_800)
            })
        {
            return Err("Use a title of 1–200 bytes and an optional author of at most 200 bytes, language en/de and at most 100 nonempty steps of 4000 bytes with durations up to 604800 seconds".to_owned());
        }
        crate::recipe_structure::validate(self)?;
        for line in self
            .lines
            .iter()
            .filter(|line| line.prepared_part.is_some())
        {
            nutrition::validate_line(catalogue, line)?;
        }
        nutrition::calculate(catalogue, &self.calculation()).map(|_| ())
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Record {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) import_receipt: Option<crate::imports::Receipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) creation: Option<Creation>,
    #[serde(skip)]
    pub(crate) sharing_origin: crate::recipe_origin::Origin,
    schema_version: u32,
    pub(crate) id: String,
    pub(crate) revision: u64,
    pub(crate) recipe: Recipe,
    pub(crate) catalogue: Catalogue,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Creation {
    pub(crate) actor: String,
    pub(crate) key: String,
    pub(crate) preview_id: String,
    pub(crate) recorded_at: i64,
}
impl Creation {
    pub(crate) fn valid(&self) -> bool {
        crate::membership::valid_user(&self.actor)
            && Self::valid_key(&self.key)
            && self.preview_id.starts_with("mcp-")
            && self.preview_id.len() == 68
            && self.preview_id[4..].bytes().all(|b| b.is_ascii_hexdigit())
            && self.recorded_at >= 0
    }
    pub(crate) fn valid_key(key: &str) -> bool {
        (16..=128).contains(&key.len())
            && key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
    }
}

#[derive(Serialize)]
pub(crate) struct Summary {
    id: String,
    revision: u64,
    title: String,
    language: String,
    servings: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_id: Option<String>,
}

#[derive(Debug)]
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
    pub(crate) fn latest(&self) -> impl Iterator<Item = &Record> {
        self.records.values().filter_map(|records| records.last())
    }
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
        let mut creation_keys = BTreeSet::new();
        let mut import_keys = BTreeSet::new();
        for revisions in self.records.values_mut() {
            revisions.sort_by_key(|r| r.revision);
            if revisions
                .windows(2)
                .any(|pair| !pair[1].recipe.origin.follows(&pair[0].recipe.origin))
            {
                return Err(
                    "Imported recipe provenance changed across persisted revisions".to_owned(),
                );
            }
            for (index, record) in revisions.iter().enumerate() {
                if let Some(creation) = &record.creation {
                    if record.revision != 1
                        || !creation.valid()
                        || !creation_keys.insert((creation.actor.clone(), creation.key.clone()))
                    {
                        return Err("Invalid or duplicate recipe creation receipt".to_owned());
                    }
                }
                if let Some(receipt) = &record.import_receipt {
                    if index != 0
                        || record.creation.is_some()
                        || !receipt.valid()
                        || record.recipe.origin
                            != (crate::recipe_origin::Origin::Imported {
                                source_url: receipt.source.requested_url.clone(),
                            })
                        || !import_keys.insert((receipt.actor.clone(), receipt.key.clone()))
                    {
                        return Err("Invalid or duplicate recipe import receipt".to_owned());
                    }
                }
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
                image_id: r.recipe.image_id.clone(),
            })
            .collect()
    }

    pub(crate) fn containing(&self, ingredient_ids: &BTreeSet<String>) -> Vec<Summary> {
        let ids: BTreeSet<_> = self
            .records
            .values()
            .filter_map(|versions| versions.last())
            .filter(|record| {
                let ingredients = record
                    .recipe
                    .lines
                    .iter()
                    .map(|line| line.ingredient_id.clone())
                    .collect();
                !record
                    .catalogue
                    .closure(&ingredients)
                    .is_disjoint(ingredient_ids)
            })
            .map(|record| record.id.as_str())
            .collect();
        self.list()
            .into_iter()
            .filter(|recipe| ids.contains(recipe.id.as_str()))
            .collect()
    }

    pub(crate) fn get(&self, id: &str, revision: Option<u64>) -> Result<Record, Error> {
        let records = self.records.get(id).ok_or(Error::NotFound)?;
        let mut record = revision
            .map_or_else(
                || records.last(),
                |n| records.iter().find(|r| r.revision == n),
            )
            .cloned()
            .ok_or(Error::NotFound)?;
        nutrition::estimates::attach(&mut record.catalogue).map_err(Error::Invalid)?;
        record.sharing_origin = records.last().ok_or(Error::NotFound)?.recipe.origin.clone();
        record.import_receipt = records.first().and_then(|r| r.import_receipt.clone());
        Ok(record)
    }

    fn validate_active_ingredients(
        &self,
        recipe: &Recipe,
        update: Option<(&str, u64)>,
        catalogue: &Catalogue,
    ) -> Result<(), Error> {
        let previous_ids = update
            .map(|(id, _)| {
                self.get(id, None).map(|record| {
                    record
                        .recipe
                        .lines
                        .into_iter()
                        .map(|line| line.ingredient_id)
                        .collect::<BTreeSet<_>>()
                })
            })
            .transpose()?
            .unwrap_or_default();
        if recipe.lines.iter().any(|line| {
            catalogue
                .find(&line.ingredient_id)
                .is_some_and(|ingredient| ingredient.archived)
                && !previous_ids.contains(&line.ingredient_id)
        }) {
            return Err(Error::Invalid(
                "Archived ingredients cannot be newly added to recipes".to_owned(),
            ));
        }
        Ok(())
    }

    pub(crate) fn prepare(
        &self,
        recipe: Recipe,
        update: Option<(&str, u64)>,
        catalogue: &Catalogue,
    ) -> Result<Record, Error> {
        if !self.writable {
            return Err(Error::Storage);
        }
        recipe.validate(catalogue).map_err(Error::Invalid)?;
        self.validate_active_ingredients(&recipe, update, catalogue)?;
        if self.count >= MAX_REVISIONS {
            return Err(Error::Full);
        }
        let (id, revision) = if let Some((id, expected)) = update {
            let previous = self.get(id, None)?;
            if previous.revision != expected {
                return Err(Error::Conflict);
            }
            if !recipe.origin.follows(&previous.recipe.origin) {
                return Err(Error::Invalid(
                    "An imported recipe must retain its original source URL".to_owned(),
                ));
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
        let ids: BTreeSet<String> = recipe
            .lines
            .iter()
            .map(|l| l.ingredient_id.clone())
            .collect();
        let ids = catalogue.closure(&ids);
        let snapshot = Catalogue {
            schema_version: catalogue.schema_version,
            language: catalogue.language.clone(),
            version: catalogue.version.clone(),
            ingredients: catalogue
                .ingredients
                .iter()
                .filter(|i| ids.contains(&i.id))
                .cloned()
                .collect(),
        };
        let snapshot_json = serde_json::to_string(&snapshot).map_err(|_| Error::Storage)?;
        Catalogue::from_json(&snapshot_json).map_err(Error::Invalid)?;
        Ok(Record {
            import_receipt: None,
            creation: None,
            sharing_origin: recipe.origin.clone(),
            schema_version: 1,
            id,
            revision,
            recipe,
            catalogue: snapshot,
        })
    }
    pub(crate) fn save(
        &mut self,
        recipe: Recipe,
        update: Option<(&str, u64)>,
        catalogue: &Catalogue,
    ) -> Result<Record, Error> {
        let record = self.prepare(recipe, update, catalogue)?;
        let mut record = self.publish(record)?;
        record.import_receipt = self
            .records
            .get(&record.id)
            .and_then(|revisions| revisions.first())
            .and_then(|first| first.import_receipt.clone());
        Ok(record)
    }
    pub(crate) fn creation_result(
        &self,
        actor: &str,
        key: &str,
        preview: &str,
    ) -> Result<Option<Record>, Error> {
        for revisions in self.records.values() {
            if let Some(record) = revisions.first() {
                if let Some(creation) = &record.creation {
                    if creation.actor == actor && creation.key == key {
                        if creation.preview_id != preview {
                            return Err(Error::Conflict);
                        }
                        return self.get(&record.id, Some(1)).map(Some);
                    }
                }
            }
        }
        Ok(None)
    }
    pub(crate) fn save_created(
        &mut self,
        recipe: Recipe,
        catalogue: &Catalogue,
        creation: Creation,
    ) -> Result<Record, Error> {
        if !creation.valid() {
            return Err(Error::Invalid("Invalid recipe creation receipt".to_owned()));
        }
        if let Some(record) =
            self.creation_result(&creation.actor, &creation.key, &creation.preview_id)?
        {
            return Ok(record);
        }
        let mut record = self.prepare(recipe, None, catalogue)?;
        record.creation = Some(creation);
        self.publish(record)
    }
    pub(crate) fn import_result(
        &self,
        actor: &str,
        key: &str,
        preview: &str,
        candidate: usize,
        recipe: &Recipe,
    ) -> Result<Option<Record>, Error> {
        for revisions in self.records.values() {
            if let Some(record) = revisions.first() {
                if let Some(receipt) = &record.import_receipt {
                    if receipt.actor == actor && receipt.key == key {
                        if receipt.preview_id != preview
                            || receipt.candidate != candidate
                            || serde_json::to_value(&record.recipe).map_err(|_| Error::Storage)?
                                != serde_json::to_value(recipe).map_err(|_| Error::Storage)?
                        {
                            return Err(Error::Conflict);
                        }
                        return self.get(&record.id, Some(1)).map(Some);
                    }
                }
            }
        }
        Ok(None)
    }
    pub(crate) fn save_imported(
        &mut self,
        recipe: Recipe,
        catalogue: &Catalogue,
        receipt: crate::imports::Receipt,
    ) -> Result<Record, Error> {
        if !receipt.valid()
            || recipe.origin
                != (crate::recipe_origin::Origin::Imported {
                    source_url: receipt.source.requested_url.clone(),
                })
        {
            return Err(Error::Invalid(
                "An import must retain its reviewed original source".to_owned(),
            ));
        }
        if let Some(record) = self.import_result(
            &receipt.actor,
            &receipt.key,
            &receipt.preview_id,
            receipt.candidate,
            &recipe,
        )? {
            return Ok(record);
        }
        let mut record = self.prepare(recipe, None, catalogue)?;
        record.import_receipt = Some(receipt);
        self.publish(record)
    }
    pub(crate) fn creation_activity(&self) -> Vec<serde_json::Value> {
        let mut entries = self.records.values().filter_map(|revisions| {
            let record = revisions.first()?;
            let creation = record.creation.as_ref()?;
            Some(serde_json::json!({"recipe_id":record.id,"revision":1,"actor":creation.actor,"recorded_at":creation.recorded_at,"title":record.recipe.title,"language":record.recipe.language}))
        }).collect::<Vec<_>>();
        entries.sort_by_key(|value| std::cmp::Reverse(value["recorded_at"].as_i64()));
        entries
    }
    fn publish(&mut self, record: Record) -> Result<Record, Error> {
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
