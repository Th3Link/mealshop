use super::time::{self, LocalTime};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(crate) fn text(value: &str, max: usize) -> bool {
    !value.trim().is_empty()
        && value.len() <= max
        && !value.chars().any(|c| c.is_control() && c != '\n')
}
pub(crate) fn id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum StageOrigin {
    RecipeStep { index: usize },
    Custom { note: String },
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Stage {
    pub(crate) id: String,
    pub(crate) origin: StageOrigin,
    pub(crate) name: String,
    pub(crate) instruction: String,
    pub(crate) dependencies: Vec<String>,
    pub(crate) active_seconds: u32,
    pub(crate) passive_seconds: u32,
    pub(crate) passive_min_seconds: u32,
    pub(crate) passive_max_seconds: u32,
    pub(crate) equipment: Vec<String>,
    pub(crate) lead_seconds: u32,
    pub(crate) repeat: u8,
    pub(crate) wait_between_seconds: u32,
    pub(crate) uncertainty: String,
}
impl Stage {
    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        if !id(&self.id)
            || !text(&self.name, 200)
            || !text(&self.instruction, 4000)
            || self.dependencies.len() > 100
            || self.dependencies.iter().any(|v| !id(v))
            || self.active_seconds > 604_800
            || self.passive_max_seconds > 604_800
            || self.passive_min_seconds > self.passive_seconds
            || self.passive_seconds > self.passive_max_seconds
            || self.lead_seconds > 604_800
            || !(1..=12).contains(&self.repeat)
            || self.wait_between_seconds > 604_800
            || (self.repeat == 1 && self.wait_between_seconds != 0)
            || self.equipment.len() > 20
            || self.equipment.iter().any(|v| !text(v, 120))
            || (!self.uncertainty.is_empty() && !text(&self.uncertainty, 1000))
        {
            return Err("cooking_stage_invalid");
        }
        if let StageOrigin::Custom { note } = &self.origin {
            if !text(note, 500) {
                return Err("cooking_stage_invalid");
            }
        }
        Ok(())
    }
    pub(crate) fn seconds(&self) -> i64 {
        i64::from(self.active_seconds) + i64::from(self.passive_seconds)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Kind {
    Cooking,
    Bread,
    Rolls,
    Pizza,
    Baguette,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipeRef {
    pub(crate) id: String,
    pub(crate) revision: u64,
    pub(crate) title: String,
    pub(crate) language: String,
    pub(crate) reference_servings: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Draft {
    pub(crate) recipe_id: String,
    pub(crate) recipe_revision: u64,
    pub(crate) servings: u32,
    pub(crate) name: String,
    pub(crate) kind: Kind,
    pub(crate) timezone: String,
    pub(crate) target: Option<LocalTime>,
    pub(crate) stages: Vec<Stage>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Timer {
    Pending,
    Running {
        started_at: i64,
        active_until: i64,
        deadline: i64,
    },
    Paused {
        started_at: i64,
        paused_at: i64,
        active_remaining: u32,
        remaining: u32,
    },
    Completed {
        started_at: Option<i64>,
        completed_at: i64,
    },
    Cancelled,
}
impl Timer {
    pub(crate) fn complete(&self) -> bool {
        matches!(self, Self::Completed { .. })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Step {
    pub(crate) stage: Stage,
    pub(crate) cycle: u8,
    pub(crate) lag_seconds: u32,
    pub(crate) not_before: Option<i64>,
    pub(crate) timer: Timer,
    pub(crate) generation: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Timing {
    pub(crate) id: String,
    pub(crate) start: i64,
    pub(crate) active_until: i64,
    pub(crate) finish: i64,
    pub(crate) earliest: i64,
    pub(crate) latest: i64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RunState {
    Active,
    Completed,
    Cancelled,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Subscription {
    pub(crate) enabled: bool,
    pub(crate) language: String,
    pub(crate) permission: Permission,
    pub(crate) reported_at: i64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Permission {
    Default,
    Granted,
    Denied,
    Unsupported,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Run {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) kind: Kind,
    pub(crate) recipe: RecipeRef,
    pub(crate) servings: u32,
    pub(crate) timezone: String,
    pub(crate) target: Option<LocalTime>,
    pub(crate) created_at: i64,
    pub(crate) status: RunState,
    pub(crate) steps: Vec<Step>,
    pub(crate) schedule: Vec<Timing>,
    pub(crate) subscriptions: BTreeMap<String, Subscription>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Data {
    pub(crate) schema_version: u32,
    pub(crate) revision: u64,
    pub(crate) delivery_revision: u64,
    pub(crate) next_id: u64,
    pub(crate) runs: Vec<Run>,
    pub(crate) notices: Vec<super::delivery::Notice>,
}
impl Default for Data {
    fn default() -> Self {
        Self {
            schema_version: 1,
            revision: 0,
            delivery_revision: 0,
            next_id: 1,
            runs: vec![],
            notices: vec![],
        }
    }
}
impl Data {
    pub(crate) fn allocate(&mut self, prefix: &str) -> Result<String, &'static str> {
        let id = format!("{prefix}-{}", self.next_id);
        self.next_id = self.next_id.checked_add(1).ok_or("cooking_capacity")?;
        Ok(id)
    }
    pub(crate) fn run(&self, id: &str) -> Result<&Run, &'static str> {
        self.runs
            .iter()
            .find(|r| r.id == id)
            .ok_or("cooking_run_not_found")
    }
    pub(crate) fn run_mut(&mut self, id: &str) -> Result<&mut Run, &'static str> {
        self.runs
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or("cooking_run_not_found")
    }
}
pub(crate) fn timer_valid(timer: &Timer) -> bool {
    match timer {
        Timer::Pending | Timer::Cancelled => true,
        Timer::Running {
            started_at,
            active_until,
            deadline,
        } => {
            time::valid(*started_at)
                && time::valid(*deadline)
                && started_at <= active_until
                && active_until <= deadline
        }
        Timer::Paused {
            started_at,
            paused_at,
            active_remaining,
            remaining,
        } => {
            time::valid(*started_at)
                && time::valid(*paused_at)
                && started_at <= paused_at
                && active_remaining <= remaining
                && *remaining <= 1_209_600
        }
        Timer::Completed {
            started_at,
            completed_at,
        } => {
            time::valid(*completed_at)
                && started_at.is_none_or(|at| time::valid(at) && at <= *completed_at)
        }
    }
}
