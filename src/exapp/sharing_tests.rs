use super::*;
use super::{
    library_tests::{Fixture, home, join, recipe},
    tests::call,
};

pub(super) async fn saved_recipe(app: &Router, home: &str, actor: &str) -> Value {
    let (status, saved) = call(
        app,
        Some(actor),
        "POST",
        &format!("/api/households/{home}/domain/api/recipes"),
        recipe(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    saved
}
pub(super) async fn grant(
    app: &Router,
    home: &str,
    revision: u64,
    resource: Value,
    recipient: Value,
) -> Value {
    let (status, value) = call(
        app,
        Some("alice"),
        "POST",
        &format!("/api/households/{home}/shares"),
        json!({"expected_revision":revision,"resource":resource,"recipient":recipient}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    value
}
async fn accept_household(app: &Router, shared: &str, home: &str) -> StatusCode {
    call(
        app,
        Some("bob"),
        "PUT",
        &format!("{shared}/accept"),
        json!({"expected_version":1,"household_id":home}),
    )
    .await
    .0
}
#[tokio::test]
async fn personal_grants_pin_publish_scale_and_revoke_without_household_access() {
    let fixture = Fixture::new().await;
    let app = fixture.app().await;
    let home = home(&app, "alice").await;
    let saved = saved_recipe(&app, &home, "alice").await;
    let resource = json!({"kind":"recipe","recipe_id":saved["id"],"revision":1});
    let created = grant(
        &app,
        &home,
        0,
        resource,
        json!({"kind":"account","user":"bob"}),
    )
    .await;
    let shared = format!("/api/shared/{home}/{}", created["id"].as_str().unwrap());
    assert_eq!(
        call(&app, Some("mallory"), "GET", &shared, json!({}))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    let first = call(&app, Some("bob"), "GET", &shared, json!({})).await.1;
    assert_eq!(first["content"]["record"]["recipe"], saved["recipe"]);
    let scaled = call(
        &app,
        Some("bob"),
        "GET",
        &format!("{shared}?servings=3"),
        json!({}),
    )
    .await
    .1;
    assert_eq!(
        scaled["content"]["scaled"]["lines"][0]["grams"],
        "150.750000"
    );
    assert_eq!(
        call(
            &app,
            Some("bob"),
            "GET",
            &format!("/api/households/{home}/domain/api/household/profiles"),
            json!({})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let mut changed = recipe();
    changed["title"] = json!("Explicit new shared revision");
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "PUT",
            &format!(
                "/api/households/{home}/domain/api/recipes/{}",
                saved["id"].as_str().unwrap()
            ),
            json!({"expected_revision":1,"recipe":changed})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, Some("bob"), "GET", &shared, json!({})).await.1,
        first
    );
    let outgoing = format!(
        "/api/households/{home}/shares/{}",
        created["id"].as_str().unwrap()
    );
    assert_eq!(call(&app,Some("alice"),"PUT",&outgoing,json!({"expected_revision":1,"resource":{"kind":"recipe","recipe_id":saved["id"],"revision":2}})).await.0,StatusCode::OK);
    assert_eq!(
        call(&app, Some("bob"), "GET", &shared, json!({})).await.1["content"]["record"]["revision"],
        2
    );
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "DELETE",
            &outgoing,
            json!({"expected_revision":2})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, Some("bob"), "GET", &shared, json!({})).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(&app, Some("bob"), "GET", "/api/shared", json!({}))
            .await
            .1["total"],
        0
    );
}
#[tokio::test]
async fn household_invitation_requires_destination_owner_and_checks_current_membership() {
    let fixture = Fixture::new().await;
    let app = fixture.app().await;
    let source = home(&app, "alice").await;
    let target = home(&app, "bob").await;
    let base = format!("/api/households/{target}");
    join(&app, &base, "bob", "carol", "reader").await;
    let saved = saved_recipe(&app, &source, "alice").await;
    let created = grant(
        &app,
        &source,
        0,
        json!({"kind":"recipe","recipe_id":saved["id"],"revision":1}),
        json!({"kind":"household_invitation","user":"bob"}),
    )
    .await;
    let shared = format!("/api/shared/{source}/{}", created["id"].as_str().unwrap());
    assert_eq!(
        call(&app, Some("bob"), "GET", &shared, json!({})).await.0,
        StatusCode::NOT_FOUND
    );
    let pending = call(&app, Some("bob"), "GET", "/api/shared", json!({}))
        .await
        .1;
    assert_eq!(pending["items"][0]["share"]["pending"], true);
    assert_eq!(pending["items"][0]["content"], Value::Null);
    assert!(!pending.to_string().contains("Original collection recipe"));
    assert_eq!(
        accept_household(&app, &shared, &source).await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        accept_household(&app, &shared, &target).await,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, Some("carol"), "GET", &shared, json!({})).await.0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &app,
            Some("carol"),
            "DELETE",
            &shared,
            json!({"expected_version":2})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &app,
            Some("bob"),
            "DELETE",
            &format!("{base}/members/carol"),
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, Some("carol"), "GET", &shared, json!({})).await.0,
        StatusCode::NOT_FOUND
    );
    drop(app);
    let reopened = fixture.app().await;
    assert_eq!(
        call(&reopened, Some("bob"), "GET", &shared, json!({}))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &reopened,
            Some("bob"),
            "DELETE",
            &shared,
            json!({"expected_version":2})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&reopened, Some("bob"), "GET", &shared, json!({}))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
}
#[tokio::test]
async fn reclassification_restricts_older_shared_revisions_without_rewriting_history() {
    let fixture = Fixture::new().await;
    let app = fixture.app().await;
    let home = home(&app, "alice").await;
    let saved = saved_recipe(&app, &home, "alice").await;
    let id = saved["id"].as_str().unwrap();
    let file = fixture
        .root
        .join("data")
        .join(&home)
        .join("recipes")
        .join(format!("{id}-1.json"));
    let before = std::fs::read(&file).unwrap();
    let created = grant(
        &app,
        &home,
        0,
        json!({"kind":"recipe","recipe_id":id,"revision":1}),
        json!({"kind":"account","user":"bob"}),
    )
    .await;
    let shared = format!("/api/shared/{home}/{}", created["id"].as_str().unwrap());
    let mut imported = recipe();
    imported["origin"] = json!({"kind":"imported","source_url":"https://example.org/original"});
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "PUT",
            &format!("/api/households/{home}/domain/api/recipes/{id}"),
            json!({"expected_revision":1,"recipe":imported})
        )
        .await
        .0,
        StatusCode::OK
    );
    let source_link = json!({"kind":"source_link","source_url":"https://example.org/original"});
    assert_eq!(
        call(
            &app,
            Some("bob"),
            "GET",
            &format!("{shared}?servings=3&display_unit=kg"),
            json!({})
        )
        .await
        .1["content"],
        source_link
    );
    assert_eq!(
        call(&app, Some("bob"), "GET", "/api/shared", json!({}))
            .await
            .1["items"][0]["content"],
        source_link
    );
    assert_eq!(std::fs::read(&file).unwrap(), before);
    drop(app);
    let reopened = fixture.app().await;
    let result = call(&reopened, Some("bob"), "GET", &shared, json!({}))
        .await
        .1;
    assert_eq!(result["content"], source_link);
    assert!(!result.to_string().contains("Original collection recipe"));
}

