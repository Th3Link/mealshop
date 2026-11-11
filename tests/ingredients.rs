//! Household catalogue isolation, persistence and conservative composite constraints.
#![allow(clippy::unwrap_used)]
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use tower::ServiceExt;
static NEXT: AtomicU64 = AtomicU64::new(0);
const TOKEN: &str = "writer_writer_writer_writer_writer_writer";
const READER: &str = "reader_reader_reader_reader_reader_reader";
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "mealshop-ingredients-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn app(&self) -> axum::Router {
        mealshop_server::family_router(
            include_str!("../data/catalogue.json"),
            self.0.join("recipes"),
            self.0.join("household"),
            TOKEN.to_owned(),
            Some(READER.to_owned()),
        )
        .unwrap()
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
async fn call(
    app: &axum::Router,
    method: &str,
    path: &str,
    value: Value,
    reader: bool,
) -> (StatusCode, Value) {
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header(
                    "authorization",
                    format!("Bearer {}", if reader { READER } else { TOKEN }),
                )
                .header("content-type", "application/json")
                .body(if method == "GET" {
                    Body::empty()
                } else {
                    Body::from(value.to_string())
                })
                .unwrap(),
        )
        .await
        .unwrap();
    let status = r.status();
    let bytes = to_bytes(r.into_body(), 3_000_000).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}
