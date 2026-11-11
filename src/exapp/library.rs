//! Current household authorization around revision-checked private collections.
use super::{Failure, Shared, household_router, input, user};
use crate::{
    library::{Draft, HomeLibrary, RecipeRef, Resource, Store},
    membership::Role,
};
use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::{Path, Query, Request, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Deserialize;
use serde_json::{Value, json};
use tower::ServiceExt;

pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/api/households/{home}/collections", get(list).post(create))
        .route(
            "/api/households/{home}/collections/{id}",
            get(detail).put(update).delete(remove),
        )
}
fn failure(code: &'static str) -> Failure {
    let status = match code {
        "household_access_denied" | "household_read_only" | "collection_owner_required" => {
            StatusCode::FORBIDDEN
        }
        "collection_not_found" => StatusCode::NOT_FOUND,
        "revision_conflict" => StatusCode::CONFLICT,
        "library_storage_failed" => StatusCode::SERVICE_UNAVAILABLE,
        "library_capacity" => StatusCode::INSUFFICIENT_STORAGE,
        _ => StatusCode::UNPROCESSABLE_ENTITY,
    };
    Failure(status, code)
}
fn response(value: Value) -> Response {
    ([(header::CACHE_CONTROL, "no-store")], Json(value)).into_response()
}
async fn access<T: Send + 'static>(
    app: Shared,
    home: String,
    actor: String,
    write: bool,
    action: impl FnOnce(&mut Store, &str, Role) -> Result<T, &'static str> + Send + 'static,
) -> Result<T, Failure> {
    tokio::task::spawn_blocking(move || {
        let membership = app
            .membership
            .lock()
            .map_err(|_| "library_storage_failed")?;
        let role = membership
            .role(&home, &actor)
            .ok_or("household_access_denied")?;
        if write && role == Role::Reader {
            return Err("household_read_only");
        }
        let mut library = app.library.lock().map_err(|_| "library_storage_failed")?;
        action(&mut library, &home, role)
    })
    .await
    .map_err(|_| failure("library_storage_failed"))?
    .map_err(failure)
}
fn editable(data: &HomeLibrary, id: Option<&str>, role: Role) -> bool {
    role != Role::Reader
        && (role == Role::Owner
            || id.is_none_or(|id| {
                !data.shares.values().any(|share| {
                    matches!(&share.resource, Resource::Collection {collection_id} if collection_id == id)
                })
            }))
}
#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ListingQuery {
    page: usize,
    per_page: usize,
    q: String,
}
impl Default for ListingQuery {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
            q: String::new(),
        }
    }
}
impl ListingQuery {
    fn from(request: &Request) -> Result<Self, Failure> {
        let Query(page) = Query::<Self>::try_from_uri(request.uri())
            .map_err(|_| Failure(StatusCode::BAD_REQUEST, "invalid_page"))?;
        if page.page == 0
            || !(1..=50).contains(&page.per_page)
            || page.page.checked_mul(page.per_page).is_none()
            || page.q.len() > 200
        {
            return Err(Failure(StatusCode::BAD_REQUEST, "invalid_page"));
        }
        Ok(page)
    }
    fn offset(&self) -> usize {
        (self.page - 1) * self.per_page
    }
}
async fn list(
    State(app): State<Shared>,
    Path(home): Path<String>,
    request: Request,
) -> Result<Response, Failure> {
    let actor = user(&app, &request)?;
    let page = ListingQuery::from(&request)?;
    let (data, role) = access(app, home, actor, false, |s, h, role| Ok((s.home(h), role))).await?;
    let query = page.q.trim().to_lowercase();
    let matching: Vec<_> = data
        .collections
        .values()
        .filter(|c| {
            c.name.to_lowercase().contains(&query) || c.description.to_lowercase().contains(&query)
        })
        .collect();
    let total = matching.len();
    let collections: Vec<_> = matching.into_iter().skip(page.offset()).take(page.per_page)
        .map(|c| json!({"id":c.id,"name":c.name,"description":c.description,"count":c.entries.len(),"editable":editable(&data,Some(&c.id),role)})).collect();
    Ok(response(
        json!({"revision":data.revision,"collections":collections,"page":page.page,"per_page":page.per_page,"total":total}),
    ))
}
pub(super) async fn source(
    app: Shared,
    home: String,
    reference: &RecipeRef,
) -> Result<Value, Failure> {
    if !crate::recipes::valid_id(&reference.recipe_id) || reference.revision == 0 {
        return Err(failure("invalid_collection_reference"));
    }
    let router = household_router(app.clone(), home)
        .await
        .map_err(|_| failure("library_storage_failed"))?;
    let request = Request::builder()
        .uri(format!(
            "/api/recipes/{}?revision={}",
            reference.recipe_id, reference.revision
        ))
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
    if result.status() == StatusCode::NOT_FOUND {
        return Err(failure("invalid_collection_reference"));
    }
    if !result.status().is_success() {
        return Err(failure("library_storage_failed"));
    }
    let bytes = to_bytes(result.into_body(), 2_097_152)
        .await
        .map_err(|_| failure("library_storage_failed"))?;
    serde_json::from_slice(&bytes).map_err(|_| failure("library_storage_failed"))
}
async fn detail(
    State(app): State<Shared>,
    Path((home, id)): Path<(String, String)>,
    request: Request,
) -> Result<Response, Failure> {
    let actor = user(&app, &request)?;
    let page = ListingQuery::from(&request)?;
    let selected = id.clone();
    let (revision, collection, can_edit) = access(
        app.clone(),
        home.clone(),
        actor.clone(),
        false,
        move |s, h, role| {
            let data = s.home(h);
            let collection = data
                .collections
                .get(&selected)
                .cloned()
                .ok_or("collection_not_found")?;
            let can_edit = editable(&data, Some(&selected), role);
            Ok((data.revision, collection, can_edit))
        },
    )
    .await?;
    let mut items = Vec::new();
    for reference in collection
        .entries
        .iter()
        .skip(page.offset())
        .take(page.per_page)
    {
        let record = source(app.clone(), home.clone(), reference).await?;
        let recipe = &record["recipe"];
        items.push(json!({"recipe_id":reference.recipe_id,"revision":reference.revision,"title":recipe["title"],"language":recipe["language"],"servings":recipe["servings"],"image_id":recipe.get("image_id")}));
    }
    access(app, home, actor, false, |_, _, _| Ok(())).await?;
    Ok(response(
        json!({"revision":revision,"collection":collection,"editable":can_edit,"items":items,"page":page.page,"per_page":page.per_page}),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Save {
    expected_revision: u64,
    collection: Draft,
}
async fn save(
    app: Shared,
    home: String,
    id: Option<String>,
    request: Request,
) -> Result<Response, Failure> {
    let actor = user(&app, &request)?;
    let checking = id.clone();
    let previous = access(
        app.clone(),
        home.clone(),
        actor.clone(),
        true,
        move |s, h, role| {
            let data = s.home(h);
            if !editable(&data, checking.as_deref(), role) {
                return Err("collection_owner_required");
            }
            Ok(data)
        },
    )
    .await?;
    let body: Save = input(request).await?;
    body.collection.validate().map_err(failure)?;
    if previous.revision != body.expected_revision {
        return Err(failure("revision_conflict"));
    }
    if id
        .as_ref()
        .is_some_and(|id| !previous.collections.contains_key(id))
    {
        return Err(failure("collection_not_found"));
    }
    let existing = id.as_ref().and_then(|id| previous.collections.get(id));
    for reference in &body.collection.entries {
        if existing.is_none_or(|c| !c.entries.contains(reference)) {
            source(app.clone(), home.clone(), reference).await?;
        }
    }
    let value = access(app, home, actor, true, move |s, h, role| {
        if !editable(&s.home(h), id.as_deref(), role) {
            return Err("collection_owner_required");
        }
        let id = s.save(h, body.expected_revision, id.as_deref(), body.collection)?;
        Ok(json!({"id":id,"revision":s.home(h).revision}))
    })
    .await?;
    Ok(response(value))
}
async fn create(
    State(app): State<Shared>,
    Path(home): Path<String>,
    request: Request,
) -> Result<Response, Failure> {
    save(app, home, None, request).await
}
async fn update(
    State(app): State<Shared>,
    Path((home, id)): Path<(String, String)>,
    request: Request,
) -> Result<Response, Failure> {
    save(app, home, Some(id), request).await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Revision {
    expected_revision: u64,
}
async fn remove(
    State(app): State<Shared>,
    Path((home, id)): Path<(String, String)>,
    request: Request,
) -> Result<Response, Failure> {
    let actor = user(&app, &request)?;
    access(app.clone(), home.clone(), actor.clone(), true, |_, _, _| {
        Ok(())
    })
    .await?;
    let body: Revision = input(request).await?;
    let value = access(app, home, actor, true, move |s, h, role| {
        if !editable(&s.home(h), Some(&id), role) {
            return Err("collection_owner_required");
        }
        s.remove(h, body.expected_revision, &id)?;
        Ok(json!({"revision":s.home(h).revision}))
    })
    .await?;
    Ok(response(value))
}