#[tokio::test]
async fn direct_household_grants_require_both_ownership_and_current_target_membership() {
    let fixture = Fixture::new().await;
    let app = fixture.app().await;
    let source = home(&app, "alice").await;
    let destination = home(&app, "alice").await;
    let foreign = home(&app, "bob").await;
    let saved = saved_recipe(&app, &source, "alice").await;
    let resource = json!({"kind":"recipe","recipe_id":saved["id"],"revision":1});
    let outgoing = format!("/api/households/{source}/shares");
    assert_eq!(call(&app,Some("alice"),"POST",&outgoing,json!({"expected_revision":0,"resource":resource,"recipient":{"kind":"household","home_id":foreign}})).await.0,StatusCode::FORBIDDEN);
    assert_eq!(call(&app,Some("alice"),"POST",&outgoing,json!({"expected_revision":0,"resource":resource,"recipient":{"kind":"household","home_id":source}})).await.0,StatusCode::UNPROCESSABLE_ENTITY);
    let created = grant(
        &app,
        &source,
        0,
        resource,
        json!({"kind":"household","home_id":destination}),
    )
    .await;
    let shared = format!("/api/shared/{source}/{}", created["id"].as_str().unwrap());
    assert_eq!(
        call(&app, Some("bob"), "GET", &shared, json!({})).await.0,
        StatusCode::NOT_FOUND
    );
    let base = format!("/api/households/{destination}");
    join(&app, &base, "alice", "bob", "reader").await;
    assert_eq!(
        call(&app, Some("bob"), "GET", &shared, json!({})).await.0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "DELETE",
            &format!("{base}/members/bob"),
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, Some("bob"), "GET", &shared, json!({})).await.0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn unknown_recipients_invalid_references_and_directory_failure_do_not_publish_grants() {
    let fixture = Fixture::new().await;
    let app = fixture.app().await;
    let source = home(&app, "alice").await;
    let saved = saved_recipe(&app, &source, "alice").await;
    let path = format!("/api/households/{source}/shares");
    let resource = json!({"kind":"recipe","recipe_id":saved["id"],"revision":1});
    for (target, recipient) in [
        (
            resource.clone(),
            json!({"kind":"account","user":"not-a-registered-account"}),
        ),
        (
            json!({"kind":"recipe","recipe_id":"../household/profiles","revision":1}),
            json!({"kind":"account","user":"bob"}),
        ),
    ] {
        assert_eq!(
            call(
                &app,
                Some("alice"),
                "POST",
                &path,
                json!({"expected_revision":0,"resource":target,"recipient":recipient})
            )
            .await
            .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    fixture.stop_directory();
    assert_eq!(call(&app,Some("alice"),"POST",&path,json!({"expected_revision":0,"resource":resource,"recipient":{"kind":"account","user":"bob"}})).await.0,StatusCode::SERVICE_UNAVAILABLE);
    let result = call(&app, Some("alice"), "GET", &path, json!({})).await.1;
    assert_eq!(result["revision"], 0);
    assert_eq!(result["shares"], json!([]));
}
