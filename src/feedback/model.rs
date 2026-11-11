use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Source {
    Manual,
    Session { run_id: String },
}
#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Recipe {
    pub(crate) id: String,
    pub(crate) revision: u64,
    pub(crate) title: String,
    pub(crate) language: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub(crate) date: String,
    pub(crate) timezone: String,
    pub(crate) attendees: Vec<String>,
    pub(crate) active_seconds: Option<u32>,
    pub(crate) household_rating: Option<u8>,
    pub(crate) ratings: BTreeMap<String, u8>,
}
impl Input {
    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        let people: BTreeSet<_> = self.attendees.iter().collect();
        if !crate::planning::date(&self.date)
            || jiff::tz::TimeZone::get(&self.timezone).is_err()
            || people.is_empty()
            || people.len() > 100
            || people.len() != self.attendees.len()
            || people.iter().any(|id| !super::valid_id(id))
            || self.active_seconds.is_some_and(|n| n > 604_800)
            || self.household_rating.is_some_and(|n| !(1..=5).contains(&n))
            || self
                .ratings
                .iter()
                .any(|(id, n)| !people.contains(id) || !(1..=5).contains(n))
        {
            return Err("history_input_invalid");
        }
        Ok(())
    }
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Entry {
    pub(crate) id: String,
    pub(crate) source: Source,
    pub(crate) recipe: Recipe,
    pub(crate) servings: u32,
    pub(crate) input: Input,
    pub(crate) recorded_by: String,
    pub(crate) recorded_at: i64,
    pub(crate) updated_by: String,
    pub(crate) updated_at: i64,
    pub(crate) epoch: Option<u64>,
}
#[derive(Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RatingStats {
    pub(crate) count: u32,
    pub(crate) sum: u32,
}
impl RatingStats {
    fn change(&mut self, value: u8, direction: i32) -> Result<(), &'static str> {
        self.count = self
            .count
            .checked_add_signed(direction)
            .ok_or("history_storage_failed")?;
        self.sum = self
            .sum
            .checked_add_signed(direction * i32::from(value))
            .ok_or("history_storage_failed")?;
        Ok(())
    }
}
#[derive(Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Stats {
    pub(crate) cooks: u32,
    pub(crate) household: RatingStats,
    pub(crate) members: BTreeMap<String, RatingStats>,
    pub(crate) dates: BTreeMap<String, u32>,
}
impl Stats {
    fn change(
        &mut self,
        input: &Input,
        scope: &BTreeSet<String>,
        direction: i32,
    ) -> Result<(), &'static str> {
        self.cooks = self
            .cooks
            .checked_add_signed(direction)
            .ok_or("history_storage_failed")?;
        let date = self.dates.entry(input.date.clone()).or_default();
        *date = date
            .checked_add_signed(direction)
            .ok_or("history_storage_failed")?;
        self.dates.retain(|_, count| *count > 0);
        if let Some(rating) = input.household_rating {
            self.household.change(rating, direction)?;
        }
        for (id, rating) in &input.ratings {
            if scope.contains(id) {
                self.members
                    .entry(id.clone())
                    .or_default()
                    .change(*rating, direction)?;
            }
        }
        self.members.retain(|_, value| value.count > 0);
        Ok(())
    }
}
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Policy {
    pub(crate) enabled: bool,
    pub(crate) member_ids: BTreeSet<String>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Data {
    pub(crate) schema_version: u32,
    pub(crate) revision: u64,
    pub(crate) next_id: u64,
    pub(crate) epoch: u64,
    pub(crate) policy: Policy,
    pub(crate) entries: BTreeMap<String, Entry>,
    pub(crate) stats: BTreeMap<String, Stats>,
}
impl Default for Data {
    fn default() -> Self {
        Self {
            schema_version: 1,
            revision: 0,
            next_id: 1,
            epoch: 1,
            policy: Policy::default(),
            entries: BTreeMap::new(),
            stats: BTreeMap::new(),
        }
    }
}
impl Data {
    pub(crate) fn delta(&mut self, entry: &Entry, direction: i32) -> Result<(), &'static str> {
        if self.policy.enabled && entry.epoch == Some(self.epoch) {
            self.stats
                .entry(entry.recipe.id.clone())
                .or_default()
                .change(&entry.input, &self.policy.member_ids, direction)?;
            self.stats.retain(|_, value| value.cooks > 0);
        }
        Ok(())
    }
    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != 1
            || self.next_id == 0
            || self.epoch == 0
            || self.entries.len() > 1000
            || self.policy.member_ids.len() > 100
            || self.policy.member_ids.iter().any(|id| !super::valid_id(id))
            || (!self.policy.enabled && !self.policy.member_ids.is_empty())
        {
            return Err("history_storage_failed");
        }
        let mut rebuilt = Self {
            policy: self.policy.clone(),
            epoch: self.epoch,
            ..Self::default()
        };
        let mut sessions = BTreeSet::new();
        for (id, entry) in &self.entries {
            entry.input.validate()?;
            let number = id.strip_prefix("cook-").and_then(|s| s.parse::<u64>().ok());
            if id != &entry.id
                || number.is_none_or(|n| n == 0 || n >= self.next_id || *id != format!("cook-{n}"))
                || !crate::recipes::valid_id(&entry.recipe.id)
                || entry.recipe.revision == 0
                || !super::valid_text(&entry.recipe.title, 200)
                || !matches!(entry.recipe.language.as_str(), "en" | "de")
                || !(1..=1_000_000).contains(&entry.servings)
                || !crate::cooking::actor_valid(&entry.recorded_by)
                || !crate::cooking::actor_valid(&entry.updated_by)
                || entry.recorded_at < 0
                || entry.updated_at < entry.recorded_at
                || entry
                    .epoch
                    .is_some_and(|epoch| epoch == 0 || epoch > self.epoch)
            {
                return Err("history_storage_failed");
            }
            if let Source::Session { run_id } = &entry.source {
                if !super::valid_id(run_id) || !sessions.insert(run_id) {
                    return Err("history_duplicate_session");
                }
            }
            rebuilt.delta(entry, 1)?;
        }
        if rebuilt.stats != self.stats {
            return Err("history_storage_failed");
        }
        Ok(())
    }
}
