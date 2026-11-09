//! Revisioned household declarations and annotations over immutable reference records.
use crate::nutrition::catalogue::{
    Basis, Catalogue, Ingredient, IngredientText, Source, SourceKind,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

pub(crate) type Shared = Arc<Mutex<Store>>;
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Annotation {
    #[serde(default)]
    pub(crate) organization: Option<crate::ingredient_groups::Organization>,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    #[serde(default)]
    pub(crate) components: Vec<String>,
    #[serde(default)]
    pub(crate) declared_allergens: Vec<String>,
    #[serde(default)]
    pub(crate) archived: bool,
    #[serde(default)]
    pub(crate) favorite: bool,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema_version: u32,
    revision: u64,
    #[serde(default)]
    dietary_revision: Option<u64>,
    customs: Vec<Ingredient>,
    annotations: BTreeMap<String, Annotation>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Declaration {
    pub(crate) name: String,
    pub(crate) language: String,
    pub(crate) form: String,
    pub(crate) recorded_on: String,
    #[serde(default)]
    pub(crate) source_url: String,
    #[serde(default)]
    pub(crate) annotations: Annotation,
}
pub(crate) struct Store {
    base: Catalogue,
    effective: Catalogue,
    document: Document,
    path: Option<PathBuf>,
    writable: bool,
}
impl Store {
    pub(crate) fn memory(mut base: Catalogue) -> Result<Self, String> {
        crate::ingredient_groups::seed(&mut base)?;
        Ok(Self {
            effective: base.clone(),
            base,
            document: Document {
                schema_version: 1,
                revision: 0,
                dietary_revision: Some(0),
                customs: vec![],
                annotations: BTreeMap::new(),
            },
            path: None,
            writable: true,
        })
    }
    pub(crate) fn open(base: Catalogue, directory: &Path) -> Result<Self, String> {
        let mut store = Self::memory(base)?;
        store.path = Some(directory.join("ingredients.json"));
        if directory.join(".ingredients.pending").exists() {
            return Err("Interrupted ingredient write; recover before startup".to_owned());
        }
        if let Some(path) = store.path.as_ref().filter(|path| path.exists()) {
            let mut bytes = Vec::new();
            File::open(path)
                .map_err(|_| "Cannot open ingredient overlay")?
                .take(2_097_153)
                .read_to_end(&mut bytes)
                .map_err(|_| "Cannot read ingredient overlay")?;
            if bytes.len() > 2_097_152 {
                return Err("Ingredient overlay exceeds 2 MiB".to_owned());
            }
            store.document =
                serde_json::from_slice(&bytes).map_err(|_| "Invalid ingredient overlay")?;
        }
        store.validate(&store.document)?;
        store.effective = store.build(&store.document)?;
        Ok(store)
    }
    pub(crate) fn revision(&self) -> u64 {
        self.document.revision
    }
    pub(crate) fn dietary_revision(&self) -> u64 {
        self.document
            .dietary_revision
            .unwrap_or(self.document.revision)
    }
    pub(crate) fn catalogue(&self) -> Catalogue {
        self.effective.clone()
    }
    fn build(&self, document: &Document) -> Result<Catalogue, String> {
        let mut result = self.merge(document);
        crate::ingredient_groups::resolve(&mut result, document.revision)?;
        Ok(result)
    }
    fn merge(&self, document: &Document) -> Catalogue {
        let mut result = self.base.clone();
        result.ingredients.extend(document.customs.iter().cloned());
        for ingredient in &mut result.ingredients {
            if let Some(annotation) = document.annotations.get(&ingredient.id) {
                if let Some(org) = &annotation.organization {
                    ingredient.organization = org.clone();
                }
                ingredient.tags.clone_from(&annotation.tags);
                ingredient.components.clone_from(&annotation.components);
                ingredient
                    .declared_allergens
                    .clone_from(&annotation.declared_allergens);
                ingredient.archived = annotation.archived;
                ingredient.favorite = annotation.favorite;
            }
        }
        result
    }
    fn validate(&self, document: &Document) -> Result<(), String> {
        if document
            .dietary_revision
            .is_some_and(|revision| revision > document.revision)
            || document.schema_version != 1
            || document.customs.len() > 500
            || document.annotations.len() > 1000
            || document
                .customs
                .iter()
                .any(|i| i.source.kind != SourceKind::HouseholdDeclaration)
        {
            return Err("Invalid household ingredient overlay".to_owned());
        }
        let catalogue = self.build(document)?;
        if document
            .annotations
            .keys()
            .any(|id| catalogue.find(id).is_none())
        {
            return Err("Unknown annotated ingredient".to_owned());
        }
        let json =
            serde_json::to_string(&catalogue).map_err(|_| "Ingredient serialization failed")?;
        Catalogue::from_json(&json)?;
        Ok(())
    }
    fn save(&mut self, mut document: Document, expected: u64) -> Result<(), &'static str> {
        if expected != self.document.revision {
            return Err("revision_conflict");
        }
        if !self.writable {
            return Err("ingredient_storage_failed");
        }
        let previous = self.merge(&self.document);
        let current = self.merge(&document);
        let unchanged = previous
            .ingredients
            .iter()
            .map(|item| {
                (
                    &item.id,
                    &item.tags,
                    &item.components,
                    &item.declared_allergens,
                    &item.organization,
                )
            })
            .eq(current.ingredients.iter().map(|item| {
                (
                    &item.id,
                    &item.tags,
                    &item.components,
                    &item.declared_allergens,
                    &item.organization,
                )
            }));
        document.dietary_revision = Some(if unchanged {
            self.dietary_revision()
        } else {
            self.dietary_revision()
                .checked_add(1)
                .ok_or("ingredient_capacity")?
        });
        document.revision = expected.checked_add(1).ok_or("ingredient_capacity")?;
        self.validate(&document).map_err(|_| "invalid_ingredient")?;
        let effective = self.build(&document).map_err(|_| "invalid_ingredient")?;
        let bytes = serde_json::to_vec(&document).map_err(|_| "ingredient_storage_failed")?;
        if bytes.len() > 2_097_152 {
            return Err("ingredient_capacity");
        }
        if let Some(path) = &self.path {
            let pending = path.with_file_name(".ingredients.pending");
            let persist = || -> std::io::Result<()> {
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
                fs::rename(pending, path)?;
                #[cfg(unix)]
                File::open(
                    path.parent()
                        .ok_or_else(|| std::io::Error::other("Missing ingredient parent"))?,
                )?
                .sync_all()?;
                Ok(())
            };
            if persist().is_err() {
                self.writable = false;
                return Err("ingredient_storage_failed");
            }
        }
        self.effective = effective;
        self.document = document;
        Ok(())
    }
    pub(crate) fn create(
        &mut self,
        input: Declaration,
        expected: u64,
    ) -> Result<String, &'static str> {
        if !matches!(input.language.as_str(), "en" | "de")
            || !crate::planning::date(&input.recorded_on)
        {
            return Err("invalid_ingredient");
        }
        let id = format!("custom-{:08}", self.document.customs.len() + 1);
        let translations = if input.language == "de" {
            BTreeMap::from([(
                "de".to_owned(),
                IngredientText {
                    name: input.name.clone(),
                    aliases: vec![],
                    form: Some(input.form.clone()),
                },
            )])
        } else {
            BTreeMap::new()
        };
        let ingredient = Ingredient {
            constituents: BTreeMap::new(),
            organization: crate::ingredient_groups::Organization::default(),
            reference_source: None,
            nutrition_inherited_from: None,
            id: id.clone(),
            category: "other".to_owned(),
            name: if input.language == "en" {
                input.name
            } else {
                format!("Household ingredient {}", self.document.customs.len() + 1)
            },
            form: if input.language == "en" {
                input.form
            } else {
                "As declared by household".to_owned()
            },
            aliases: vec![],
            basis: Basis::Per100gEdiblePortion,
            source: Source {
                kind: SourceKind::HouseholdDeclaration,
                name: "Household declaration".to_owned(),
                url: input.source_url,
                record_id: id.clone(),
                version: "1".to_owned(),
                retrieved_on: input.recorded_on,
                license: "User declaration; reuse rights unspecified".to_owned(),
            },
            uncertainty:
                "Identity and composition declared by household; nutritional composition unknown"
                    .to_owned(),
            nutrients: BTreeMap::new(),
            education: vec![],
            translations,
            tags: vec![],
            components: vec![],
            declared_allergens: vec![],
            archived: false,
            favorite: false,
        };
        let mut document = self.document.clone();
        document.customs.push(ingredient);
        document.annotations.insert(id.clone(), input.annotations);
        self.save(document, expected)?;
        Ok(id)
    }
    pub(crate) fn annotate(
        &mut self,
        id: String,
        mut input: Annotation,
        expected: u64,
    ) -> Result<(), &'static str> {
        let mut document = self.document.clone();
        if input.organization.is_none() {
            input.organization = document
                .annotations
                .get(&id)
                .and_then(|a| a.organization.clone());
        }
        if let Some(current) = &mut input.organization {
            let previous = document
                .annotations
                .get(&id)
                .and_then(|a| a.organization.as_ref())
                .or_else(|| self.base.find(&id).map(|i| &i.organization));
            if let Some(previous) = previous {
                if current.intolerance_triggers.is_none() {
                    current
                        .intolerance_triggers
                        .clone_from(&previous.intolerance_triggers);
                } else if current
                    .intolerance_triggers
                    .as_ref()
                    .is_some_and(Vec::is_empty)
                    && previous.intolerance_triggers.is_none()
                {
                    current.intolerance_triggers = None;
                }
                if let (
                    crate::ingredient_groups::Nutrition::Own { constituents, .. },
                    crate::ingredient_groups::Nutrition::Own {
                        constituents: old, ..
                    },
                ) = (&mut current.nutrition, &previous.nutrition)
                {
                    if constituents.is_none() {
                        constituents.clone_from(old);
                    } else if constituents.as_ref().is_some_and(BTreeMap::is_empty) && old.is_none()
                    {
                        *constituents = None;
                    }
                }
            }
        }
        document.annotations.insert(id, input);
        self.save(document, expected)
    }
}
