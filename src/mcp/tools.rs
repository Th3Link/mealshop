use super::{
    Authority, Scope,
    store::{Preview, Store},
};
use crate::{
    nutrition::{self, catalogue::Catalogue},
    recipes::{self, Recipe, Record},
};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};
pub(super) struct Error {
    pub(super) code: &'static str,
    pub(super) message: String,
}
impl From<&'static str> for Error {
    fn from(code: &'static str) -> Self {
        Self {
            code,
            message: code.replace('_', " "),
        }
    }
}
impl From<recipes::Error> for Error {
    fn from(error: recipes::Error) -> Self {
        match error {
            recipes::Error::Invalid(message) => Self {
                code: "invalid_recipe",
                message,
            },
            recipes::Error::NotFound => "recipe_not_found".into(),
            recipes::Error::Conflict => "mcp_idempotency_conflict".into(),
            recipes::Error::Storage => "mcp_storage_failed".into(),
            recipes::Error::Full => "recipe_storage_full".into(),
        }
    }
}
fn parse<T: DeserializeOwned>(value: Value) -> Result<T, Error> {
    serde_json::from_value(value).map_err(|_| Error {
        code: "invalid_arguments",
        message: "Check the tool schema, exact decimal quantities and required fields".to_owned(),
    })
}
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Search {
    query: String,
    page: Option<usize>,
}
fn search(value: Value) -> Result<Search, Error> {
    let query: Search = parse(value)?;
    if query.query.len() > 200
        || query
            .page
            .is_some_and(|p| p == 0 || p.checked_mul(20).is_none())
    {
        return Err("invalid_arguments".into());
    }
    Ok(query)
}
fn page(values: Vec<Value>, page: Option<usize>) -> Value {
    let total = values.len();
    let page = page.unwrap_or(1).min(total.div_ceil(20).max(1));
    json!({"total":total,"page":page,"per_page":20,"items":values.into_iter().skip((page-1)*20).take(20).collect::<Vec<_>>()})
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Read {
    id: String,
    revision: Option<u64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Ingredient {
    id: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Draft {
    recipe: Recipe,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Create {
    preview_id: String,
    idempotency_key: String,
    confirmed: bool,
}
fn source(record: &Record) -> Option<Value> {
    match &record.sharing_origin {
        crate::recipe_origin::Origin::Imported { source_url } => {
            Some(json!({"kind":"source_link","source_url":source_url}))
        }
        crate::recipe_origin::Origin::Owned => None,
    }
}
fn content(record: &Record) -> Result<Value, Error> {
    if let Some(source) = source(record) {
        return Ok(json!({"id":record.id,"revision":record.revision,"content":source}));
    }
    let nutrition = nutrition::calculate(&record.catalogue, &record.recipe.calculation())
        .map_err(|_| Error::from("mcp_storage_failed"))?;
    Ok(
        json!({"id":record.id,"revision":record.revision,"content":{"kind":"recipe","recipe":record.recipe,"nutrition":nutrition}}),
    )
}
pub(super) struct Context<'a> {
    pub(super) actor: &'a Authority,
    pub(super) scope: Scope,
    pub(super) store: &'a mut Store,
    pub(super) recipes: &'a mut recipes::Store,
    pub(super) catalogue: &'a Catalogue,
    pub(super) ingredient_revision: u64,
    pub(super) images: &'a std::path::Path,
    pub(super) now: i64,
}
pub(super) fn execute(ctx: &mut Context<'_>, name: &str, args: Value) -> Result<Value, Error> {
    match name {
        "ingredient_search" => {
            let search = search(args)?;
            let q = search.query.trim().to_lowercase();
            let values = ctx
                .catalogue
                .ingredients
                .iter()
                .filter(|i| {
                    i.name.to_lowercase().contains(&q)
                        || i.id.contains(&q)
                        || i.aliases.iter().any(|a| a.to_lowercase().contains(&q))
                        || i.translations.values().any(|tr| {
                            tr.name.to_lowercase().contains(&q)
                                || tr.aliases.iter().any(|a| a.to_lowercase().contains(&q))
                        })
                })
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| Error::from("mcp_storage_failed"))?;
            Ok(page(values, search.page))
        }
        "ingredient_get" => {
            let input: Ingredient = parse(args)?;
            if input.id.len() > 200 {
                return Err("invalid_arguments".into());
            }
            let ingredient = ctx.catalogue.find(&input.id).ok_or("unknown_ingredient")?;
            serde_json::to_value(ingredient).map_err(|_| "mcp_storage_failed".into())
        }
        "recipe_search" => {
            let search = search(args)?;
            let q = search.query.trim().to_lowercase();
            let mut values = Vec::new();
            for record in ctx.recipes.latest() {
                match &record.recipe.origin {
                    crate::recipe_origin::Origin::Imported { source_url } => {
                        if source_url.to_lowercase().contains(&q) {
                            values.push(json!({"id":record.id,"revision":record.revision,"content":{"kind":"source_link","source_url":source_url}}));
                        }
                    }
                    crate::recipe_origin::Origin::Owned => {
                        if record.recipe.title.to_lowercase().contains(&q) {
                            values.push(json!({"id":record.id,"revision":record.revision,"content":{"kind":"recipe_summary","title":record.recipe.title,"language":record.recipe.language,"servings":record.recipe.servings}}));
                        }
                    }
                }
            }
            Ok(page(values, search.page))
        }
        "recipe_get" => {
            let input: Read = parse(args)?;
            if !recipes::valid_id(&input.id) || input.revision == Some(0) {
                return Err("invalid_arguments".into());
            }
            content(&ctx.recipes.get(&input.id, input.revision)?)
        }
        "recipe_preview_create" => preview(ctx, args),
        "recipe_create" => create(ctx, args),
        _ => Err("unknown_tool".into()),
    }
}
fn preview(ctx: &mut Context<'_>, args: Value) -> Result<Value, Error> {
    let input: Draft = serde_json::from_value(args).map_err(|_| Error {
        code: "invalid_recipe",
        message: "Check recipe fields and exact decimal quantities; no recipe was saved".to_owned(),
    })?;
    crate::recipe_images::validate_reference(Some(ctx.images), input.recipe.image_id.as_deref())?;
    let record = ctx
        .recipes
        .prepare(input.recipe.clone(), None, ctx.catalogue)?;
    if serde_json::to_vec(&record)
        .map_err(|_| Error::from("mcp_storage_failed"))?
        .len()
        > 2_097_152
    {
        return Err(recipes::Error::Full.into());
    }
    let projected = content(&record)?;
    let expires_at = ctx.now.checked_add(600).ok_or("mcp_clock_invalid")?;
    let id = ctx.store.preview(Preview {
        actor: ctx.actor.user.clone(),
        grant_revision: ctx.store.grants.revision,
        ingredient_revision: ctx.ingredient_revision,
        deadline: std::time::Instant::now()
            .checked_add(std::time::Duration::from_secs(600))
            .ok_or("mcp_clock_invalid")?,
        recipe: input.recipe,
    })?;
    // A preview predicts content, not an unpublished recipe identity.
    Ok(
        json!({"preview_id":id,"expires_at":expires_at,"ingredient_revision":ctx.ingredient_revision,"content":projected["content"],"saved":false,"confirmation_required":true}),
    )
}
fn create(ctx: &mut Context<'_>, args: Value) -> Result<Value, Error> {
    if ctx.scope != Scope::Create || ctx.actor.role == crate::membership::Role::Reader {
        return Err("mcp_read_only".into());
    }
    let input: Create = parse(args)?;
    if !input.confirmed {
        return Err("mcp_confirmation_required".into());
    }
    let receipt = recipes::Creation {
        actor: ctx.actor.user.clone(),
        key: input.idempotency_key,
        preview_id: input.preview_id,
        recorded_at: ctx.now,
    };
    if !receipt.valid() {
        return Err("invalid_arguments".into());
    }
    if let Some(record) =
        ctx.recipes
            .creation_result(&receipt.actor, &receipt.key, &receipt.preview_id)?
    {
        return Ok(json!({"replayed":true,"record":content(&record)?}));
    }
    let preview = ctx
        .store
        .previews
        .get(&receipt.preview_id)
        .ok_or("mcp_preview_expired")?;
    if preview.actor != ctx.actor.user || preview.grant_revision != ctx.store.grants.revision {
        return Err("mcp_preview_unavailable".into());
    }
    if preview.deadline <= std::time::Instant::now() {
        return Err("mcp_preview_expired".into());
    }
    if preview.ingredient_revision != ctx.ingredient_revision {
        return Err("mcp_ingredients_changed".into());
    }
    crate::recipe_images::validate_reference(Some(ctx.images), preview.recipe.image_id.as_deref())?;
    let recipe = preview.recipe.clone();
    let id = receipt.preview_id.clone();
    let record = ctx.recipes.save_created(recipe, ctx.catalogue, receipt)?;
    ctx.store.previews.remove(&id);
    Ok(json!({"replayed":false,"record":content(&record)?}))
}
