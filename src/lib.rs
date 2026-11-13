//! Mealshop ingredient catalogue and nutrition HTTP application.

mod api;
mod constituents;
mod cooking;
pub mod exapp;
mod household;
mod ingredient_api;
mod ingredient_extensions;
mod ingredient_groups;
mod ingredients;
mod library;
mod membership;
mod nutrition;
mod planning;
mod quantities;
mod recipe_api;
mod recipe_images;
mod recipe_origin;
mod recipe_structure;
mod recipes;
mod shopping;

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

/// Builds a local, single-household application with durable recipe revisions.
///
/// The directory is exclusively locked for this application lifetime. This application has
/// no authentication and must only be exposed to trusted local clients. Each saved revision
/// pins its ingredient catalogue subset for reproducible serving estimates.
///
/// # Errors
/// Returns an error for an invalid catalogue, unavailable/locked directory or corrupt revision.
pub fn router_with_recipe_directory(
    json: &str,
    directory: impl AsRef<std::path::Path>,
) -> Result<axum::Router, String> {
    let catalogue = nutrition::catalogue::Catalogue::from_json(json)?;
    let store = recipes::Store::open(directory.as_ref())?;
    Ok(api::router(catalogue.clone()).merge(recipe_api::router(catalogue, store)?))
}

/// Project documentation published together with the API reference.
pub mod docs;

/// Builds the protected local household-profile API in a dedicated directory.
/// The operator token authorizes all household profile reads, exports, edits and removals.
/// # Errors
/// Returns an error for invalid token, catalogue, persisted data or unavailable storage.
pub fn household_router(
    json: &str,
    directory: impl AsRef<std::path::Path>,
    token: String,
) -> Result<axum::Router, String> {
    household::router(json, directory.as_ref(), token, None, None, None)
}

/// Builds household routes with operator and optional read-only credentials.
/// # Errors
/// Returns an error for invalid credentials, catalogue or household storage.
pub fn household_router_with_reader(
    json: &str,
    directory: impl AsRef<std::path::Path>,
    token: String,
    reader: Option<String>,
) -> Result<axum::Router, String> {
    household::router(json, directory.as_ref(), token, reader, None, None)
}

/// Builds recipes, household profiles and protected family meal planning with shared revisions.
/// # Errors
/// Returns an error for invalid data, credentials or unavailable/locked storage directories.
pub fn family_router(
    json: &str,
    recipe_directory: impl AsRef<std::path::Path>,
    household_directory: impl AsRef<std::path::Path>,
    token: String,
    reader: Option<String>,
) -> Result<axum::Router, String> {
    let catalogue = nutrition::catalogue::Catalogue::from_json(json)?;
    let recipes = std::sync::Arc::new(std::sync::Mutex::new(recipes::Store::open(
        recipe_directory.as_ref(),
    )?));
    let ingredients = std::sync::Arc::new(std::sync::Mutex::new(ingredients::Store::open(
        catalogue.clone(),
        household_directory.as_ref(),
    )?));
    let household = household::router(
        json,
        household_directory.as_ref(),
        token,
        reader,
        Some(recipes.clone()),
        Some(ingredients.clone()),
    )?;
    Ok(api::router(catalogue.clone())
        .merge(recipe_api::router_shared(
            ingredients,
            recipes,
            Some(household_directory.as_ref().join("images")),
        ))
        .merge(household))
}

mod feedback;

#[cfg(test)]
mod recipe_creation_tests;

mod imports;
mod mcp;

mod brewing;
mod inventory;
