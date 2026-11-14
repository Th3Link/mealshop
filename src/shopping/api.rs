use super::{
    model::{Item, Origin, Quantity, Requirement, text},
    sources::{self, Draft, MealSlot, Selection},
    store::Store,
};
use crate::household::{self, Shared};
use axum::{
    Json, Router,
    extract::{State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::get,
};
use serde::Deserialize;
use serde_json::json;
#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Command {
    SaveProduct {
        product: super::products::ProductInput,
    },
    AddProduct {
        barcode: String,
        packages: u32,
    },
    RemoveProduct {
        barcode: String,
    },
    SaveShop {
        id: Option<String>,
        name: String,
    },
    RemoveShop {
        id: String,
    },
    Preference {
        ingredient_id: String,
        first: Option<String>,
        second: Option<String>,
    },
    AssignShop {
        key: String,
        shop_id: Option<String>,
    },
    Order {
        shop_id: String,
        keys: Vec<String>,
    },
    Learning {
        enabled: bool,
    },
    ResetHistory,
    StartSession {
        shop_id: String,
    },
    FinishSession {
        id: String,
        discard: bool,
    },
    AddManual {
        source_id: Option<String>,
        label: String,
        ingredient_id: Option<String>,
        quantity: Quantity,
        #[serde(default)]
        purchased: bool,
    },
    AddRecipe {
        source_id: Option<String>,
        recipe: Selection,
    },
    SyncPlan {
        plan_revision: u64,
        slots: Vec<MealSlot>,
    },
    RemoveSource {
        id: String,
    },
    Check {
        key: String,
        checked: bool,
    },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Mutation {
    pub(super) expected_revision: u64,
    pub(super) command: Command,
}
pub(crate) fn routes() -> Router<Shared> {
    Router::new()
        .merge(super::plan_review::routes())
        .merge(super::mobile::routes())
        .route("/api/household/shopping", get(list).post(change))
        .route(
            "/api/household/shopping/lookup",
            axum::routing::post(super::lookup::lookup),
        )
}
fn failure(code: &str) -> Response {
    let status = match code {
        "shopping_revision_conflict" => StatusCode::CONFLICT,
        "shopping_storage_failed" => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::UNPROCESSABLE_ENTITY,
    };
    household::reply(status, json!({"code":code}))
}
async fn list(State(state): State<Shared>, headers: HeaderMap) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    match tokio::task::spawn_blocking(move || {
        state
            .shopping
            .lock()
            .map(|store| store.view())
            .map_err(|_| ())
    })
    .await
    {
        Ok(Ok(value)) => household::reply(StatusCode::OK, value),
        _ => failure("shopping_storage_failed"),
    }
}
async fn change(
    State(state): State<Shared>,
    headers: HeaderMap,
    body: Result<Json<Mutation>, JsonRejection>,
) -> Response {
    if !household::can_write(&state, &headers) {
        return household::reply(
            StatusCode::FORBIDDEN,
            json!({"code":"household_write_required"}),
        );
    }
    let Ok(Json(input)) = body else {
        return failure("shopping_request_invalid");
    };
    match tokio::task::spawn_blocking(move || {
        let recipes = state
            .recipes
            .as_ref()
            .map(|store| store.lock())
            .transpose()
            .map_err(|_| "recipe_store_unavailable")?;
        let plans = state.plans.lock().map_err(|_| "mealplan_storage_failed")?;
        let ingredients = state
            .catalogue
            .lock()
            .map_err(|_| "ingredient_storage_failed")?;
        let mut store = state
            .shopping
            .lock()
            .map_err(|_| "shopping_storage_failed")?;
        execute(
            &mut store,
            &ingredients,
            recipes.as_deref(),
            &plans,
            input,
            None,
        )?;
        Ok::<_, &'static str>(store.view())
    })
    .await
    {
        Ok(Ok(value)) => household::reply(StatusCode::OK, value),
        Ok(Err(code)) => failure(code),
        Err(_) => failure("shopping_storage_failed"),
    }
}
pub(super) fn execute(
    store: &mut Store,
    ingredients: &crate::ingredients::Store,
    recipes: Option<&crate::recipes::Store>,
    plans: &crate::planning::PlanStore,
    input: Mutation,
    receipt: Option<super::mobile::Receipt>,
) -> Result<(), &'static str> {
    store.change(input.expected_revision, |data| {
        match input.command {
            Command::SaveProduct { product } => {
                data.save_product(product, &ingredients.catalogue(), super::lookup::now()?)
            }
            Command::AddProduct { barcode, packages } => {
                data.add_product(&barcode, packages, &ingredients.catalogue())
            }
            Command::RemoveProduct { barcode } => {
                data.products
                    .records
                    .remove(&super::products::gtin(&barcode)?)
                    .ok_or("shopping_product_not_found")?;
                Ok(())
            }
            Command::SaveShop { id, name } => data.save_shop(id, &name),
            Command::RemoveShop { id } => data.remove_shop(&id),
            Command::Preference {
                ingredient_id,
                first,
                second,
            } => {
                if !ingredients
                    .catalogue()
                    .ingredients
                    .iter()
                    .any(|i| i.id == ingredient_id)
                {
                    return Err("unknown_ingredient");
                }
                data.set_preference(ingredient_id, super::shops::Preference { first, second })
            }
            Command::AssignShop { key, shop_id } => data.assign(key, shop_id),
            Command::Order { shop_id, keys } => data.order(shop_id, keys),
            Command::Learning { enabled } => {
                data.learning(enabled);
                Ok(())
            }
            Command::ResetHistory => {
                data.shops.sessions.clear();
                Ok(())
            }
            Command::StartSession { shop_id } => data.start_session(shop_id),
            Command::FinishSession { id, discard } => data.finish_session(&id, discard),
            Command::AddManual {
                source_id,
                label,
                ingredient_id,
                quantity,
                purchased,
            } => add_manual(
                data,
                ingredients,
                source_id,
                label,
                ingredient_id,
                quantity,
                purchased,
            ),
            Command::RemoveSource { id } => {
                data.sources
                    .iter_mut()
                    .find(|s| s.id == id)
                    .ok_or("shopping_source_not_found")?
                    .active = false;
                Ok(())
            }
            Command::Check { key, checked } => data.check(&key, checked),
            Command::AddRecipe { source_id, recipe } => sources::upsert(
                data,
                source_id,
                sources::recipe(recipes.ok_or("recipe_store_unavailable")?, &recipe)?,
            ),
            Command::SyncPlan {
                plan_revision,
                slots,
            } => sources::sync(
                data,
                recipes.ok_or("recipe_store_unavailable")?,
                plans.shopping_recipes(plan_revision, &slots)?,
            ),
        }?;
        if let Some(receipt) = receipt {
            data.mobile_receipts.retain(|r| {
                r.issued_at.saturating_add(super::mobile::RETENTION) >= receipt.recorded_at
            });
            data.mobile_receipts.push(receipt);
        }
        Ok(())
    })
}

