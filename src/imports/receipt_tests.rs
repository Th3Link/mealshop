use super::{
    model::{Method, Receipt, Source},
    parser,
};
use crate::{
    nutrition::catalogue::Catalogue,
    recipes::{Recipe, Store},
};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "mealshop-source-receipt-{}-{}",
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
fn receipt() -> Receipt {
    let original=parser::extract(r#"{"@type":"Recipe","name":"Source title","author":[{"name":"First author"},{"name":"Second author"}],"recipeIngredient":["100,500 g Haferkorn"],"recipeInstructions":[{"@type":"HowToStep","text":"Mix carefully"}]}"#).unwrap().remove(0);
    Receipt {
        actor: "alice".to_owned(),
        key: "synthetic-import-key-0001".to_owned(),
        preview_id: format!("import-{}", "a".repeat(64)),
        candidate: 0,
        recorded_at: 100,
        source: Source {
            requested_url: "https://example.org/source".to_owned(),
            final_url: Some("https://example.org/canonical".to_owned()),
            method: Method::JsonLd,
            retrieved_at: Some(99),
            importer_version: "schema-recipe-1".to_owned(),
            original,
        },
    }
}
fn recipe() -> Recipe {
    serde_json::from_value(serde_json::json!({"title":"Reviewed title","language":"en","servings":2,"preparation":"as_recorded","origin":{"kind":"imported","source_url":"https://example.org/source"},"lines":[{"ingredient_id":"oats","grams":"100.500000"}],"steps":[]})).unwrap()
}
#[test]
fn fetched_original_fields_are_private_immutable_and_retained_after_actual_reload() {
    let dir = Directory::new();
    let catalogue = Catalogue::from_json(include_str!("../../data/catalogue.json")).unwrap();
    let mut store = Store::open(&dir.0).unwrap();
    let record = store
        .save_imported(recipe(), &catalogue, receipt())
        .unwrap();
    let source = &record.import_receipt.as_ref().unwrap().source;
    assert_eq!(
        source.original.author.as_deref(),
        Some("First author / Second author")
    );
    assert_eq!(source.original.ingredients[0], "100,500 g Haferkorn");
    assert!(source.original.total_time.is_none());
    let bytes = std::fs::read(dir.0.join(format!("{}-1.json", record.id))).unwrap();
    assert!(!String::from_utf8_lossy(&bytes).contains("<html"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(dir.0.join(format!("{}-1.json", record.id)))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    let mut edited = recipe();
    edited.title = "Reviewed edit".to_owned();
    store
        .save(edited, Some((&record.id, 1)), &catalogue)
        .unwrap();
    let second: serde_json::Value = serde_json::from_slice(
        &std::fs::read(dir.0.join(format!("{}-2.json", record.id))).unwrap(),
    )
    .unwrap();
    assert!(second.get("import_receipt").is_none());
    drop(store);
    let store = Store::open(&dir.0).unwrap();
    let fetched = store.get(&record.id, None).unwrap();
    assert_eq!(
        fetched.import_receipt.unwrap().source.original.ingredients[0],
        "100,500 g Haferkorn"
    );
}
#[test]
fn corrupt_or_misplaced_source_receipts_fail_closed() {
    let dir = Directory::new();
    let catalogue = Catalogue::from_json(include_str!("../../data/catalogue.json")).unwrap();
    let mut store = Store::open(&dir.0).unwrap();
    let record = store
        .save_imported(recipe(), &catalogue, receipt())
        .unwrap();
    drop(store);
    let path = dir.0.join(format!("{}-1.json", record.id));
    let mut data: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    data["import_receipt"]["source"]["requested_url"] =
        serde_json::json!("https://example.org/replaced");
    std::fs::write(path, serde_json::to_vec(&data).unwrap()).unwrap();
    assert!(Store::open(&dir.0).is_err());
}
