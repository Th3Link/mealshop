//! Protected durable cooking instances and reviewed multi-day schedules.
pub(crate) mod delivery;
mod model;
mod plan;
mod source;
pub(crate) mod store;
mod time;

#[derive(Clone)]
pub(crate) struct Actor {
    pub(crate) user: String,
    pub(crate) notifications_available: bool,
}
pub(crate) fn actor_valid(user: &str) -> bool {
    !user.is_empty() && user.len() <= 255 && !user.chars().any(char::is_control)
}

mod operations;
mod view;

mod api;
pub(crate) use api::routes;

pub(crate) mod internal;
pub(crate) use delivery::{Job, Outcome};
