use super::{store::Draft, *};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "mealshop-feedback-{}-{}",
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
fn draft(rating: Option<u8>) -> Draft {
    Draft {
        source: Source::Manual,
        recipe: Recipe {
            id: "recipe-00000001".to_owned(),
            revision: 1,
            title: "Synthetic history recipe".to_owned(),
            language: "en".to_owned(),
        },
        servings: 2,
        input: Input {
            date: "2026-11-10".to_owned(),
            timezone: "Europe/Berlin".to_owned(),
            attendees: vec!["person-a".to_owned()],
            active_seconds: None,
            household_rating: rating,
            ratings: BTreeMap::from([("person-a".to_owned(), 4)]),
        },
    }
}
fn policy() -> Policy {
    Policy {
        enabled: true,
        member_ids: BTreeSet::from(["person-a".to_owned()]),
    }
}
#[test]
fn epochs_incremental_corrections_and_deletion_survive_restart_without_resurrection() {
    let directory = Directory::new();
    let mut store = directory.store();
    let old = store.save(0, None, draft(Some(5)), "alice", 100).unwrap();
    assert!(store.data.stats.is_empty());
    store.policy(1, policy()).unwrap();
    assert!(store.data.stats.is_empty());
    let id = store.save(2, None, draft(Some(5)), "alice", 101).unwrap();
    let stats = &store.data.stats["recipe-00000001"];
    assert_eq!(
        (stats.cooks, stats.household.count, stats.household.sum),
        (1, 1, 5)
    );
    assert_eq!(stats.members["person-a"].sum, 4);
    store
        .save(3, Some(&id), draft(Some(1)), "bob", 102)
        .unwrap();
    let stats = &store.data.stats["recipe-00000001"];
    assert_eq!(
        (stats.cooks, stats.household.count, stats.household.sum),
        (1, 1, 1)
    );
    assert_eq!(store.data.entries[&id].recorded_by, "alice");
    assert_eq!(store.data.entries[&id].updated_by, "bob");
    drop(store);
    let mut store = directory.store();
    assert_eq!(store.data.stats["recipe-00000001"].household.sum, 1);
    store.remove(4, &id).unwrap();
    assert!(store.data.stats.is_empty());
    store
        .save(5, Some(&old), draft(None), "alice", 103)
        .unwrap();
    assert_eq!(store.data.stats["recipe-00000001"].household.count, 0);
    store.policy(6, Policy::default()).unwrap();
    assert!(store.data.stats.is_empty());
    assert!(!store.data.entries.is_empty());
    store.policy(7, policy()).unwrap();
    assert!(store.data.stats.is_empty());
    drop(store);
    let mut store = directory.store();
    assert!(store.data.stats.is_empty());
    store.clear(8).unwrap();
    assert!(store.data.entries.is_empty());
    assert!(store.data.policy.enabled);
}
#[test]
fn invalid_duplicate_stale_and_ambiguous_writes_do_not_change_history() {
    let directory = Directory::new();
    let mut store = directory.store();
    let mut first = draft(Some(5));
    first.source = Source::Session {
        run_id: "cooking-1".to_owned(),
    };
    store.save(0, None, first, "alice", 100).unwrap();
    let before = std::fs::read(directory.0.join("history.json")).unwrap();
    let mut duplicate = draft(Some(1));
    duplicate.source = Source::Session {
        run_id: "cooking-1".to_owned(),
    };
    assert_eq!(
        store.save(1, None, duplicate, "alice", 101).unwrap_err(),
        "history_duplicate_session"
    );
    assert_eq!(
        store.save(0, None, draft(None), "alice", 101).unwrap_err(),
        "history_revision_conflict"
    );
    let mut invalid = draft(Some(0));
    invalid.input.ratings.insert("nonattendee".to_owned(), 5);
    assert!(store.save(1, None, invalid, "alice", 101).is_err());
    assert_eq!(
        std::fs::read(directory.0.join("history.json")).unwrap(),
        before
    );
    std::fs::create_dir(directory.0.join(".history.pending")).unwrap();
    assert_eq!(
        store.save(1, None, draft(None), "alice", 101).unwrap_err(),
        "history_storage_failed"
    );
    std::fs::remove_dir(directory.0.join(".history.pending")).unwrap();
    assert_eq!(
        store.remove(1, "cook-1").unwrap_err(),
        "history_storage_failed"
    );
    drop(store);
    assert_eq!(directory.store().data.entries.len(), 1);
}
#[test]
fn corrupted_aggregates_or_interrupted_transactions_fail_startup() {
    let directory = Directory::new();
    let mut store = directory.store();
    store.policy(0, policy()).unwrap();
    store.save(1, None, draft(Some(5)), "alice", 100).unwrap();
    drop(store);
    let path = directory.0.join("history.json");
    let original = std::fs::read(&path).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&original).unwrap();
    value["stats"]["recipe-00000001"]["household"]["sum"] = serde_json::json!(4);
    std::fs::write(&path, value.to_string()).unwrap();
    assert!(Store::open(&directory.0).is_err());
    std::fs::write(&path, original).unwrap();
    std::fs::write(directory.0.join(".history.pending"), "{}").unwrap();
    assert!(Store::open(&directory.0).is_err());
}
