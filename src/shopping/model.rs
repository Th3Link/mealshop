use crate::quantities::{Amount, SCALE};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Unit {
    G,
    Kg,
    Ml,
    L,
    Piece,
}
impl Unit {
    pub(crate) fn base(self) -> (Self, u128) {
        match self {
            Self::Kg => (Self::G, 1000),
            Self::L => (Self::Ml, 1000),
            unit => (unit, 1),
        }
    }
    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::G => "g",
            Self::Kg => "kg",
            Self::Ml => "ml",
            Self::L => "l",
            Self::Piece => "piece",
        }
    }
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Quantity {
    pub(crate) amount: Amount,
    pub(crate) unit: Unit,
}
impl Quantity {
    pub(crate) fn normalized(&self) -> Result<(Unit, Amount), &'static str> {
        if self.amount.atoms() == 0 {
            return Err("shopping_quantity_invalid");
        }
        let (unit, factor) = self.unit.base();
        Ok((unit, amount(self.amount.atoms() * factor)?))
    }
}
pub(crate) fn decimal(atoms: u128) -> String {
    format!("{}.{:06}", atoms / SCALE, atoms % SCALE)
}
pub(crate) fn amount(atoms: u128) -> Result<Amount, &'static str> {
    Amount::try_from(decimal(atoms)).map_err(|_| "shopping_quantity_invalid")
}
pub(crate) fn text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Origin {
    Product {
        product: super::products::Product,
        packages: u32,
    },
    Manual {
        label: String,
        quantity: Quantity,
    },
    Recipe {
        recipe_id: String,
        revision: u64,
        title: String,
        language: String,
        servings: u32,
    },
    Meal {
        date: String,
        meal: String,
        recipe_id: String,
        revision: u64,
        title: String,
        language: String,
        servings: u32,
    },
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Item {
    pub(crate) key: String,
    pub(crate) ingredient_id: Option<String>,
    pub(crate) name: String,
    pub(crate) translations: BTreeMap<String, String>,
    pub(crate) unit: Unit,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Requirement {
    pub(crate) item: Item,
    pub(crate) amount: Amount,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Source {
    pub(crate) id: String,
    pub(crate) active: bool,
    pub(crate) origin: Origin,
    pub(crate) requirements: Vec<Requirement>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Purchase {
    pub(crate) id: String,
    pub(crate) source_id: String,
    pub(crate) item: Item,
    pub(crate) amount: Amount,
    pub(crate) origin: Origin,
    pub(crate) reversed: bool,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Data {
    pub(crate) schema_version: u32,
    pub(crate) revision: u64,
    pub(crate) next_id: u64,
    pub(crate) sources: Vec<Source>,
    pub(crate) purchases: Vec<Purchase>,
    #[serde(default)]
    pub(crate) shops: super::shops::Shops,
    #[serde(default)]
    pub(crate) products: super::products::Products,
}
impl Default for Data {
    fn default() -> Self {
        Self {
            schema_version: 1,
            revision: 0,
            next_id: 1,
            sources: vec![],
            purchases: vec![],
            shops: super::shops::Shops::default(),
            products: super::products::Products::default(),
        }
    }
}
impl Data {
    pub(crate) fn allocate(&mut self, kind: &str) -> Result<String, &'static str> {
        let id = format!("{kind}-{}", self.next_id);
        self.next_id = self.next_id.checked_add(1).ok_or("shopping_capacity")?;
        Ok(id)
    }
    pub(crate) fn purchased(&self, source: &str, key: &str) -> u128 {
        self.purchases
            .iter()
            .filter(|p| !p.reversed && p.source_id == source && p.item.key == key)
            .map(|p| p.amount.atoms())
            .sum()
    }
    pub(crate) fn check(&mut self, key: &str, checked: bool) -> Result<(), &'static str> {
        if !self
            .sources
            .iter()
            .any(|s| s.requirements.iter().any(|r| r.item.key == key))
            && !self.purchases.iter().any(|p| p.item.key == key)
        {
            return Err("shopping_item_not_found");
        }
        let before = self.purchases.len();
        if checked {
            let pending: Vec<_> = self
                .sources
                .iter()
                .filter(|s| s.active)
                .flat_map(|s| {
                    s.requirements
                        .iter()
                        .filter(|r| r.item.key == key)
                        .map(move |r| (s, r))
                })
                .filter_map(|(s, r)| {
                    let remaining = r.amount.atoms().saturating_sub(self.purchased(&s.id, key));
                    (remaining > 0)
                        .then(|| (s.id.clone(), r.item.clone(), s.origin.clone(), remaining))
                })
                .collect();
            for (source_id, item, origin, remaining) in pending {
                let id = self.allocate("purchase")?;
                self.purchases.push(Purchase {
                    id,
                    source_id,
                    item,
                    origin,
                    amount: amount(remaining)?,
                    reversed: false,
                });
            }
        } else {
            for purchase in self.purchases.iter_mut().filter(|p| p.item.key == key) {
                purchase.reversed = true;
            }
        }
        self.record_check(key, checked, self.purchases.len() > before)?;
        Ok(())
    }
    pub(crate) fn rows(&self) -> Vec<serde_json::Value> {
        let mut rows: BTreeMap<String, (Item, u128, u128, Vec<serde_json::Value>)> =
            BTreeMap::new();
        let mut receipts_by_source: BTreeMap<&str, BTreeMap<&str, Vec<&Purchase>>> =
            BTreeMap::new();
        for purchase in self.purchases.iter().filter(|p| !p.reversed) {
            receipts_by_source
                .entry(&purchase.source_id)
                .or_default()
                .entry(&purchase.item.key)
                .or_default()
                .push(purchase);
        }
        for source in &self.sources {
            let mut items: BTreeMap<String, Item> = source
                .requirements
                .iter()
                .map(|r| (r.item.key.clone(), r.item.clone()))
                .collect();
            let source_receipts = receipts_by_source.get(source.id.as_str());
            if let Some(entries) = source_receipts {
                for (key, receipts) in entries {
                    if let Some(purchase) = receipts.first() {
                        items
                            .entry((*key).to_owned())
                            .or_insert_with(|| purchase.item.clone());
                    }
                }
            }
            for (key, item) in items {
                let required = if source.active {
                    source
                        .requirements
                        .iter()
                        .find(|r| r.item.key == key)
                        .map_or(0, |r| r.amount.atoms())
                } else {
                    0
                };
                let receipts = source_receipts
                    .and_then(|entries| entries.get(key.as_str()))
                    .cloned()
                    .unwrap_or_default();
                let purchased = receipts.iter().map(|p| p.amount.atoms()).sum();
                let remaining = required.saturating_sub(purchased);
                if purchased == 0 && remaining == 0 {
                    continue;
                }
                let contribution = serde_json::json!({"source_id":source.id,"origin":source.origin,"required":decimal(required),"purchased":decimal(purchased),"receipts":receipts});
                let row = rows.entry(key).or_insert_with(|| (item, 0, 0, vec![]));
                row.1 += remaining;
                row.2 += purchased;
                row.3.push(contribution);
            }
        }
        rows.into_values().map(|(item,remaining,purchased,contributions)| serde_json::json!({"key":item.key,"ingredient_id":item.ingredient_id,"name":item.name,"translations":item.translations,"unit":item.unit,"remaining":decimal(remaining),"purchased":decimal(purchased),"total":decimal(remaining+purchased),"contributions":contributions})).collect()
    }
}
