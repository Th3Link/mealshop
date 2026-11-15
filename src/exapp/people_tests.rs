#![allow(clippy::unwrap_used)]
use super::{
    library_tests::{Fixture, home, join},
    tests::call,
};
use axum::http::StatusCode;
use serde_json::{Value, json};
fn member() -> Value {
    json!({"id":"person","name":"Synthetic person","allergies":[],"allergen_ids":[],"intolerances":[],"excluded_ingredients":[],"disliked_ingredients":[]})
}
fn rules() -> Value {
    json!({"version":1,"preset":"vegan","exceptions":[{"ingredient_id":"egg-whole-raw","household_produced_only":true}],"max_active_seconds":300,"availability":[{"weekdays":[4],"meal":"dinner"}]})
}
#[tokio::test]
#[allow(
    clippy::too_many_lines,
    reason = "The acceptance scenario keeps its restart and permission transitions together"
)]
async fn person_activity_requires_explicit_owner_grants_and_reader_permissions_survive_restart() {
    let f = Fixture::new().await;
    let mut app = f.app().await;
    let id = home(&app, "alice").await;
    let base = format!("/api/households/{id}");
    let domain = format!("{base}/domain/api/household");
    join(&app, &base, "alice", "bob", "editor").await;
    join(&app, &base, "alice", "carol", "reader").await;
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "PUT",
            &format!("{domain}/profiles"),
            json!({"expected_revision":0,"members":[member()]})
        )
        .await
        .0,
        StatusCode::OK
    );
    let access = format!("{domain}/people/person/activity/access");
    let activity = format!("{domain}/people/person/activity");
    assert_eq!(
        call(&app, Some("alice"), "GET", &activity, json!({}))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let input = json!({"expected_profile_revision":1,"expected_generation":null,"expected_access_revision":0,"grants":{"alice":"edit","carol":"edit"}});
    assert_eq!(
        call(&app, Some("alice"), "PUT", &access, input.clone())
            .await
            .1["code"],
        "person_self_grant_confirmation_required"
    );
    let mut confirmed = input;
    confirmed["confirm_self_access"] = json!(true);
    let granted = call(&app, Some("alice"), "PUT", &access, confirmed).await;
    assert_eq!(granted.0, StatusCode::OK);
    let generation = granted.1["generation"].as_u64().unwrap();
    let key = "activity-0123456789abcdef0123456789abcdef0123456789abcdef";
    let entry = json!({"expected_generation":generation,"expected_access_revision":1,"expected_revision":0,"entry":{"date":"2026-11-13","kind":"running","duration_seconds":2700,"intensity":null,"notes":"Synthetic private activity"}});
    assert_eq!(
        call(
            &app,
            Some("carol"),
            "PUT",
            &format!("{activity}/{key}"),
            entry.clone()
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, Some("bob"), "GET", &activity, json!({})).await.0,
        StatusCode::FORBIDDEN
    );
    let public = call(
        &app,
        Some("bob"),
        "GET",
        &format!("{domain}/profiles"),
        json!({}),
    )
    .await
    .1;
    assert_eq!(public["revision"], 1);
    assert!(!public.to_string().contains("Synthetic private activity"));
    assert!(public.get("private_people").is_none());
    drop(app);
    app = f.app().await;
    let restored = call(
        &app,
        Some("carol"),
        "GET",
        &format!("{activity}/export"),
        json!({}),
    )
    .await;
    assert_eq!(restored.0, StatusCode::OK);
    assert_eq!(restored.1["entries"][key]["duration_seconds"], 2700);
    assert!(restored.1["energy"].is_null());
    assert_eq!(call(&app,Some("carol"),"PUT",&access,json!({"expected_profile_revision":1,"expected_generation":generation,"expected_access_revision":1,"grants":{}})).await.0,StatusCode::FORBIDDEN);
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "DELETE",
            &format!("{base}/members/carol"),
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
    join(&app, &base, "alice", "carol", "reader").await;
    assert_eq!(
        call(&app, Some("carol"), "GET", &activity, json!({}))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &app,
            Some("bob"),
            "PUT",
            &format!("{domain}/profiles"),
            json!({"expected_revision":1,"members":[]})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let guard = json!({"expected_generation":generation,"expected_access_revision":1,"expected_revision":1});
    assert_eq!(
        call(&app, Some("alice"), "DELETE", &activity, guard)
            .await
            .0,
        StatusCode::OK
    );
    assert!(
        call(
            &app,
            Some("alice"),
            "GET",
            &format!("{activity}/export"),
            json!({})
        )
        .await
        .1["entries"]
            .as_object()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "PUT",
            &format!("{domain}/profiles"),
            json!({"expected_revision":1,"members":[]})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "PUT",
            &format!("{domain}/profiles"),
            json!({"expected_revision":2,"members":[member()]})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, Some("alice"), "GET", &activity, json!({}))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "PUT",
            &format!("{activity}/{key}"),
            entry
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
}
#[tokio::test]
#[allow(
    clippy::too_many_lines,
    reason = "The acceptance scenario keeps its restart and permission transitions together"
)]
async fn dietary_exceptions_never_override_allergies_or_unknown_origin_and_legacy_writes_keep_rules()
 {
    let f = Fixture::new().await;
    let app = f.app().await;
    let id = home(&app, "alice").await;
    let d = format!("/api/households/{id}/domain/api/household");
    let mut m = member();
    m["rules"] = rules();
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "PUT",
            &format!("{d}/profiles"),
            json!({"expected_revision":0,"members":[m.clone()]})
        )
        .await
        .0,
        StatusCode::OK
    );
    let evaluate = json!({"member_ids":["person"],"ingredient_ids":["egg-whole-raw"]});
    let unknown = call(
        &app,
        Some("alice"),
        "POST",
        &format!("{d}/evaluate"),
        evaluate.clone(),
    )
    .await
    .1;
    assert_eq!(unknown["status"], "review_required");
    assert_eq!(
        unknown["dietary_rules"][0]["unknown_ingredients"][0],
        "egg-whole-raw"
    );
    let annotation = json!({"expected_revision":0,"annotations":{"organization":{"production":"household_produced","diet":"vegetarian","origin":"animal"}}});
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "PUT",
            &format!("{d}/ingredients/egg-whole-raw"),
            annotation
        )
        .await
        .0,
        StatusCode::OK
    );
    // A legacy annotation editor cannot erase a new origin declaration by omission.
    let legacy_annotation = json!({"expected_revision":1,"annotations":{"organization":{"diet":"vegetarian","origin":"animal"}}});
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "PUT",
            &format!("{d}/ingredients/egg-whole-raw"),
            legacy_annotation
        )
        .await
        .0,
        StatusCode::OK
    );
    let foods = call(
        &app,
        Some("alice"),
        "GET",
        &format!("{d}/ingredients"),
        json!({}),
    )
    .await
    .1;
    let egg = foods["ingredients"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == "egg-whole-raw")
        .unwrap();
    assert_eq!(egg["organization"]["production"], "household_produced");
    let allowed = call(
        &app,
        Some("alice"),
        "POST",
        &format!("{d}/evaluate"),
        evaluate.clone(),
    )
    .await
    .1;
    assert_eq!(allowed["status"], "review_required");
    assert_eq!(
        allowed["dietary_rules"][0]["exceptions"][0],
        "egg-whole-raw"
    );
    m["allergen_ids"] = json!(["egg"]);
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "PUT",
            &format!("{d}/profiles"),
            json!({"expected_revision":1,"members":[m]})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &app,
            Some("alice"),
            "POST",
            &format!("{d}/evaluate"),
            evaluate
        )
        .await
        .1["status"],
        "blocked"
    );
    let legacy = call(
        &app,
        Some("alice"),
        "PUT",
        &format!("{d}/profiles"),
        json!({"expected_revision":2,"members":[member()]}),
    )
    .await;
    assert_eq!(legacy.0, StatusCode::OK);
    assert_eq!(legacy.1["members"][0]["rules"], rules());
    let record=call(&app,Some("alice"),"POST",&format!("/api/households/{id}/domain/api/recipes"),json!({"title":"Synthetic rule recipe","language":"en","servings":1,"preparation":"as_recorded","lines":[{"ingredient_id":"oats","grams":"30"}],"steps":[{"instruction":"Prepare","active_seconds":600,"passive_seconds":0}]})).await.1;
    let plan = json!({"date":"2026-11-13","meal":"lunch","recipe":{"id":record["id"],"revision":1},"attendees":["person"],"calendar":null,"servings":1,"active_seconds":3600,"elapsed_seconds":3600,"advance_seconds":0});
    let preview = call(
        &app,
        Some("alice"),
        "POST",
        &format!("{d}/mealplan/preview"),
        plan.clone(),
    )
    .await;
    assert_eq!(preview.0, StatusCode::OK);
    assert_eq!(preview.1["feasible"], false);
    assert!(
        preview.1["conflicts"]
            .as_array()
            .unwrap()
            .contains(&json!("person_active_time_exceeded"))
    );
    assert!(
        preview.1["conflicts"]
            .as_array()
            .unwrap()
            .contains(&json!("person_unavailable"))
    );
    let mut allocated = plan;
    allocated["allocations"] = json!([{"date":"2026-11-13","meal":"dinner","servings":1}]);
    let dinner = call(
        &app,
        Some("alice"),
        "POST",
        &format!("{d}/mealplan/preview"),
        allocated.clone(),
    )
    .await;
    assert_eq!(dinner.0, StatusCode::OK);
    assert!(
        !dinner.1["conflicts"]
            .as_array()
            .unwrap()
            .contains(&json!("person_unavailable"))
    );
    allocated["allocations"][0]["date"] = json!("2026-11-14");
    let saturday = call(
        &app,
        Some("alice"),
        "POST",
        &format!("{d}/mealplan/preview"),
        allocated,
    )
    .await;
    assert!(
        saturday.1["conflicts"]
            .as_array()
            .unwrap()
            .contains(&json!("person_unavailable"))
    );
}
