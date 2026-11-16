use super::*;
use std::{fs, path::PathBuf};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("mealshop-device-{}", random_id().unwrap())))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[allow(clippy::needless_pass_by_value)]
fn call(engine: &mut Engine, path: &str, method: &str, body: Value) -> (u16, Value) {
    let reply = engine
        .execute(Command::Request {
            generation: engine.active.as_ref().map(|(id, _)| id.clone()),
            path: path.to_owned(),
            method: method.to_owned(),
            body: (body != Value::Null)
                .then(|| STANDARD.encode(serde_json::to_vec(&body).unwrap())),
            content_type: (body != Value::Null).then(|| "application/json".to_owned()),
        })
        .unwrap();
    let bytes = STANDARD.decode(reply["body"].as_str().unwrap()).unwrap();
    (
        u16::try_from(reply["status"].as_u64().unwrap()).unwrap(),
        serde_json::from_slice(&bytes).unwrap(),
    )
}
fn home(engine: &mut Engine) -> String {
    let (status, body) = call(
        engine,
        "/api/households",
        "POST",
        json!({"name":"Synthetic on-device household"}),
    );
    assert_eq!(status, 200, "{body}");
    body["id"].as_str().unwrap().to_owned()
}

#[test]
fn real_local_household_shopping_and_recipes_survive_a_core_restart() {
    let fixture = Fixture::new();
    let mut engine = Engine::open(&fixture.0).unwrap();
    assert_eq!(engine.status()["storage"], "ready");
    let id = home(&mut engine);
    let root = format!("/api/households/{id}/domain/api");
    let (status, body) = call(
        &mut engine,
        &format!("{root}/household/shopping"),
        "POST",
        json!({"expected_revision":0,"command":{"operation":"add_manual","label":"Synthetic exact amount","quantity":{"amount":"0.100001","unit":"kg"}}}),
    );
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["items"][0]["remaining"], "100.001000");
    let (status, ingredient) = call(
        &mut engine,
        &format!("{root}/household/ingredients"),
        "POST",
        json!({"expected_revision":0,"ingredient":{"name":"Synthetic local ingredient","language":"en","form":"As declared","recorded_on":"2026-11-16"}}),
    );
    assert_eq!(status, 201, "{ingredient}");
    let (status, recipe) = call(
        &mut engine,
        &format!("{root}/recipes"),
        "POST",
        json!({"title":"Local recipe","language":"en","servings":1,"preparation":"as_recorded","lines":[{"ingredient_id":ingredient["id"],"grams":"1.000001"}],"steps":[]}),
    );
    assert_eq!(status, 201, "{recipe}");
    let recipe_id = recipe["id"].as_str().unwrap().to_owned();
    drop(engine);
    let mut restarted = Engine::open(&fixture.0).unwrap();
    assert_eq!(restarted.status()["storage"], "ready");
    let (status, saved) = call(
        &mut restarted,
        &format!("{root}/recipes/{recipe_id}"),
        "GET",
        Value::Null,
    );
    assert_eq!(status, 200, "{saved}");
    assert_eq!(saved["recipe"]["lines"][0]["grams"], "1.000001");
    let (_, shopping) = call(
        &mut restarted,
        &format!("{root}/household/shopping"),
        "GET",
        Value::Null,
    );
    assert_eq!(shopping["items"][0]["remaining"], "100.001000");
}

#[test]
fn a_second_owner_cannot_touch_the_active_dataset() {
    let fixture = Fixture::new();
    let mut engine = Engine::open(&fixture.0).unwrap();
    let id = home(&mut engine);
    assert!(matches!(Engine::open(&fixture.0), Err("device_in_use")));
    assert_eq!(
        call(&mut engine, "/api/households", "GET", Value::Null).1["households"][0]["id"],
        id
    );
}

#[test]
fn an_empty_interrupted_bootstrap_reuses_its_generation_without_resetting_saved_data() {
    let fixture = Fixture::new();
    let engine = Engine::open(&fixture.0).unwrap();
    let original = engine.active.as_ref().unwrap().0.clone();
    drop(engine);
    fs::remove_file(fixture.0.join("active.json")).unwrap();
    let mut engine = Engine::open(&fixture.0).unwrap();
    assert_eq!(engine.active.as_ref().unwrap().0, original);
    home(&mut engine);
    drop(engine);
    fs::remove_file(fixture.0.join("active.json")).unwrap();
    assert_eq!(
        Engine::open(&fixture.0).unwrap().status()["storage"],
        "recovery"
    );
}

