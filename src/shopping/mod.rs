//! Revisioned household shopping requirements and purchased provenance.
mod api;
mod model;
pub(crate) mod sources;
mod store;
pub(crate) use api::routes;
pub(crate) use store::Store;

mod shops;

mod products;

mod lookup;
