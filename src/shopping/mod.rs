//! Revisioned household shopping requirements and purchased provenance.
mod api;
pub(crate) mod model;
pub(crate) mod sources;
pub(crate) mod store;
pub(crate) use api::routes;
pub(crate) use store::Store;

mod shops;

pub(crate) mod products;

mod lookup;
