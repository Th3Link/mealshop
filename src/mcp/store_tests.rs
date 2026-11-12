use super::{Scope, Store, store::Preview};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "mealshop-mcp-settings-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
#[test]
fn previews_expire_monotonically_and_identical_scope_saves_preserve_live_drafts() {
    let dir = Directory::new();
    let mut store = Store::open(&dir.0).unwrap();
    let grants = BTreeMap::from([("alice".to_owned(), Scope::Create)]);
    store.configure(0, grants.clone()).unwrap();
    let now = Instant::now();
    let recipe=serde_json::from_value(serde_json::json!({"title":"Synthetic preview","language":"en","servings":1,"preparation":"as_recorded","lines":[{"ingredient_id":"oats","grams":"1"}],"steps":[]})).unwrap();
    store
        .preview(Preview {
            actor: "alice".to_owned(),
            grant_revision: 1,
            ingredient_revision: 0,
            deadline: now + Duration::from_secs(60),
            recipe,
        })
        .unwrap();
    store.configure(1, grants).unwrap();
    assert_eq!(store.grants.revision, 1);
    assert_eq!(store.previews.len(), 1);
    assert!(matches!(
        store.scope("alice", crate::membership::Role::Reader),
        Scope::Read
    ));
    store.expire(now + Duration::from_secs(61));
    assert!(store.previews.is_empty());
    drop(store);
    assert!(matches!(
        Store::open(&dir.0)
            .unwrap()
            .scope("alice", crate::membership::Role::Owner),
        Scope::Create
    ));
}
#[test]
fn interrupted_or_corrupt_settings_fail_closed_and_scope_files_stay_private() {
    let dir = Directory::new();
    let mut store = Store::open(&dir.0).unwrap();
    store
        .configure(0, BTreeMap::from([("alice".to_owned(), Scope::Read)]))
        .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(dir.0.join("mcp.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    std::fs::write(dir.0.join(".mcp.pending"), b"incomplete").unwrap();
    assert!(Store::open(&dir.0).is_err());
    std::fs::remove_file(dir.0.join(".mcp.pending")).unwrap();
    std::fs::write(
        dir.0.join("mcp.json"),
        br#"{"schema_version":1,"revision":1,"users":{"alice":"unrestricted"}}"#,
    )
    .unwrap();
    assert!(Store::open(&dir.0).is_err());
}
