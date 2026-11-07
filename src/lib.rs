//! Mealshop ingredient catalogue and nutrition HTTP application.

mod api;
mod nutrition;
mod quantities;

/// Builds the application with the bundled, versioned starter catalogue.
///
/// # Errors
/// Returns a validation error if the embedded catalogue is invalid.
pub fn router() -> Result<axum::Router, String> {
    router_from_catalogue_json(include_str!("../data/catalogue.json"))
}

/// Builds an application with an operator-provided, read-only catalogue.
///
/// The catalogue replaces the entire bundled dataset. No user data is persisted.
///
/// # Errors
/// Returns an error for unsupported schema, invalid quantities, missing provenance,
/// duplicate IDs or a catalogue larger than one MiB.
pub fn router_from_catalogue_json(json: &str) -> Result<axum::Router, String> {
    nutrition::catalogue::Catalogue::from_json(json).map(api::router)
}

/// Project documentation published together with the API reference.
pub mod docs;