fn add_manual(
    data: &mut super::model::Data,
    ingredients: &crate::ingredients::Store,
    source_id: Option<String>,
    label: String,
    ingredient_id: Option<String>,
    quantity: Quantity,
    purchased: bool,
) -> Result<(), &'static str> {
    if purchased && source_id.is_some() {
        return Err("shopping_request_invalid");
    }
    if !text(&label, 200) {
        return Err("shopping_label_invalid");
    }
    let (unit, amount) = quantity.normalized()?;
    let updating = source_id.is_some();
    let id = if let Some(id) = source_id {
        id
    } else {
        data.allocate("source")?
    };
    let (name, translations) = if let Some(ingredient) = &ingredient_id {
        let catalogue = ingredients.catalogue();
        let item = catalogue
            .ingredients
            .iter()
            .find(|i| &i.id == ingredient)
            .ok_or("unknown_ingredient")?;
        (
            item.name.clone(),
            item.translations
                .iter()
                .map(|(lang, tr)| (lang.clone(), tr.name.clone()))
                .collect(),
        )
    } else {
        (label.clone(), std::collections::BTreeMap::new())
    };
    let key = ingredient_id.as_ref().map_or_else(
        || format!("manual:{id}:{}", unit.key()),
        |id| format!("ingredient:{id}:{}", unit.key()),
    );
    let draft = Draft {
        origin: Origin::Manual { label, quantity },
        requirements: vec![Requirement {
            item: Item {
                key,
                ingredient_id,
                name,
                translations,
                unit,
            },
            amount,
        }],
    };
    if updating {
        sources::upsert(data, Some(id), draft)
    } else {
        if purchased {
            let requirement = draft
                .requirements
                .first()
                .ok_or("shopping_request_invalid")?;
            let purchase_id = data.allocate("purchase")?;
            data.purchases.push(super::model::Purchase {
                id: purchase_id,
                source_id: id.clone(),
                item: requirement.item.clone(),
                amount: requirement.amount.clone(),
                origin: draft.origin.clone(),
                reversed: false,
            });
            data.record_check(&requirement.item.key, true, true)?;
        }
        data.sources.push(super::model::Source {
            id,
            active: true,
            origin: draft.origin,
            requirements: draft.requirements,
        });
        Ok(())
    }
}
