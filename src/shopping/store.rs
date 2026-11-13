use super::model::{Data, Item, Origin, Unit, text};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub(crate) struct Store {
    path: PathBuf,
    pub(crate) data: Data,
    writable: bool,
}
fn item_valid(item: &Item, source: &str) -> bool {
    let expected = item.ingredient_id.as_ref().map_or_else(
        || format!("manual:{source}:{}", item.unit.key()),
        |id| format!("ingredient:{id}:{}", item.unit.key()),
    );
    text(&item.key, 300)
        && item.key == expected
        && text(&item.name, 200)
        && !matches!(item.unit, Unit::Kg | Unit::L)
        && item.ingredient_id.as_ref().is_none_or(|id| text(id, 120))
        && item.translations.len() <= 20
        && item
            .translations
            .iter()
            .all(|(lang, name)| text(lang, 20) && text(name, 200))
}
pub(crate) fn origin_valid(origin: &Origin) -> bool {
    if let Origin::Meal { date, meal, .. } = origin {
        if !crate::planning::date(date) || !text(meal, 80) {
            return false;
        }
    }
    match origin {
        Origin::Offer { offer, packages } => {
            crate::inventory::validation::offer(offer) && (1..=1_000_000).contains(packages)
        }
        Origin::Product { product, packages } => {
            super::products::product_valid(product) && (1..=1_000_000).contains(packages)
        }
        Origin::Manual { label, quantity } => text(label, 200) && quantity.normalized().is_ok(),
        Origin::Recipe {
            recipe_id,
            revision,
            title,
            language,
            servings,
        }
        | Origin::Meal {
            recipe_id,
            revision,
            title,
            language,
            servings,
            ..
        } => {
            crate::recipes::valid_id(recipe_id)
                && *revision > 0
                && text(title, 200)
                && matches!(language.as_str(), "en" | "de")
                && (1..=1_000_000).contains(servings)
        }
    }
}
fn validate(data: &Data) -> Result<(), &'static str> {
    if data.schema_version != 1
        || data.next_id == 0
        || data.sources.len() > 1000
        || data.purchases.len() > 5000
    {
        return Err("shopping_capacity");
    }
    let mut ids = BTreeSet::new();
    let valid_id = |id: &str, prefix: &str| {
        id.strip_prefix(prefix)
            .and_then(|n| n.parse::<u64>().ok())
            .is_some_and(|n| n > 0 && n < data.next_id && id == format!("{prefix}{n}"))
    };
    for source in &data.sources {
        if !ids.insert(&source.id)
            || !valid_id(&source.id, "source-")
            || !origin_valid(&source.origin)
            || source.requirements.len() > 100
        {
            return Err("shopping_data_invalid");
        }
        let mut keys = BTreeSet::new();
        for requirement in &source.requirements {
            if !keys.insert(&requirement.item.key)
                || !item_valid(&requirement.item, &source.id)
                || requirement.amount.atoms() == 0
            {
                return Err("shopping_data_invalid");
            }
        }
    }
    for purchase in &data.purchases {
        if !ids.insert(&purchase.id)
            || !valid_id(&purchase.id, "purchase-")
            || !data.sources.iter().any(|s| s.id == purchase.source_id)
            || !item_valid(&purchase.item, &purchase.source_id)
            || purchase.amount.atoms() == 0
            || !origin_valid(&purchase.origin)
        {
            return Err("shopping_data_invalid");
        }
    }
    super::mobile::validate_receipts(data)?;
    data.validate_shops()?;
    data.validate_products()?;
    crate::inventory::validation::validate(data)?;
    Ok(())
}
impl Store {
    pub(crate) fn open(directory: &Path) -> Result<Self, String> {
        if directory.join(".shopping.pending").exists() {
            return Err(
                "Interrupted shopping write; recover a verified backup before startup".to_owned(),
            );
        }
        let path = directory.join("shopping.json");
        let data = if path.exists() {
            let mut bytes = Vec::new();
            File::open(&path)
                .map_err(|_| "Cannot read shopping list")?
                .take(4_194_305)
                .read_to_end(&mut bytes)
                .map_err(|_| "Cannot read shopping list")?;
            if bytes.len() > 4_194_304 {
                return Err("Shopping data exceeds 4 MiB".to_owned());
            }
            serde_json::from_slice(&bytes)
                .map_err(|_| "Invalid shopping data; restore a verified backup")?
        } else {
            Data::default()
        };
        validate(&data).map_err(|_| "Invalid shopping data; restore a verified backup")?;
        Ok(Self {
            path,
            data,
            writable: true,
        })
    }
    pub(crate) fn ready(&self) -> Result<(), &'static str> {
        if self.writable {
            Ok(())
        } else {
            Err("shopping_storage_failed")
        }
    }
    pub(crate) fn prepare(
        &self,
        expected: u64,
        action: impl FnOnce(&mut Data) -> Result<(), &'static str>,
    ) -> Result<Data, &'static str> {
        self.ready()?;
        if expected != self.data.revision {
            return Err("shopping_revision_conflict");
        }
        let mut next = self.data.clone();
        action(&mut next)?;
        crate::inventory::allocations::reconcile(&mut next)?;
        next.prune_shop_metadata();
        next.revision = next.revision.checked_add(1).ok_or("shopping_capacity")?;
        validate(&next)?;
        let bytes = serde_json::to_vec(&next).map_err(|_| "shopping_storage_failed")?;
        if bytes.len() > 4_194_304 {
            return Err("shopping_capacity");
        }
        Ok(next)
    }
    pub(crate) fn change(
        &mut self,
        expected: u64,
        action: impl FnOnce(&mut Data) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let next = self.prepare(expected, action)?;
        let bytes = serde_json::to_vec(&next).map_err(|_| "shopping_storage_failed")?;
        let pending = self.path.with_file_name(".shopping.pending");
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
            return Err("shopping_storage_failed");
        }
        self.data = next;
        Ok(())
    }
    pub(crate) fn view(&self) -> serde_json::Value {
        serde_json::json!({"revision":self.data.revision,"items":self.data.shop_rows(),"sources":self.data.sources,"shops":self.data.shops,"products":self.data.products.records})
    }
}
