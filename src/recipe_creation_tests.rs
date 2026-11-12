//! Atomic creation receipts are part of the published recipe revision, never a later journal.
#![allow(clippy::unwrap_used)]
use crate::{
    nutrition::catalogue::Catalogue,
    recipes::{Creation, Error, Recipe, Store},
};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "mealshop-mcp-receipt-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn store(&self) -> Store {
        Store::open(&self.0).unwrap()
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn catalogue() -> Catalogue {
    Catalogue::from_json(include_str!("../data/catalogue.json")).unwrap()
}
fn recipe() -> Recipe {
    serde_json::from_value(serde_json::json!({"title":"Synthetic MCP recipe","language":"en","servings":2,"preparation":"as_recorded","lines":[{"ingredient_id":"oats","grams":"100.500000"}],"steps":[]})).unwrap()
}
fn receipt() -> Creation {
    Creation {
        actor: "alice".to_owned(),
        key: "synthetic-request-0001".to_owned(),
        preview_id: format!("mcp-{}", "a".repeat(64)),
        recorded_at: 100,
    }
}
#[test]
fn published_creation_retries_resolve_after_restart_and_keep_private_receipts() {
    let dir = Directory::new();
    let catalogue = catalogue();
    let mut store = dir.store();
    let first = store.save_created(recipe(), &catalogue, receipt()).unwrap();
    assert_eq!(store.list().len(), 1);
    let same = store.save_created(recipe(), &catalogue, receipt()).unwrap();
    assert_eq!(same.id, first.id);
    let bytes = std::fs::read(dir.0.join(format!("{}-1.json", first.id))).unwrap();
    let stored: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(stored["creation"]["actor"], "alice");
    drop(store);
    let mut store = dir.store();
    let same = store.save_created(recipe(), &catalogue, receipt()).unwrap();
    assert_eq!(same.id, first.id);
    assert_eq!(store.list().len(), 1);
    let mut other = receipt();
    other.preview_id = format!("mcp-{}", "b".repeat(64));
    assert!(matches!(
        store.save_created(recipe(), &catalogue, other),
        Err(Error::Conflict)
    ));
    let mut edited = recipe();
    edited.title = "Revised MCP recipe".to_owned();
    store
        .save(edited, Some((&first.id, 1)), &catalogue)
        .unwrap();
    assert_eq!(
        store
            .creation_result("alice", &receipt().key, &receipt().preview_id)
            .unwrap()
            .unwrap()
            .revision,
        1
    );
    assert_eq!(store.creation_activity()[0]["actor"], "alice");
    assert!(store.creation_activity()[0].get("key").is_none());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(dir.0.join(format!("{}-1.json", first.id)))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
#[test]
fn preview_is_nonpersistent_and_uses_the_same_validation_as_publication() {
    let dir = Directory::new();
    let catalogue = catalogue();
    let mut store = dir.store();
    let mut invalid = recipe();
    invalid.lines[0].ingredient_id = "not-a-food".to_owned();
    assert!(matches!(
        store.prepare(invalid.clone(), None, &catalogue),
        Err(Error::Invalid(_))
    ));
    assert!(matches!(
        store.save_created(invalid, &catalogue, receipt()),
        Err(Error::Invalid(_))
    ));
    assert!(store.list().is_empty());
    store.prepare(recipe(), None, &catalogue).unwrap();
    assert!(store.list().is_empty());
    assert!(
        dir.0
            .read_dir()
            .unwrap()
            .all(|p| p.unwrap().file_name() == ".mealshop.lock")
    );
}
#[test]
fn a_damaged_or_duplicated_receipt_fails_restart_instead_of_permitting_replay() {
    let dir = Directory::new();
    let catalogue = catalogue();
    let mut store = dir.store();
    let first = store.save_created(recipe(), &catalogue, receipt()).unwrap();
    let second = store.save(recipe(), None, &catalogue).unwrap();
    drop(store);
    let first_path = dir.0.join(format!("{}-1.json", first.id));
    let second_path = dir.0.join(format!("{}-1.json", second.id));
    let first: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&first_path).unwrap()).unwrap();
    let mut second: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&second_path).unwrap()).unwrap();
    second["creation"] = first["creation"].clone();
    std::fs::write(second_path, second.to_string()).unwrap();
    assert!(Store::open(&dir.0).is_err());
}
