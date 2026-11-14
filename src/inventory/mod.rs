//! Explicit household stock movements and shopping coverage share one atomic publication.
pub(crate) mod allocations;
mod cooking;
mod ledger;
mod lots;
pub(crate) mod model;
mod offers;
mod operations;
pub(crate) mod prepared;
mod purchases;
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
pub(crate) mod validation;

mod api;
mod view;
pub(crate) use api::routes;
fn now() -> Result<u64, &'static str> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|t| t.as_secs())
        .map_err(|_| "inventory_storage_failed")
}
