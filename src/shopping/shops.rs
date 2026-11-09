use super::model::{Data, text};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Shops {
    pub(crate) entries: Vec<Shop>,
    pub(crate) preferences: BTreeMap<String, Preference>,
    pub(crate) assignments: BTreeMap<String, Option<String>>,
    pub(crate) manual_order: BTreeMap<String, Vec<String>>,
    pub(crate) learning: bool,
    pub(crate) sessions: Vec<Session>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Shop {
    pub(crate) id: String,
    pub(crate) name: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Preference {
    pub(crate) first: Option<String>,
    pub(crate) second: Option<String>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Session {
    pub(crate) id: String,
    pub(crate) shop_id: String,
    pub(crate) completed: bool,
    pub(crate) events: Vec<String>,
    pub(crate) corrected: BTreeSet<String>,
}
impl Data {
    pub(crate) fn shop_exists(&self, id: &str) -> bool {
        self.shops.entries.iter().any(|shop| shop.id == id)
    }
    pub(crate) fn item_exists(&self, key: &str) -> bool {
        self.sources
            .iter()
            .any(|s| s.requirements.iter().any(|r| r.item.key == key))
            || self.purchases.iter().any(|p| p.item.key == key)
    }
    pub(crate) fn assigned_shop(&self, key: &str, ingredient: Option<&str>) -> Option<String> {
        self.shops.assignments.get(key).cloned().unwrap_or_else(|| {
            ingredient
                .and_then(|id| self.shops.preferences.get(id))
                .and_then(|p| p.first.clone())
        })
    }
    pub(crate) fn record_check(
        &mut self,
        key: &str,
        checked: bool,
        changed: bool,
    ) -> Result<(), &'static str> {
        if !checked {
            for session in &mut self.shops.sessions {
                if session.events.iter().any(|event| event == key) {
                    session.corrected.insert(key.to_owned());
                }
            }
        } else if changed {
            let ingredient = self
                .sources
                .iter()
                .flat_map(|s| &s.requirements)
                .find(|r| r.item.key == key)
                .and_then(|r| r.item.ingredient_id.as_deref());
            let assigned = self.assigned_shop(key, ingredient);
            if let Some(session) = self.shops.sessions.iter_mut().find(|s| !s.completed) {
                if assigned.as_ref() == Some(&session.shop_id)
                    && !session.events.iter().any(|e| e == key)
                {
                    if session.events.len() >= 1000 {
                        return Err("shopping_capacity");
                    }
                    session.events.push(key.to_owned());
                }
            }
        }
        Ok(())
    }
    pub(crate) fn save_shop(&mut self, id: Option<String>, name: &str) -> Result<(), &'static str> {
        if !text(name, 120)
            || self
                .shops
                .entries
                .iter()
                .any(|s| Some(&s.id) != id.as_ref() && s.name.eq_ignore_ascii_case(name.trim()))
        {
            return Err("shopping_shop_invalid");
        }
        if let Some(id) = id {
            name.trim().clone_into(
                &mut self
                    .shops
                    .entries
                    .iter_mut()
                    .find(|s| s.id == id)
                    .ok_or("shopping_shop_invalid")?
                    .name,
            );
        } else {
            let id = self.allocate("shop")?;
            self.shops.entries.push(Shop {
                id,
                name: name.trim().to_owned(),
            });
        }
        Ok(())
    }
    pub(crate) fn remove_shop(&mut self, id: &str) -> Result<(), &'static str> {
        if !self.shop_exists(id)
            || self
                .shops
                .sessions
                .iter()
                .any(|s| !s.completed && s.shop_id == id)
        {
            return Err("shopping_shop_invalid");
        }
        self.shops.entries.retain(|s| s.id != id);
        self.shops
            .assignments
            .retain(|_, shop| shop.as_deref() != Some(id));
        for p in self.shops.preferences.values_mut() {
            if p.first.as_deref() == Some(id) {
                p.first = None;
            }
            if p.second.as_deref() == Some(id) {
                p.second = None;
            }
        }
        self.shops.manual_order.remove(id);
        self.shops.sessions.retain(|s| s.shop_id != id);
        Ok(())
    }
    pub(crate) fn set_preference(
        &mut self,
        ingredient: String,
        preference: Preference,
    ) -> Result<(), &'static str> {
        if preference.first.is_some() && preference.first == preference.second {
            return Err("shopping_shop_invalid");
        }
        for shop in [&preference.first, &preference.second]
            .into_iter()
            .flatten()
        {
            if !self.shop_exists(shop) {
                return Err("shopping_shop_invalid");
            }
        }
        self.shops.preferences.insert(ingredient, preference);
        Ok(())
    }
    pub(crate) fn assign(
        &mut self,
        key: String,
        shop_id: Option<String>,
    ) -> Result<(), &'static str> {
        if !self.item_exists(&key) || shop_id.as_ref().is_some_and(|id| !self.shop_exists(id)) {
            return Err("shopping_shop_invalid");
        }
        self.shops.assignments.insert(key, shop_id);
        Ok(())
    }
    pub(crate) fn order(&mut self, shop_id: String, keys: Vec<String>) -> Result<(), &'static str> {
        if !self.shop_exists(&shop_id)
            || keys.len() > 1000
            || keys.iter().collect::<BTreeSet<_>>().len() != keys.len()
            || keys.iter().any(|key| !self.item_exists(key))
        {
            return Err("shopping_shop_invalid");
        }
        if keys.is_empty() {
            self.shops.manual_order.remove(&shop_id);
        } else {
            self.shops.manual_order.insert(shop_id, keys);
        }
        Ok(())
    }
    pub(crate) fn learning(&mut self, enabled: bool) {
        self.shops.learning = enabled;
        if !enabled {
            self.shops.sessions.retain(|s| s.completed);
        }
    }
    pub(crate) fn start_session(&mut self, shop_id: String) -> Result<(), &'static str> {
        if !self.shops.learning
            || !self.shop_exists(&shop_id)
            || self.shops.sessions.iter().any(|s| !s.completed)
        {
            return Err("shopping_session_invalid");
        }
        let id = self.allocate("session")?;
        self.shops.sessions.push(Session {
            id,
            shop_id,
            completed: false,
            events: vec![],
            corrected: BTreeSet::new(),
        });
        Ok(())
    }
    pub(crate) fn finish_session(&mut self, id: &str, discard: bool) -> Result<(), &'static str> {
        let session = self
            .shops
            .sessions
            .iter_mut()
            .find(|s| s.id == id && !s.completed)
            .ok_or("shopping_session_invalid")?;
        session.completed = true;
        if discard {
            self.shops.sessions.retain(|s| s.id != id);
        }
        Ok(())
    }
    pub(crate) fn shop_rows(&self) -> Vec<serde_json::Value> {
        let mut ranked = Vec::new();
        let mut scores: BTreeMap<(&str, &str), (u64, usize)> = BTreeMap::new();
        for session in self
            .shops
            .sessions
            .iter()
            .filter(|s| s.completed && s.events.len() >= 2 && s.corrected.is_empty())
        {
            for (position, key) in session.events.iter().enumerate() {
                let entry = scores.entry((&session.shop_id, key)).or_default();
                entry.0 += position as u64 * 10_000 / (session.events.len() as u64 - 1);
                entry.1 += 1;
            }
        }
        for mut row in self.rows() {
            let key = row["key"].as_str().unwrap_or_default();
            let ingredient = row["ingredient_id"].as_str();
            let shop = self.assigned_shop(key, ingredient);
            let fallback = ingredient
                .and_then(|id| self.shops.preferences.get(id))
                .and_then(|p| p.second.clone())
                .filter(|id| Some(id) != shop.as_ref());
            let (sum, samples) = shop
                .as_deref()
                .and_then(|id| scores.get(&(id, key)))
                .copied()
                .unwrap_or_default();
            let score = sum / samples.max(1) as u64;
            let manual = shop
                .as_ref()
                .and_then(|id| self.shops.manual_order.get(id))
                .and_then(|order| order.iter().position(|k| k == key));
            let basis = if manual.is_some() {
                "manual"
            } else if self.shops.learning && samples >= 3 {
                "learned"
            } else if self.shops.learning {
                "uncertain"
            } else {
                "default"
            };
            let rank = manual.map_or_else(
                || {
                    if basis == "learned" {
                        (1, score)
                    } else {
                        (2, 0)
                    }
                },
                |position| (0, position as u64),
            );
            let name = row["name"].as_str().unwrap_or_default().to_lowercase();
            row["shop_id"] = serde_json::json!(shop);
            row["fallback_shop_id"] = serde_json::json!(fallback);
            row["order_basis"] = serde_json::json!(basis);
            row["samples"] = serde_json::json!(samples);
            ranked.push(((shop, rank, name), row));
        }
        ranked.sort_by(|a, b| a.0.cmp(&b.0));
        ranked.into_iter().map(|(_, row)| row).collect()
    }
    pub(crate) fn validate_shops(&self) -> Result<(), &'static str> {
        let shops = &self.shops;
        if shops.entries.len() > 100
            || shops.preferences.len() > 2000
            || shops.assignments.len() > 2000
            || shops.sessions.len() > 200
        {
            return Err("shopping_capacity");
        }
        let mut ids = BTreeSet::new();
        for shop in &shops.entries {
            if !text(&shop.name, 120)
                || !ids.insert(&shop.id)
                || !self.allocated_id(&shop.id, "shop-")
            {
                return Err("shopping_data_invalid");
            }
        }
        if shops.sessions.iter().filter(|s| !s.completed).count() > 1
            || (!shops.learning && shops.sessions.iter().any(|s| !s.completed))
        {
            return Err("shopping_data_invalid");
        }
        for session in &shops.sessions {
            if !ids.insert(&session.id)
                || !self.allocated_id(&session.id, "session-")
                || !self.shop_exists(&session.shop_id)
                || session.events.len() > 1000
                || session.events.iter().collect::<BTreeSet<_>>().len() != session.events.len()
                || session.events.iter().any(|k| !self.item_exists(k))
                || session
                    .corrected
                    .iter()
                    .any(|k| !session.events.contains(k))
            {
                return Err("shopping_data_invalid");
            }
        }
        if shops.assignments.iter().any(|(key, shop)| {
            !self.item_exists(key) || shop.as_ref().is_some_and(|id| !self.shop_exists(id))
        }) || shops.preferences.iter().any(|(id, p)| {
            !text(id, 120)
                || (p.first.is_some() && p.first == p.second)
                || [&p.first, &p.second]
                    .into_iter()
                    .flatten()
                    .any(|id| !self.shop_exists(id))
        }) || shops.manual_order.iter().any(|(id, keys)| {
            !self.shop_exists(id)
                || keys.len() > 1000
                || keys.iter().collect::<BTreeSet<_>>().len() != keys.len()
                || keys.iter().any(|k| !self.item_exists(k))
        }) {
            return Err("shopping_data_invalid");
        }
        Ok(())
    }
    pub(crate) fn prune_shop_metadata(&mut self) {
        let keys: BTreeSet<_> = self
            .sources
            .iter()
            .flat_map(|s| &s.requirements)
            .map(|r| r.item.key.clone())
            .chain(self.purchases.iter().map(|p| p.item.key.clone()))
            .collect();
        self.shops.assignments.retain(|k, _| keys.contains(k));
        for order in self.shops.manual_order.values_mut() {
            order.retain(|k| keys.contains(k));
        }
    }
    pub(crate) fn allocated_id(&self, id: &str, prefix: &str) -> bool {
        id.strip_prefix(prefix)
            .and_then(|n| n.parse::<u64>().ok())
            .is_some_and(|n| n > 0 && n < self.next_id && id == format!("{prefix}{n}"))
    }
}
