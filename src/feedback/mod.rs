//! Explicit completed cooks and opt-in household preference epochs.
mod model;
mod store;
pub(crate) use model::{Data, Entry, Input, Policy, Recipe, Source};
pub(crate) use store::Store;
fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
}
fn valid_text(value: &str, max: usize) -> bool {
    !value.trim().is_empty()
        && value.len() <= max
        && !value.chars().any(|c| c.is_control() && c != '\n')
}

mod api;
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
pub(crate) use api::routes;
#[derive(Clone)]
pub(crate) struct Actor {
    pub(crate) user: String,
    pub(crate) owner: bool,
}
