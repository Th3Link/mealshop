use super::{
    Failure, Shared,
    access::{self, Context, locked},
    failure, response,
};
use crate::{
    exapp::{household_router, user},
    library::RecipeRef,
    recipe_origin::Origin,
};
use axum::{
    body::{Body, to_bytes},
    extract::{Path, Query, Request, State},
    http::{StatusCode, header},
    response::Response,
};
use serde::Deserialize;
use serde_json::{Value, json};
use tower::ServiceExt;

#[derive(Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Unit {
    #[default]
    G,
    Kg,
}
#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct ReadQuery {
    page: usize,
    per_page: usize,
    revision: Option<u64>,
    servings: Option<u32>,
    display_unit: Unit,
}
impl Default for ReadQuery {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
            revision: None,
            servings: None,
            display_unit: Unit::G,
        }
    }
}
impl ReadQuery {
    fn parse(request: &Request) -> Result<Self, Failure> {
        let Query(query) = Query::<Self>::try_from_uri(request.uri())
            .map_err(|_| Failure(StatusCode::BAD_REQUEST, "invalid_page"))?;
        if query.page == 0
            || !(1..=50).contains(&query.per_page)
            || query.page.checked_mul(query.per_page).is_none()
            || query.servings.is_some_and(|n| n == 0 || n > 1_000_000)
        {
            return Err(Failure(StatusCode::BAD_REQUEST, "invalid_page"));
        }
        Ok(query)
    }
    fn offset(&self) -> usize {
        (self.page - 1) * self.per_page
    }
}
fn metadata(context: &Context) -> Value {
    json!({"id":context.grant.id,"source_home":context.home,"version":context.grant.version,"owner":context.owner,
        "recipient":context.grant.recipient,"recipient_label":context.recipient_label,"pending":context.pending,
        "resource_kind":if context.collection.is_some(){"collection"}else{"recipe"}})
}
fn origin(record: &Value) -> Result<Origin, Failure> {
    serde_json::from_value(
        record
            .get("sharing_origin")
            .or_else(|| record["recipe"].get("origin"))
            .cloned()
            .unwrap_or_else(|| json!({"kind":"owned"})),
    )
    .map_err(|_| failure("library_storage_failed"))
}
fn summary(record: &Value) -> Result<Value, Failure> {
    match origin(record)? {
        Origin::Imported { source_url } => {
            Ok(json!({"kind":"source_link","source_url":source_url}))
        }
        Origin::Owned => Ok(
            json!({"kind":"recipe","title":record["recipe"]["title"],"language":record["recipe"]["language"],"servings":record["recipe"]["servings"],"revision":record["revision"],"has_image":record["recipe"].get("image_id").is_some_and(Value::is_string)}),
        ),
    }
}
async fn fetch(app: Shared, home: String, path: String) -> Result<Response, Failure> {
    let router = household_router(app.clone(), home)
        .await
        .map_err(|_| failure("library_storage_failed"))?;
    let request = Request::builder()
        .uri(path)
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", app.config.secret),
        )
        .body(Body::empty())
        .map_err(|_| failure("library_storage_failed"))?;
    let result = router
        .oneshot(request)
        .await
        .map_err(|_| failure("library_storage_failed"))?;
    if !result.status().is_success() {
        return Err(failure("share_not_found"));
    }
    Ok(result)
}
async fn projection(
    app: Shared,
    home: String,
    reference: &RecipeRef,
    query: &ReadQuery,
) -> Result<Value, Failure> {
    if query.revision.is_some_and(|r| r != reference.revision) {
        return Err(failure("share_not_found"));
    }
    let record = crate::exapp::library::source(app.clone(), home.clone(), reference).await?;
    if let Origin::Imported { source_url } = origin(&record)? {
        return Ok(json!({"kind":"source_link","source_url":source_url}));
    }
    let scaled = if query.servings.is_some() || matches!(query.display_unit, Unit::Kg) {
        let original = record["recipe"]["servings"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| failure("library_storage_failed"))?;
        let servings = query.servings.unwrap_or(original);
        let unit = match query.display_unit {
            Unit::G => "g",
            Unit::Kg => "kg",
        };
        let path = format!(
            "/api/recipes/{}/scale?revision={}&servings={servings}&display_unit={unit}",
            reference.recipe_id, reference.revision
        );
        let result = fetch(app, home, path).await?;
        let bytes = to_bytes(result.into_body(), 2_097_152)
            .await
            .map_err(|_| failure("library_storage_failed"))?;
        Some(
            serde_json::from_slice::<Value>(&bytes)
                .map_err(|_| failure("library_storage_failed"))?,
        )
    } else {
        None
    };
    Ok(json!({"kind":"recipe","record":record,"scaled":scaled}))
}
async fn selected(
    app: Shared,
    home: String,
    id: String,
    actor: String,
) -> Result<Context, Failure> {
    locked(app, move |m, s| {
        access::context(m, s, &home, &id, &actor, false)
    })
    .await
}
fn entry_reference(
    context: &Context,
    id: &str,
    revision: Option<u64>,
) -> Result<RecipeRef, Failure> {
    context
        .collection
        .as_ref()
        .and_then(|c| {
            c.entries
                .iter()
                .find(|r| r.recipe_id == id && revision.is_none_or(|n| n == r.revision))
        })
        .cloned()
        .ok_or_else(|| failure("share_not_found"))
}
pub(super) async fn list(State(app): State<Shared>, request: Request) -> Result<Response, Failure> {
    let actor = user(&app, &request)?;
    let query = ReadQuery::parse(&request)?;
    let whom = actor.clone();
    let all = locked(app.clone(), move |m, s| {
        let mut result = Vec::new();
        for (home, data) in s.homes() {
            for grant in data.shares.values() {
                if access::addressed(m, &grant.recipient, &whom) {
                    result.push(access::context(m, s, home, &grant.id, &whom, true)?);
                }
            }
        }
        Ok(result)
    })
    .await?;
    let mut items = Vec::new();
    for context in all.iter().skip(query.offset()).take(query.per_page) {
        let payload = if context.pending {
            None
        } else if let Some(collection) = &context.collection {
            Some(
                json!({"kind":"collection","name":collection.name,"description":collection.description,"count":collection.entries.len()}),
            )
        } else {
            let reference = context
                .grant
                .resource
                .reference()
                .ok_or_else(|| failure("share_not_found"))?;
            Some(summary(
                &crate::exapp::library::source(app.clone(), context.home.clone(), &reference)
                    .await?,
            )?)
        };
        if !context.pending {
            access::ensure(app.clone(), context.clone(), actor.clone()).await?;
        }
        items.push(json!({"share":metadata(context),"content":payload}));
    }
    Ok(response(
        json!({"items":items,"total":all.len(),"page":query.page,"per_page":query.per_page}),
    ))
}
pub(super) async fn detail(
    State(app): State<Shared>,
    Path((home, id)): Path<(String, String)>,
    request: Request,
) -> Result<Response, Failure> {
    let actor = user(&app, &request)?;
    let query = ReadQuery::parse(&request)?;
    let context = selected(app.clone(), home, id, actor.clone()).await?;
    let payload = if let Some(collection) = &context.collection {
        let mut items = Vec::new();
        for reference in collection
            .entries
            .iter()
            .skip(query.offset())
            .take(query.per_page)
        {
            let record =
                crate::exapp::library::source(app.clone(), context.home.clone(), reference).await?;
            items.push(json!({"recipe_id":reference.recipe_id,"revision":reference.revision,"content":summary(&record)?}));
        }
        json!({"kind":"collection","name":collection.name,"description":collection.description,"count":collection.entries.len(),"items":items,"page":query.page,"per_page":query.per_page})
    } else {
        let reference = context
            .grant
            .resource
            .reference()
            .ok_or_else(|| failure("share_not_found"))?;
        projection(app.clone(), context.home.clone(), &reference, &query).await?
    };
    access::ensure(app, context.clone(), actor).await?;
    Ok(response(
        json!({"share":metadata(&context),"content":payload}),
    ))
}
pub(super) async fn entry(
    State(app): State<Shared>,
    Path((home, id, recipe)): Path<(String, String, String)>,
    request: Request,
) -> Result<Response, Failure> {
    let actor = user(&app, &request)?;
    let query = ReadQuery::parse(&request)?;
    let context = selected(app.clone(), home, id, actor.clone()).await?;
    let reference = entry_reference(&context, &recipe, query.revision)?;
    let payload = projection(app.clone(), context.home.clone(), &reference, &query).await?;
    access::ensure(app, context.clone(), actor).await?;
    Ok(response(
        json!({"share":metadata(&context),"content":payload}),
    ))
}
async fn image_response(
    app: Shared,
    context: Context,
    actor: String,
    reference: RecipeRef,
) -> Result<Response, Failure> {
    let record =
        crate::exapp::library::source(app.clone(), context.home.clone(), &reference).await?;
    if !origin(&record)?.is_owned() {
        return Err(failure("share_not_found"));
    }
    let id = record["recipe"]["image_id"]
        .as_str()
        .ok_or_else(|| failure("share_not_found"))?;
    if !crate::recipe_images::valid_id(id) {
        return Err(failure("share_not_found"));
    }
    let result = fetch(
        app.clone(),
        context.home.clone(),
        format!("/api/household/images/{id}"),
    )
    .await?;
    access::ensure(app, context, actor).await?;
    Ok(result)
}
pub(super) async fn image(
    State(app): State<Shared>,
    Path((home, id)): Path<(String, String)>,
    request: Request,
) -> Result<Response, Failure> {
    let actor = user(&app, &request)?;
    let query = ReadQuery::parse(&request)?;
    let context = selected(app.clone(), home, id, actor.clone()).await?;
    let reference = context
        .grant
        .resource
        .reference()
        .ok_or_else(|| failure("share_not_found"))?;
    if query.revision.is_some_and(|n| n != reference.revision) {
        return Err(failure("share_not_found"));
    }
    image_response(app, context, actor, reference).await
}
pub(super) async fn entry_image(
    State(app): State<Shared>,
    Path((home, id, recipe)): Path<(String, String, String)>,
    request: Request,
) -> Result<Response, Failure> {
    let actor = user(&app, &request)?;
    let query = ReadQuery::parse(&request)?;
    let context = selected(app.clone(), home, id, actor.clone()).await?;
    let reference = entry_reference(&context, &recipe, query.revision)?;
    image_response(app, context, actor, reference).await
}
