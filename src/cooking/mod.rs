//! Protected durable cooking instances and reviewed multi-day schedules.
pub(crate) mod delivery;
pub(crate) mod model;
pub(crate) mod run_source;
pub(crate) use model::{Run, RunState, Stage};
pub(crate) use time::LocalTime;
pub(crate) mod plan;
mod source;
pub(crate) mod store;
pub(crate) mod time;

#[derive(Clone)]
pub(crate) struct Actor {
    pub(crate) user: String,
    pub(crate) notifications_available: bool,
}
pub(crate) fn actor_valid(user: &str) -> bool {
    !user.is_empty() && user.len() <= 255 && !user.chars().any(char::is_control)
}

pub(crate) mod operations;
pub(crate) mod view;

mod api;
pub(crate) use api::routes;

pub(crate) mod internal;
pub(crate) use delivery::{Job, Outcome};