#[cfg(unix)]
#[test]
fn a_linked_dataset_cannot_reclaim_an_unrelated_stale_marker() {
    let fixture = Fixture::new();
    let outside = Fixture::new();
    fs::create_dir(&outside.0).unwrap();
    fs::write(outside.0.join(".membership.lock"), b"Synthetic other owner").unwrap();
    let engine = Engine::open(&fixture.0).unwrap();
    let data = engine
        .root
        .dataset(&engine.active.as_ref().unwrap().0)
        .unwrap();
    drop(engine);
    fs::remove_dir_all(&data).unwrap();
    std::os::unix::fs::symlink(&outside.0, &data).unwrap();
    assert_eq!(
        Engine::open(&fixture.0).unwrap().status()["storage"],
        "recovery"
    );
    assert_eq!(
        fs::read(outside.0.join(".membership.lock")).unwrap(),
        b"Synthetic other owner"
    );
}

#[cfg(unix)]
#[test]
fn an_obsolete_cleanup_failure_does_not_hide_valid_active_households() {
    let fixture = Fixture::new();
    let outside = Fixture::new();
    fs::create_dir(&outside.0).unwrap();
    let mut engine = Engine::open(&fixture.0).unwrap();
    let home = home(&mut engine);
    let unused = engine.root.path.join("datasets").join(random_id().unwrap());
    drop(engine);
    std::os::unix::fs::symlink(&outside.0, &unused).unwrap();
    let mut engine = Engine::open(&fixture.0).unwrap();
    assert_eq!(engine.status()["storage"], "ready");
    assert_eq!(engine.status()["cleanup_complete"], false);
    assert_eq!(
        call(&mut engine, "/api/households", "GET", Value::Null).1["households"][0]["id"],
        home
    );
    assert!(unused.is_symlink());
}

#[test]
fn an_unrelated_existing_directory_is_not_adopted_or_modified() {
    let fixture = Fixture::new();
    fs::create_dir_all(&fixture.0).unwrap();
    fs::write(
        fixture.0.join("membership.json"),
        b"Synthetic unrelated data",
    )
    .unwrap();
    assert!(Engine::open(&fixture.0).is_err());
    assert_eq!(
        fs::read(fixture.0.join("membership.json")).unwrap(),
        b"Synthetic unrelated data"
    );
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 1);
}

#[test]
fn interrupted_data_is_preserved_and_exposes_recovery_instead_of_empty_data() {
    let fixture = Fixture::new();
    let mut engine = Engine::open(&fixture.0).unwrap();
    let id = home(&mut engine);
    let active = engine.active.as_ref().unwrap().0.clone();
    let marker = engine
        .root
        .dataset(&active)
        .unwrap()
        .join(".membership.pending");
    drop(engine);
    fs::write(&marker, b"Interrupted synthetic transaction").unwrap();
    let mut recovery = Engine::open(&fixture.0).unwrap();
    assert_eq!(recovery.status()["storage"], "recovery");
    assert_eq!(
        fs::read(marker).unwrap(),
        b"Interrupted synthetic transaction"
    );
    assert!(
        recovery
            .execute(Command::Request {
                generation: None,
                path: format!("/api/households/{id}"),
                method: "GET".to_owned(),
                body: None,
                content_type: None
            })
            .is_err()
    );
}

#[test]
fn account_invites_are_unavailable_without_a_nextcloud_host() {
    let fixture = Fixture::new();
    let mut engine = Engine::open(&fixture.0).unwrap();
    let id = home(&mut engine);
    let (status, body) = call(
        &mut engine,
        &format!("/api/households/{id}/invitations"),
        "POST",
        json!({"user":"other","role":"editor"}),
    );
    assert_eq!(status, 422);
    assert_eq!(body["code"], "requires_server");
    drop(engine);
    assert_eq!(
        Engine::open(&fixture.0).unwrap().status()["storage"],
        "ready"
    );
}

#[test]
fn protocol_cannot_supply_an_alternate_actor_or_external_url() {
    assert!(serde_json::from_value::<Frame>(json!({"id":1,"command":{"operation":"request","path":"/api/households","method":"GET","body":null,"content_type":null,"user":"other"}})).is_err());
    let fixture = Fixture::new();
    let mut engine = Engine::open(&fixture.0).unwrap();
    assert!(
        engine
            .execute(Command::Request {
                generation: engine.active.as_ref().map(|(id, _)| id.clone()),
                path: "https://example.org/api/households".to_owned(),
                method: "GET".to_owned(),
                body: None,
                content_type: None
            })
            .is_err()
    );
}