fn declaration() -> Value {
    json!({"expected_revision":0,"ingredient":{"name":"Family spice mix","language":"en","form":"Dry mixture","recorded_on":"2026-11-09","annotations":{"components":["oats"],"tags":["family mix"],"declared_allergens":["milk"]}}})
}
async fn raw(
    app: &axum::Router,
    method: &str,
    path: &str,
    mime: &str,
    data: Vec<u8>,
    reader: bool,
) -> (StatusCode, Vec<u8>) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header(
                    "authorization",
                    format!("Bearer {}", if reader { READER } else { TOKEN }),
                )
                .header("content-type", mime)
                .body(Body::from(data))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 6_000_000)
        .await
        .unwrap()
        .to_vec();
    (status, bytes)
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn declarations_archive_reverse_lookup_and_composite_constraints_survive_restart() {
    let dir = Directory::new();
    let app = dir.app();
    let path = "/api/household/ingredients";
    assert_eq!(
        call(&app, "POST", path, declaration(), true).await.0,
        StatusCode::FORBIDDEN
    );
    let (status, created) = call(&app, "POST", path, declaration(), false).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_str().unwrap();
    assert_eq!(
        call(&app, "POST", path, declaration(), false).await.0,
        StatusCode::CONFLICT
    );
    let list = call(&app, "GET", path, Value::Null, false).await.1;
    let custom = list["ingredients"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == id)
        .unwrap();
    assert_eq!(custom["source"]["kind"], "household_declaration");
    assert_eq!(custom["source"]["url"], "");
    assert_eq!(custom["nutrients"], json!({}));
    let recipe = json!({"title":"Composite test","language":"en","servings":1,"preparation":"as_recorded","lines":[{"ingredient_id":id,"grams":"25"}],"steps":[{"instruction":"Mix","active_seconds":1,"passive_seconds":0}]});
    let (status, saved) = call(&app, "POST", "/api/recipes", recipe, false).await;
    assert_eq!(status, StatusCode::CREATED);
    let reverse = call(
        &app,
        "GET",
        "/api/household/ingredients/oats/recipes",
        Value::Null,
        true,
    )
    .await
    .1;
    assert_eq!(reverse["recipes"].as_array().unwrap().len(), 1);
    let nutrient = saved["nutrition"]["nutrients"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == "energy_kcal")
        .unwrap();
    assert!(nutrient["total"].is_null());
    let update = json!({"expected_revision":1,"annotations":{"components":[id]}});
    assert_eq!(
        call(&app, "PUT", &format!("{path}/{id}"), update, false)
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let profile = json!({"expected_revision":0,"members":[{"id":"person","name":"Test person","allergies":[],"intolerances":[],"allergen_ids":["milk"],"excluded_ingredients":[],"disliked_ingredients":[]}]});
    assert_eq!(
        call(&app, "PUT", "/api/household/profiles", profile, false)
            .await
            .0,
        StatusCode::OK
    );
    let plan = json!({"date":"2026-11-09","meal":"dinner","recipe":{"id":saved["id"],"revision":1},"attendees":["person"],"calendar":null,"servings":1,"active_seconds":60,"elapsed_seconds":60,"advance_seconds":0});
    let preview = call(
        &app,
        "POST",
        "/api/household/mealplan/preview",
        plan.clone(),
        false,
    )
    .await
    .1;
    assert_eq!(preview["feasible"], false);
    // Removing current declarations must not erase the ingredient identity/allergens pinned by this recipe.
    assert_eq!(
        call(
            &app,
            "PUT",
            &format!("{path}/{id}"),
            json!({"expected_revision":1,"annotations":{"archived":true}}),
            false
        )
        .await
        .0,
        StatusCode::OK
    );
    let preview = call(
        &app,
        "POST",
        "/api/household/mealplan/preview",
        plan.clone(),
        false,
    )
    .await
    .1;
    assert_eq!(preview["feasible"], false);
    let fresh = json!({"title":"Archived ingredient","language":"en","servings":1,"preparation":"as_recorded","lines":[{"ingredient_id":id,"grams":"1"}],"steps":[]});
    assert_eq!(
        call(&app, "POST", "/api/recipes", fresh, false).await.0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    drop(app);
    let reopened = dir.app();
    let again = call(&reopened, "GET", path, Value::Null, true).await.1;
    assert_eq!(again["revision"], 2);
    assert_eq!(
        again["ingredients"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["id"] == id)
            .unwrap()["archived"],
        true
    );
    assert_eq!(
        call(
            &reopened,
            "POST",
            "/api/household/mealplan/preview",
            plan,
            false
        )
        .await
        .1["feasible"],
        false
    );
    let isolated = Directory::new();
    assert!(
        !call(&isolated.app(), "GET", path, Value::Null, false)
            .await
            .1["ingredients"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["id"] == id)
    );
}

#[tokio::test]
async fn jpeg_and_webp_uploads_retain_exact_private_bytes() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let dir = Directory::new();
    let app = dir.app();
    for (mime, encoded) in [
        (
            "image/jpeg",
            "/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAgGBgcGBQgHBwcJCQgKDBQNDAsLDBkSEw8UHRofHh0aHBwgJC4nICIsIxwcKDcpLDAxNDQ0Hyc5PTgyPC4zNDL/2wBDAQkJCQwLDBgNDRgyIRwhMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjL/wAARCAADAAIDASIAAhEBAxEB/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/8QAHwEAAwEBAQEBAQEBAQAAAAAAAAECAwQFBgcICQoL/8QAtREAAgECBAQDBAcFBAQAAQJ3AAECAxEEBSExBhJBUQdhcRMiMoEIFEKRobHBCSMzUvAVYnLRChYkNOEl8RcYGRomJygpKjU2Nzg5OkNERUZHSElKU1RVVldYWVpjZGVmZ2hpanN0dXZ3eHl6goOEhYaHiImKkpOUlZaXmJmaoqOkpaanqKmqsrO0tba3uLm6wsPExcbHyMnK0tPU1dbX2Nna4uPk5ebn6Onq8vP09fb3+Pn6/9oADAMBAAIRAxEAPwDmaKKK9M84/9k=",
        ),
        (
            "image/webp",
            "UklGRjQAAABXRUJQVlA4ICgAAACQAQCdASoCAAMAAUAmJYgCdLoAA5gA/vBTH+j64RZGX9CjZRJ1AAAA",
        ),
    ] {
        let image = STANDARD.decode(encoded).unwrap();
        let (status, value) = raw(
            &app,
            "POST",
            "/api/household/images",
            mime,
            image.clone(),
            false,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        let uploaded: Value = serde_json::from_slice(&value).unwrap();
        let path = format!("/api/household/images/{}", uploaded["id"].as_str().unwrap());
        assert_eq!(raw(&app, "GET", &path, "", vec![], true).await.1, image);
    }
}

#[tokio::test]
async fn private_images_reject_active_content_and_preserve_immutable_references() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let dir = Directory::new();
    let app = dir.app();
    let png=STANDARD.decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aL1sAAAAASUVORK5CYII=").unwrap();
    assert_eq!(
        raw(
            &app,
            "POST",
            "/api/household/images",
            "image/png",
            png.clone(),
            true
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        raw(
            &app,
            "POST",
            "/api/household/images",
            "image/svg+xml",
            b"<svg onload='alert(1)'/>".to_vec(),
            false
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let mut huge = png.clone();
    huge[16..20].copy_from_slice(&100_000_u32.to_be_bytes());
    assert_eq!(
        raw(
            &app,
            "POST",
            "/api/household/images",
            "image/png",
            huge,
            false
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let (status, result) = raw(
        &app,
        "POST",
        "/api/household/images",
        "image/png",
        png.clone(),
        false,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let value: Value = serde_json::from_slice(&result).unwrap();
    let id = value["id"].as_str().unwrap();
    let recipe = json!({"title":"With image","language":"en","image_id":id,"servings":1,"preparation":"as_recorded","lines":[{"ingredient_id":"oats","grams":"50"}],"steps":[]});
    let (status, saved) = call(&app, "POST", "/api/recipes", recipe.clone(), false).await;
    assert_eq!(status, StatusCode::CREATED);
    let summaries = call(&app, "GET", "/api/recipes", Value::Null, true).await.1;
    assert_eq!(summaries["recipes"][0]["image_id"], id);
    let mut missing = recipe;
    missing["image_id"] = json!("image-99999999.png");
    assert_eq!(
        call(&app, "POST", "/api/recipes", missing, false).await.0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    drop(app);
    let app = dir.app();
    let (status, bytes) = raw(
        &app,
        "GET",
        &format!("/api/household/images/{id}"),
        "",
        vec![],
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bytes, png);
    assert_eq!(
        call(
            &app,
            "GET",
            &format!("/api/recipes/{}", saved["id"].as_str().unwrap()),
            Value::Null,
            true
        )
        .await
        .1["recipe"]["image_id"],
        id
    );
}

#[tokio::test]
async fn explicit_person_tag_exclusions_block_nested_and_pinned_ingredients() {
    let dir = Directory::new();
    let app = dir.app();
    assert_eq!(
        call(
            &app,
            "PUT",
            "/api/household/ingredients/oats",
            json!({"expected_revision":0,"annotations":{"tags":["test-intolerance"]}}),
            false
        )
        .await
        .0,
        StatusCode::OK
    );
    let profile = json!({"expected_revision":0,"members":[{"id":"person","name":"Test person","allergies":[],"intolerances":[],"excluded_tags":["TEST-INTOLERANCE"],"excluded_ingredients":[],"disliked_ingredients":[]}]});
    assert_eq!(
        call(&app, "PUT", "/api/household/profiles", profile, false)
            .await
            .0,
        StatusCode::OK
    );
    let mut ingredient = declaration();
    ingredient["expected_revision"] = json!(1);
    let created = call(
        &app,
        "POST",
        "/api/household/ingredients",
        ingredient,
        false,
    )
    .await
    .1;
    let recipe = json!({"title":"Tagged components","language":"en","servings":1,"preparation":"as_recorded","lines":[{"ingredient_id":created["id"],"grams":"25"}],"steps":[{"instruction":"Mix","active_seconds":1,"passive_seconds":0}]});
    let saved = call(&app, "POST", "/api/recipes", recipe, false).await.1;
    assert_eq!(
        call(
            &app,
            "PUT",
            "/api/household/ingredients/oats",
            json!({"expected_revision":2,"annotations":{}}),
            false
        )
        .await
        .0,
        StatusCode::OK
    );
    let plan = json!({"date":"2026-11-09","meal":"lunch","recipe":{"id":saved["id"],"revision":1},"attendees":["person"],"calendar":null,"servings":1,"active_seconds":60,"elapsed_seconds":60,"advance_seconds":0});
    assert_eq!(
        call(&app, "POST", "/api/household/mealplan/preview", plan, false)
            .await
            .1["feasible"],
        false
    );
}

#[tokio::test]
async fn favorites_do_not_invalidate_plans_and_concurrent_constraints_never_gain_a_reviewed_version()
 {
    let dir = Directory::new();
    let app = dir.app();
    let recipe = json!({"title":"Concurrency fixture","language":"en","servings":1,"preparation":"as_recorded","lines":[{"ingredient_id":"oats","grams":"25"}],"steps":[{"instruction":"Mix","active_seconds":1,"passive_seconds":0}]});
    let saved = call(&app, "POST", "/api/recipes", recipe, false).await.1;
    let profiles = json!({"expected_revision":0,"members":[{"id":"person","name":"Test person","allergies":[],"intolerances":[],"excluded_tags":["blocked"],"excluded_ingredients":[],"disliked_ingredients":[]}]});
    assert_eq!(
        call(&app, "PUT", "/api/household/profiles", profiles, false)
            .await
            .0,
        StatusCode::OK
    );
    let plan = json!({"date":"2026-11-09","meal":"dinner","recipe":{"id":saved["id"],"revision":1},"attendees":["person"],"calendar":null,"servings":1,"active_seconds":60,"elapsed_seconds":60,"advance_seconds":0});
    let apply = json!({"plan":plan,"expected_revision":0,"profile_revision":1,"reviewed":true,"remove":false});
    assert_eq!(
        call(&app, "POST", "/api/household/mealplan/apply", apply, false)
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &app,
            "PUT",
            "/api/household/ingredients/oats",
            json!({"expected_revision":0,"annotations":{"favorite":true}}),
            false
        )
        .await
        .0,
        StatusCode::OK
    );
    let list = call(&app, "GET", "/api/household/mealplan", Value::Null, false)
        .await
        .1;
    assert_eq!(list["review_required_slots"], json!([]));
    let mut revision = 1;
    for _ in 0..12 {
        let current = call(&app, "GET", "/api/household/mealplan", Value::Null, false)
            .await
            .1;
        let apply = json!({"plan":plan,"expected_revision":current["revision"],"profile_revision":1,"reviewed":true,"remove":false});
        let annotations = json!({"expected_revision":revision,"annotations":{"favorite":true,"tags":["blocked"]}});
        let (applied, annotated) = tokio::join!(
            call(&app, "POST", "/api/household/mealplan/apply", apply, false),
            call(
                &app,
                "PUT",
                "/api/household/ingredients/oats",
                annotations,
                false
            ),
        );
        assert!(matches!(
            applied.0,
            StatusCode::OK | StatusCode::UNPROCESSABLE_ENTITY
        ));
        assert_eq!(annotated.0, StatusCode::OK);
        revision += 1;
        let current = call(&app, "GET", "/api/household/mealplan", Value::Null, false)
            .await
            .1;
        assert_eq!(
            current["review_required_slots"].as_array().unwrap().len(),
            1
        );
        assert_eq!(
            call(
                &app,
                "PUT",
                "/api/household/ingredients/oats",
                json!({"expected_revision":revision,"annotations":{"favorite":true}}),
                false
            )
            .await
            .0,
            StatusCode::OK
        );
        revision += 1;
    }
    drop(app);
    assert_legacy_overlay_loads(&dir, revision).await;
}

async fn assert_legacy_overlay_loads(dir: &Directory, revision: u64) {
    // Legacy overlays without the additive dietary counter must retain their original version.
    let path = dir.0.join("household/ingredients.json");
    let mut document: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    document.as_object_mut().unwrap().remove("dietary_revision");
    fs::write(path, serde_json::to_vec(&document).unwrap()).unwrap();
    let reopened = dir.app();
    assert_eq!(
        call(
            &reopened,
            "GET",
            "/api/household/ingredients",
            Value::Null,
            true
        )
        .await
        .1["revision"],
        revision
    );
    assert_eq!(
        call(
            &reopened,
            "GET",
            "/api/household/mealplan",
            Value::Null,
            true
        )
        .await
        .1["review_required_slots"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn subtype_nutrition_is_explicit_pinned_and_persistent() {
    let dir = Directory::new();
    let app = dir.app();
    let create = json!({"expected_revision":0,"ingredient":{"name":"Test flour group","language":"en","form":"Flour","recorded_on":"2026-11-09","annotations":{"organization":{"is_group":true,"diet":"vegan","origin":"plant","nutrition":{"mode":"own","note":"Test packaging declaration per 100 g","source_url":"","values":{"energy_kcal":{"amount":null,"range":{"lower":"300","upper":"350"},"uncertainty":"Synthetic declared range"},"protein":{"amount":"10.500000","uncertainty":"Declared label"},"fat":{"amount":"0","uncertainty":"Declared zero"}}}}}}});
    let (status, group) = call(&app, "POST", "/api/household/ingredients", create, false).await;
    assert_eq!(status, StatusCode::CREATED);
    let child = json!({"expected_revision":1,"ingredient":{"name":"Test Type 550","language":"en","form":"Flour","recorded_on":"2026-11-09","annotations":{"organization":{"group_id":group["id"],"nutrition":{"mode":"inherit"}}}}});
    let (status, child) = call(&app, "POST", "/api/household/ingredients", child, false).await;
    assert_eq!(status, StatusCode::CREATED);
    let recipe = json!({"title":"Inherited flour","language":"en","servings":2,"preparation":"as_recorded","lines":[{"ingredient_id":child["id"],"grams":"200"}],"steps":[]});
    let (status, saved) = call(&app, "POST", "/api/recipes", recipe, false).await;
    assert_eq!(status, StatusCode::CREATED);
    let nutrients = saved["nutrition"]["nutrients"].as_array().unwrap();
    assert_eq!(
        nutrients.iter().find(|n| n["id"] == "protein").unwrap()["total"],
        "21.00"
    );
    let energy = &nutrients.iter().find(|n| n["id"] == "energy_kcal").unwrap()["range"];
    assert_eq!(energy["total"], json!({"lower":"600.00","upper":"700.00"}));
    assert_eq!(
        energy["per_serving"],
        json!({"lower":"300.00","upper":"350.00"})
    );
    let update = json!({"expected_revision":2,"annotations":{"organization":{"is_group":true,"nutrition":{"mode":"own","note":"Changed label","source_url":"","values":{"protein":{"amount":"20","uncertainty":"Declared"}}}}}});
    assert_eq!(
        call(
            &app,
            "PUT",
            &format!(
                "/api/household/ingredients/{}",
                group["id"].as_str().unwrap()
            ),
            update,
            false
        )
        .await
        .0,
        StatusCode::OK
    );
    let id = saved["id"].as_str().unwrap();
    assert_eq!(
        call(
            &app,
            "GET",
            &format!("/api/recipes/{id}"),
            Value::Null,
            false
        )
        .await
        .1,
        saved
    );
    let invalid = json!({"expected_revision":3,"annotations":{"organization":{"is_group":true,"group_id":child["id"]}}});
    assert_eq!(
        call(
            &app,
            "PUT",
            &format!(
                "/api/household/ingredients/{}",
                group["id"].as_str().unwrap()
            ),
            invalid,
            false
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    drop(app);
    let reopened = dir.app();
    assert_eq!(
        call(
            &reopened,
            "GET",
            &format!("/api/recipes/{id}"),
            Value::Null,
            false
        )
        .await
        .1,
        saved
    );
    let list = call(
        &reopened,
        "GET",
        "/api/household/ingredients",
        Value::Null,
        true,
    )
    .await
    .1;
    let flour = list["ingredients"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == child["id"])
        .unwrap();
    assert_eq!(flour["nutrients"]["protein"]["amount"], "20");
    assert_eq!(flour["nutrition_inherited_from"], group["id"]);
}

#[tokio::test]
async fn inherited_household_profile_does_not_borrow_replaced_group_estimates() {
    let dir = Directory::new();
    let app = dir.app();
    let path = "/api/household/ingredients";
    let child = json!({"expected_revision":0,"ingredient":{"name":"Synthetic oat subtype","language":"en","form":"Dry oats","recorded_on":"2026-11-11","annotations":{"organization":{"group_id":"custom-group-oats","nutrition":{"mode":"inherit"}}}}});
    let (status, child) = call(&app, "POST", path, child, false).await;
    assert_eq!(status, StatusCode::CREATED);
    let reference = call(&app, "GET", path, Value::Null, true).await.1;
    let inherited = reference["ingredients"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == child["id"])
        .unwrap();
    assert_eq!(
        inherited["nutrition_estimates"][0]["method"],
        "group_envelope"
    );
    let update = json!({"expected_revision":1,"annotations":{"organization":{"is_group":true,"nutrition":{"mode":"own","note":"Synthetic household label replaces reference composition","source_url":"","values":{"protein":{"amount":"12","uncertainty":"Synthetic label"}}}}}});
    assert_eq!(
        call(
            &app,
            "PUT",
            &format!("{path}/custom-group-oats"),
            update,
            false
        )
        .await
        .0,
        StatusCode::OK
    );
    let recipe = json!({"title":"Explicit inherited declaration","language":"en","servings":1,"preparation":"as_recorded","lines":[{"ingredient_id":child["id"],"grams":"100"}],"steps":[]});
    let (status, saved) = call(&app, "POST", "/api/recipes", recipe, false).await;
    assert_eq!(status, StatusCode::CREATED);
    let nutrients = saved["nutrition"]["nutrients"].as_array().unwrap();
    let protein = nutrients.iter().find(|n| n["id"] == "protein").unwrap();
    assert_eq!(protein["total"], "12.00");
    let fat = nutrients.iter().find(|n| n["id"] == "fat").unwrap();
    assert!(
        fat["range"].is_null(),
        "An omitted household field must stay unknown"
    );
    assert!(fat["total"].is_null());
    drop(app);
    let reopened = dir.app();
    let list = call(&reopened, "GET", path, Value::Null, true).await.1;
    let inherited = list["ingredients"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == child["id"])
        .unwrap();
    assert_eq!(inherited["nutrients"]["protein"]["amount"], "12");
    assert!(
        inherited
            .get("nutrition_estimates")
            .is_none_or(|p| p == &json!([]))
    );
    let id = saved["id"].as_str().unwrap();
    assert_eq!(
        call(
            &reopened,
            "GET",
            &format!("/api/recipes/{id}"),
            Value::Null,
            true
        )
        .await
        .1,
        saved
    );
}

#[tokio::test]
async fn ingredient_profiles_reject_invalid_inputs_and_preserve_primary_sources() {
    let dir = Directory::new();
    let app = dir.app();
    let original = call(&app, "GET", "/api/household/ingredients", Value::Null, true)
        .await
        .1;
    let oats = original["ingredients"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == "oats")
        .unwrap();
    let profile = json!({"nutrition":{"mode":"own","note":"Explicit test label","source_url":"https://example.org/label","values":{"protein":{"amount":"1.250000","uncertainty":"Test label"},"salt":{"amount":"0","uncertainty":"Explicit zero"}}},"diet":"vegan","origin":"plant"});
    let update = json!({"expected_revision":0,"annotations":{"organization":profile}});
    assert_eq!(
        call(
            &app,
            "PUT",
            "/api/household/ingredients/oats",
            update.clone(),
            true
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    for invalid in [
        json!({"nutrition":{"mode":"inherit"}}),
        json!({"group_id":"missing"}),
        json!({"group_id":"oats"}),
        json!({"diet":"invented"}),
        json!({"nutrition":{"mode":"own","note":"Test","source_url":"","values":{"protein":{"amount":null,"range":{"lower":"2","upper":"1"},"uncertainty":"Reversed"}}}}),
        json!({"nutrition":{"mode":"own","note":"Test","source_url":"","values":{"protein":{"amount":"1","range":{"lower":"1","upper":"2"},"uncertainty":"Ambiguous"}}}}),
        json!({"nutrition":{"mode":"own","note":"Test","source_url":"","values":{"beta_glucan":{"amount":null,"range":{"lower":"1","upper":"2"},"uncertainty":"Unsupported constituent inference"}}}}),
        json!({"nutrition":{"mode":"own","note":"","source_url":"","values":{}}}),
        json!({"nutrition":{"mode":"own","note":"Label","source_url":"http://example.org","values":{}}}),
        json!({"nutrition":{"mode":"own","note":"Label","source_url":"","values":{"protein":{"amount":"-1","uncertainty":"Label"}}}}),
    ] {
        assert_eq!(
            call(
                &app,
                "PUT",
                "/api/household/ingredients/oats",
                json!({"expected_revision":0,"annotations":{"organization":invalid}}),
                false
            )
            .await
            .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    assert_eq!(
        call(
            &app,
            "PUT",
            "/api/household/ingredients/oats",
            update,
            false
        )
        .await
        .0,
        StatusCode::OK
    );
    // Older clients toggling favourites must not erase organization or effective values.
    assert_eq!(
        call(
            &app,
            "PUT",
            "/api/household/ingredients/oats",
            json!({"expected_revision":1,"annotations":{"favorite":true}}),
            false
        )
        .await
        .0,
        StatusCode::OK
    );
    let list = call(&app, "GET", "/api/household/ingredients", Value::Null, true)
        .await
        .1;
    let changed = list["ingredients"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == "oats")
        .unwrap();
    assert_eq!(changed["reference_source"], oats["source"]);
    assert_eq!(
        changed["source"]["retrieved_on"],
        "Not recorded for this household declaration"
    );
    assert_eq!(changed["nutrients"]["protein"]["amount"], "1.250000");
    assert_eq!(changed["nutrients"]["salt"]["amount"], "0");
    assert!(changed["nutrients"].get("calcium").is_none());
    assert_eq!(changed["organization"]["diet"], "vegan");
    assert_eq!(changed["favorite"], true);
}

#[tokio::test]
async fn ingredient_profiles_reject_duplicate_values() {
    let dir = Directory::new();
    let app = dir.app();
    let duplicate = br#"{"expected_revision":0,"annotations":{"organization":{"nutrition":{"mode":"own","note":"Label","source_url":"","values":{"protein":{"amount":"1","uncertainty":"Label"},"protein":{"amount":"2","uncertainty":"Label"}}}}}}"#;
    assert_eq!(
        raw(
            &app,
            "PUT",
            "/api/household/ingredients/oats",
            "application/json",
            duplicate.to_vec(),
            false
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn recipe_parts_preserve_associations_and_reject_cycles_and_unknown_references() {
    let dir = Directory::new();
    let app = dir.app();
    let recipe = json!({"title":"Branched preparation","language":"en","servings":2,"preparation":"as_recorded","parts":[{"id":"preparation","name":"Preparation"},{"id":"main","name":"Final assembly","uses_parts":["preparation"]}],"lines":[{"id":"line-a","part_id":"preparation","ingredient_id":"oats","grams":"100"},{"id":"line-b","part_id":"main","ingredient_id":"oats","grams":"50"}],"steps":[{"instruction":"Prepare","part_id":"preparation","ingredient_line_ids":["line-a"],"active_seconds":5,"passive_seconds":10},{"instruction":"Assemble","part_id":"main","ingredient_line_ids":["line-b"],"prepared_parts":["preparation"],"active_seconds":10,"passive_seconds":0}]});
    let (status, saved) = call(&app, "POST", "/api/recipes", recipe.clone(), false).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        saved["recipe"]["steps"][1]["prepared_parts"],
        json!(["preparation"])
    );
    let scale = call(
        &app,
        "GET",
        &format!(
            "/api/recipes/{}/scale?servings=4",
            saved["id"].as_str().unwrap()
        ),
        Value::Null,
        true,
    )
    .await
    .1;
    assert_eq!(scale["lines"][0]["id"], "line-a");
    assert_eq!(scale["lines"][0]["grams"], "200.000000");
    let protein = scale["nutrition"]["nutrients"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == "energy_kcal")
        .unwrap();
    assert_eq!(protein["total"], "1167.00");
    let mut invalid = Vec::new();
    let mut cycle = recipe.clone();
    cycle["parts"][0]["uses_parts"] = json!(["main"]);
    invalid.push(cycle);
    let mut step_cycle = recipe.clone();
    step_cycle["steps"][0]["prepared_parts"] = json!(["main"]);
    invalid.push(step_cycle);
    let mut unknown = recipe.clone();
    unknown["steps"][0]["ingredient_line_ids"] = json!(["missing"]);
    invalid.push(unknown);
    let mut duplicate = recipe.clone();
    duplicate["lines"][1]["id"] = json!("line-a");
    invalid.push(duplicate);
    let mut missing_part = recipe.clone();
    missing_part["lines"][1]["part_id"] = json!("absent");
    invalid.push(missing_part);
    for item in invalid {
        assert_eq!(
            call(&app, "POST", "/api/recipes", item, false).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    drop(app);
    let reopened = dir.app();
    assert_eq!(
        call(
            &reopened,
            "GET",
            &format!("/api/recipes/{}", saved["id"].as_str().unwrap()),
            Value::Null,
            true
        )
        .await
        .1,
        saved
    );
}

#[tokio::test]
async fn optional_constituents_aggregate_exactly_and_sorbitol_blocks_pinned_recipes() {
    let dir = Directory::new();
    let app = dir.app();
    let update = json!({"expected_revision":0,"annotations":{"organization":{"intolerance_triggers":["sorbitol"],"nutrition":{"mode":"own","note":"Synthetic test packaging","source_url":"","values":{},"constituents":{"beta_glucan":{"name":"Beta-glucan","unit":"g","amount":"4"},"custom-compound":{"name":"Test constituent","unit":"mg","amount":"2.500000"}}}}}});
    assert_eq!(
        call(
            &app,
            "PUT",
            "/api/household/ingredients/oats",
            update,
            false
        )
        .await
        .0,
        StatusCode::OK
    );
    let recipe = json!({"title":"Declared constituents","language":"en","servings":2,"preparation":"as_recorded","lines":[{"ingredient_id":"oats","grams":"150"}],"steps":[]});
    let (status, saved) = call(&app, "POST", "/api/recipes", recipe, false).await;
    assert_eq!(status, StatusCode::CREATED);
    let constituents = saved["nutrition"]["constituents"].as_array().unwrap();
    let beta = constituents
        .iter()
        .find(|c| c["id"] == "beta_glucan")
        .unwrap();
    assert_eq!(beta["total"], "6.00");
    assert_eq!(beta["per_serving"], "3.00");
    let compound = constituents
        .iter()
        .find(|c| c["id"] == "custom-compound")
        .unwrap();
    assert_eq!(compound["total"], "3.75");
    assert_eq!(compound["unit"], "mg");
    assert_eq!(compound["per_serving"], "1.88");
    let profiles = json!({"expected_revision":0,"members":[{"id":"person","name":"Test person","allergies":[],"intolerances":["sorbitol"],"excluded_ingredients":[],"disliked_ingredients":[]}]});
    assert_eq!(
        call(&app, "PUT", "/api/household/profiles", profiles, false)
            .await
            .0,
        StatusCode::OK
    );
    // Removing a live declaration does not remove the saved recipe's constraint evidence.
    assert_eq!(
        call(
            &app,
            "PUT",
            "/api/household/ingredients/oats",
            json!({"expected_revision":1,"annotations":{"organization":{"intolerance_triggers":[]}}}),
            false
        )
        .await
        .0,
        StatusCode::OK
    );
    let plan = json!({"date":"2026-11-09","meal":"lunch","recipe":{"id":saved["id"],"revision":1},"attendees":["person"],"calendar":null,"servings":2,"active_seconds":60,"elapsed_seconds":60,"advance_seconds":0});
    assert_eq!(
        call(&app, "POST", "/api/household/mealplan/preview", plan, false)
            .await
            .1["feasible"],
        false
    );
    drop(app);
    let reopened = dir.app();
    assert_eq!(
        call(
            &reopened,
            "GET",
            &format!("/api/recipes/{}", saved["id"].as_str().unwrap()),
            Value::Null,
            true
        )
        .await
        .1,
        saved
    );
}

#[tokio::test]
async fn constituent_identity_units_unknowns_and_legacy_values_remain_consistent() {
    let dir = Directory::new();
    let app = dir.app();
    let annotation = json!({"organization":{"nutrition":{"mode":"own","note":"Test evidence","source_url":"","values":{},"constituents":{"test-compound":{"name":"Test compound","unit":"mg","amount":"1"},"beta_glucan":{"name":"Beta-glucan","unit":"g","amount":"2"}}}}});
    assert_eq!(
        call(
            &app,
            "PUT",
            "/api/household/ingredients/oats",
            json!({"expected_revision":0,"annotations":annotation}),
            false
        )
        .await
        .0,
        StatusCode::OK
    );
    let (status,record)=call(&app,"POST","/api/recipes",json!({"title":"Partial constituents","language":"en","steps":[],"lines":[{"ingredient_id":"oats","grams":"100"},{"ingredient_id":"wheat-flour-type-405","grams":"100"}],"servings":2,"preparation":"as_recorded"}),false).await;
    assert_eq!(status, StatusCode::CREATED);
    let partial = &record["nutrition"];
    let beta = partial["constituents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "beta_glucan")
        .unwrap();
    assert_eq!(beta["total"], Value::Null);
    assert_eq!(beta["known_total"], "2.00");
    assert_eq!(beta["status"], "partial");
    let legacy = partial["nutrients"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "beta_glucan")
        .unwrap();
    assert_eq!(legacy["known_total"], "2.00");
    let conflict = json!({"expected_revision":1,"annotations":{"organization":{"nutrition":{"mode":"own","note":"Test evidence","source_url":"","values":{},"constituents":{"test-compound":{"name":"Test compound","unit":"g","amount":"1"}}}}}});
    assert_eq!(
        call(
            &app,
            "PUT",
            "/api/household/ingredients/wheat-flour-type-405",
            conflict,
            false
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let duplicate=br#"{"expected_revision":1,"annotations":{"organization":{"nutrition":{"mode":"own","note":"Test","source_url":"","values":{},"constituents":{"x":{"name":"Test","unit":"g","amount":"1"},"x":{"name":"Test","unit":"g","amount":"2"}}}}}}"#;
    assert_eq!(
        raw(
            &app,
            "PUT",
            "/api/household/ingredients/oats",
            "application/json",
            duplicate.to_vec(),
            false
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn older_organization_writes_preserve_unknown_constituents_and_intolerance_fields() {
    let dir = Directory::new();
    let app = dir.app();
    let profile = json!({"mode":"own","values":{},"note":"Test evidence","source_url":"","constituents":{"beta_glucan":{"name":"Beta-glucan","unit":"g","amount":"2"}}});
    assert_eq!(call(&app,"PUT","/api/household/ingredients/oats",json!({"expected_revision":0,"annotations":{"organization":{"intolerance_triggers":["sorbitol"],"nutrition":profile}}}),false).await.0,StatusCode::OK);
    let legacy = json!({"expected_revision":1,"annotations":{"favorite":true,"organization":{"nutrition":{"mode":"own","values":{},"note":"Test evidence","source_url":""}}}});
    assert_eq!(
        call(
            &app,
            "PUT",
            "/api/household/ingredients/oats",
            legacy,
            false
        )
        .await
        .0,
        StatusCode::OK
    );
    let catalogue = call(&app, "GET", "/api/household/ingredients", Value::Null, true)
        .await
        .1;
    let oats = catalogue["ingredients"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == "oats")
        .unwrap();
    assert_eq!(
        oats["organization"]["intolerance_triggers"],
        json!(["sorbitol"])
    );
    assert_eq!(oats["constituents"]["beta_glucan"]["amount"], "2");
    assert_eq!(oats["favorite"], true);
}

#[tokio::test]
async fn organizational_reverse_lookup_finds_legacy_recipes_without_rewriting_snapshots() {
    let dir = Directory::new();
    let app = dir.app();
    let (status,saved)=call(&app,"POST","/api/recipes",json!({"title":"Legacy oats","language":"en","servings":1,"preparation":"as_recorded","lines":[{"ingredient_id":"oats","grams":"100"}],"steps":[]}),false).await;
    assert_eq!(status, StatusCode::CREATED);
    drop(app);
    let path = dir
        .0
        .join("recipes")
        .join(format!("{}-1.json", saved["id"].as_str().unwrap()));
    let mut legacy: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let records = legacy["catalogue"]["ingredients"].as_array_mut().unwrap();
    records.retain(|i| i["organization"]["is_group"] != true);
    for item in records {
        item["organization"]["group_id"] = Value::Null;
    }
    let bytes = serde_json::to_vec(&legacy).unwrap();
    fs::write(&path, &bytes).unwrap();
    let reopened = dir.app();
    let (status, found) = call(
        &reopened,
        "GET",
        "/api/household/ingredients/custom-group-oats/recipes",
        Value::Null,
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(found["recipes"][0]["id"], saved["id"]);
    assert_eq!(fs::read(path).unwrap(), bytes);
}