fn export(engine: &mut Engine) -> (String, String) {
    let password = "synthetic portable passphrase".to_owned();
    let result = engine
        .execute(Command::Backup {
            passphrase: password.clone(),
        })
        .unwrap();
    (result["id"].as_str().unwrap().to_owned(), password)
}
fn import(engine: &Engine, bytes: &[u8]) -> String {
    let id = random_id().unwrap();
    fs::write(
        backup::transfer(&engine.root, "imports", &id).unwrap(),
        bytes,
    )
    .unwrap();
    id
}
fn exported_bytes(engine: &Engine, id: &str) -> Vec<u8> {
    fs::read(backup::transfer(&engine.root, "exports", id).unwrap()).unwrap()
}

#[test]
fn encrypted_backup_restores_on_another_device_and_only_replaces_after_confirmation() {
    let original = Fixture::new();
    let mut source = Engine::open(&original.0).unwrap();
    let id = home(&mut source);
    let (export_id, password) = export(&mut source);
    assert!(source.status()["last_backup"].is_null());
    let bytes = exported_bytes(&source, &export_id);
    assert!(
        !bytes
            .windows(b"Synthetic on-device".len())
            .any(|w| w == b"Synthetic on-device")
    );
    source.execute(Command::Exported { id: export_id }).unwrap();
    assert_eq!(source.status()["last_backup"]["households"], 1);
    let target = Fixture::new();
    let mut dest = Engine::open(&target.0).unwrap();
    let before = dest.active.as_ref().unwrap().0.clone();
    let import_id = import(&dest, &bytes);
    let preview = dest
        .execute(Command::Restore {
            id: import_id,
            passphrase: password,
        })
        .unwrap();
    assert_eq!(preview["manifest"]["households"], 1);
    assert_eq!(dest.active.as_ref().unwrap().0, before);
    assert_eq!(
        call(&mut dest, "/api/households", "GET", Value::Null).1["households"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let restored = dest
        .execute(Command::CommitRestore {
            token: preview["token"].as_str().unwrap().to_owned(),
        })
        .unwrap();
    assert_eq!(restored["cleanup_complete"], true);
    assert!(!dest.root.dataset(&before).unwrap().exists());
    assert_eq!(
        call(&mut dest, "/api/households", "GET", Value::Null).1["households"][0]["id"],
        id
    );
    drop(dest);
    assert_eq!(
        Engine::open(&target.0).unwrap().status()["storage"],
        "ready"
    );
}

#[test]
fn binary_image_requests_keep_their_content_type_and_exact_bytes() {
    let fixture = Fixture::new();
    let mut engine = Engine::open(&fixture.0).unwrap();
    let home = home(&mut engine);
    let root = format!("/api/households/{home}/domain/api/household/images");
    let png = STANDARD.decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aL1sAAAAASUVORK5CYII=").unwrap();
    let reply = engine
        .execute(Command::Request {
            generation: engine.active.as_ref().map(|(id, _)| id.clone()),
            path: root.clone(),
            method: "POST".to_owned(),
            body: Some(STANDARD.encode(&png)),
            content_type: Some("image/png".to_owned()),
        })
        .unwrap();
    let body = STANDARD.decode(reply["body"].as_str().unwrap()).unwrap();
    assert_eq!(reply["status"], 201, "{}", String::from_utf8_lossy(&body));
    let image: Value = serde_json::from_slice(&body).unwrap();
    let reply = engine
        .execute(Command::Request {
            generation: engine.active.as_ref().map(|(id, _)| id.clone()),
            path: format!("{root}/{}", image["id"].as_str().unwrap()),
            method: "GET".to_owned(),
            body: None,
            content_type: None,
        })
        .unwrap();
    assert_eq!(
        STANDARD.decode(reply["body"].as_str().unwrap()).unwrap(),
        png
    );
}

#[test]
fn wrong_password_damage_and_missing_final_authentication_preserve_active_data() {
    let fixture = Fixture::new();
    let mut engine = Engine::open(&fixture.0).unwrap();
    let id = home(&mut engine);
    let (export_id, password) = export(&mut engine);
    let bytes = exported_bytes(&engine, &export_id);
    let imported = import(&engine, &bytes);
    assert!(matches!(
        engine.execute(Command::Restore {
            id: imported,
            passphrase: "a different synthetic password".to_owned()
        }),
        Err("device_restore_password")
    ));
    for length in [bytes.len() - 1, bytes.len() - 16, bytes.len() / 2] {
        let imported = import(&engine, &bytes[..length]);
        assert!(
            engine
                .execute(Command::Restore {
                    id: imported,
                    passphrase: password.clone()
                })
                .is_err()
        );
    }
    let mut damaged = bytes.clone();
    let last = damaged.len() - 1;
    damaged[last] ^= 1;
    let imported = import(&engine, &damaged);
    assert!(
        engine
            .execute(Command::Restore {
                id: imported,
                passphrase: password
            })
            .is_err()
    );
    assert_eq!(
        call(&mut engine, "/api/households", "GET", Value::Null).1["households"][0]["id"],
        id
    );
    assert_eq!(
        fs::read_dir(engine.root.path.join("datasets"))
            .unwrap()
            .count(),
        1
    );
}

#[test]
fn restore_preview_expires_after_writes_and_cancellation_removes_decrypted_candidate() {
    let fixture = Fixture::new();
    let mut engine = Engine::open(&fixture.0).unwrap();
    home(&mut engine);
    let (export_id, password) = export(&mut engine);
    let imported = import(&engine, &exported_bytes(&engine, &export_id));
    let preview = engine
        .execute(Command::Restore {
            id: imported,
            passphrase: password,
        })
        .unwrap();
    home(&mut engine);
    assert!(matches!(
        engine.execute(Command::CommitRestore {
            token: preview["token"].as_str().unwrap().to_owned()
        }),
        Err("device_restore_expired")
    ));
    engine
        .execute(Command::DiscardRestore {
            token: preview["token"].as_str().unwrap().to_owned(),
        })
        .unwrap();
    assert_eq!(
        fs::read_dir(engine.root.path.join("datasets"))
            .unwrap()
            .count(),
        1
    );
    assert_eq!(
        call(&mut engine, "/api/households", "GET", Value::Null).1["households"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn queued_requests_from_the_replaced_generation_cannot_write_restored_data() {
    let fixture = Fixture::new();
    let mut engine = Engine::open(&fixture.0).unwrap();
    home(&mut engine);
    let original = engine.active.as_ref().unwrap().0.clone();
    let (export_id, password) = export(&mut engine);
    let imported = import(&engine, &exported_bytes(&engine, &export_id));
    let preview = engine
        .execute(Command::Restore {
            id: imported,
            passphrase: password,
        })
        .unwrap();
    engine
        .execute(Command::CommitRestore {
            token: preview["token"].as_str().unwrap().to_owned(),
        })
        .unwrap();
    assert!(matches!(
        engine.execute(Command::Request {
            generation: Some(original),
            path: "/api/households".to_owned(),
            method: "POST".to_owned(),
            body: Some(STANDARD.encode(br#"{"name":"Stale synthetic write"}"#)),
            content_type: Some("application/json".to_owned())
        }),
        Err("device_dataset_changed")
    ));
    assert_eq!(
        call(&mut engine, "/api/households", "GET", Value::Null).1["households"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn late_cancellation_cannot_remove_another_restore_preview() {
    let fixture = Fixture::new();
    let mut engine = Engine::open(&fixture.0).unwrap();
    home(&mut engine);
    let (file, password) = export(&mut engine);
    let imported = import(&engine, &exported_bytes(&engine, &file));
    let old = engine
        .execute(Command::Restore {
            id: imported.clone(),
            passphrase: password.clone(),
        })
        .unwrap();
    let current = engine
        .execute(Command::Restore {
            id: imported,
            passphrase: password,
        })
        .unwrap();
    assert_eq!(
        engine
            .execute(Command::DiscardRestore {
                token: old["token"].as_str().unwrap().to_owned()
            })
            .unwrap()["discarded"],
        false
    );
    engine
        .execute(Command::CommitRestore {
            token: current["token"].as_str().unwrap().to_owned(),
        })
        .unwrap();
    assert_eq!(engine.status()["storage"], "ready");
}

#[test]
fn a_backup_can_replace_a_recovery_dataset_without_discarding_it_before_commit() {
    let fixture = Fixture::new();
    let mut engine = Engine::open(&fixture.0).unwrap();
    home(&mut engine);
    let (export_id, password) = export(&mut engine);
    let bytes = exported_bytes(&engine, &export_id);
    let old = engine
        .root
        .dataset(&engine.active.as_ref().unwrap().0)
        .unwrap();
    drop(engine);
    fs::write(old.join(".membership.pending"), b"Uncommitted fixture").unwrap();
    let mut engine = Engine::open(&fixture.0).unwrap();
    assert_eq!(engine.status()["storage"], "recovery");
    let imported = import(&engine, &bytes);
    let preview = engine
        .execute(Command::Restore {
            id: imported,
            passphrase: password,
        })
        .unwrap();
    assert!(old.join(".membership.pending").exists());
    engine
        .execute(Command::CommitRestore {
            token: preview["token"].as_str().unwrap().to_owned(),
        })
        .unwrap();
    assert!(!old.exists());
    assert_eq!(engine.status()["storage"], "ready");
}

#[test]
fn the_standard_format_restores_an_archive_from_the_native_age012_build() {
    let fixture = Fixture::new();
    let mut engine = Engine::open(&fixture.0).unwrap();
    let imported = import(
        &engine,
        include_bytes!("../../tests/fixtures/device-age012.age"),
    );
    let preview = engine
        .execute(Command::Restore {
            id: imported,
            passphrase: "synthetic portable passphrase".to_owned(),
        })
        .unwrap();
    assert_eq!(preview["manifest"]["households"], 1);
    engine
        .execute(Command::CommitRestore {
            token: preview["token"].as_str().unwrap().to_owned(),
        })
        .unwrap();
    let (_, shopping) = call(
        &mut engine,
        "/api/households/home-1/domain/api/household/shopping",
        "GET",
        Value::Null,
    );
    assert_eq!(shopping["items"][0]["name"], "Synthetic durable item");
    assert_eq!(shopping["items"][0]["remaining"], "100.001000");
}

#[test]
fn whole_device_backup_never_bypasses_explicit_person_activity_read_grants() {
    let fixture = Fixture::new();
    let mut engine = Engine::open(&fixture.0).unwrap();
    let id = home(&mut engine);
    let root = format!("/api/households/{id}/domain/api/household");
    let (status, _) = call(
        &mut engine,
        &format!("{root}/profiles"),
        "PUT",
        json!({"expected_revision":0,"members":[{"id":"person","name":"Synthetic person","allergies":[],"excluded_ingredients":[],"disliked_ingredients":[]}]}),
    );
    assert_eq!(status, 200);
    let (file, _) = export(&mut engine);
    engine.execute(Command::DiscardExport { id: file }).unwrap();
    let access = format!("{root}/people/person/activity/access");
    let activity = format!("{root}/people/person/activity");
    let grant = json!({"expected_profile_revision":1,"expected_generation":null,"expected_access_revision":0,"grants":{"device":"edit"},"confirm_self_access":true});
    let (status, granted) = call(&mut engine, &access, "PUT", grant);
    assert_eq!(status, 200, "{granted}");
    let generation = granted["generation"].as_u64().unwrap();
    let (status, body) = call(
        &mut engine,
        &format!("{activity}/activity-0123456789abcdef0123456789abcdef0123456789abcdef"),
        "PUT",
        json!({"expected_generation":generation,"expected_access_revision":1,"expected_revision":0,"entry":{"date":"2026-11-16","kind":"walking","duration_seconds":60,"intensity":null,"notes":"Synthetic protected record"}}),
    );
    assert_eq!(status, 200, "{body}");
    let (status, body) = call(
        &mut engine,
        &access,
        "PUT",
        json!({"expected_profile_revision":1,"expected_generation":generation,"expected_access_revision":1,"grants":{}}),
    );
    assert_eq!(status, 200, "{body}");
    assert!(matches!(
        engine.execute(Command::Backup {
            passphrase: "synthetic portable passphrase".to_owned()
        }),
        Err("device_backup_access_required")
    ));
    assert_eq!(
        fs::read_dir(engine.root.path.join("exports"))
            .unwrap()
            .count(),
        0
    );
    assert_eq!(call(&mut engine, &activity, "GET", Value::Null).0, 403);
    let (status, body) = call(
        &mut engine,
        &access,
        "PUT",
        json!({"expected_profile_revision":1,"expected_generation":generation,"expected_access_revision":2,"grants":{"device":"read"},"confirm_self_access":true}),
    );
    assert_eq!(status, 200, "{body}");
    export(&mut engine);
}
