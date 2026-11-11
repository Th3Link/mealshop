use super::{
    ranking::{Candidate, rank},
    *,
};
use std::collections::{BTreeMap, BTreeSet};
fn candidates() -> Vec<Candidate> {
    (1..=30)
        .map(|n| Candidate {
            id: format!("recipe-{n:08}"),
            revision: 1,
            dislikes: 0,
        })
        .collect()
}
fn learn(data: &mut Data, id: &str, rating: u8) {
    let entry = Entry {
        id: format!("cook-{}", data.next_id),
        source: Source::Manual,
        recipe: Recipe {
            id: id.to_owned(),
            revision: 1,
            title: "Synthetic evaluation recipe".to_owned(),
            language: "en".to_owned(),
        },
        servings: 2,
        input: Input {
            date: "2026-10-01".to_owned(),
            timezone: "Europe/Berlin".to_owned(),
            attendees: vec!["person-a".to_owned()],
            active_seconds: None,
            household_rating: Some(rating),
            ratings: BTreeMap::new(),
        },
        recorded_by: "local".to_owned(),
        recorded_at: 1,
        updated_by: "local".to_owned(),
        updated_at: 1,
        epoch: Some(data.epoch),
    };
    data.delta(&entry, 1).unwrap();
    data.entries.insert(entry.id.clone(), entry);
    data.next_id += 1;
}
#[test]
fn cold_start_and_late_catalogue_entries_remain_discoverable_without_learning() {
    let data = Data::default();
    let rows = rank(candidates(), &data, &[], "2026-11-01").unwrap();
    assert_eq!(rows[2].candidate.id, "recipe-00000030");
    assert!(rows[2].explanation.discovery);
    assert!(
        rows.iter()
            .all(|r| r.explanation.confidence == "cold_start" && r.explanation.score == 0)
    );
    assert_eq!(
        rows.iter()
            .map(|r| &r.candidate.id)
            .collect::<BTreeSet<_>>()
            .len(),
        30
    );
}
#[test]
fn ratings_change_order_but_not_declared_dislikes_and_discovery_survives_sparse_feedback() {
    let mut data = Data {
        policy: Policy {
            enabled: true,
            member_ids: BTreeSet::new(),
        },
        ..Data::default()
    };
    learn(&mut data, "recipe-00000001", 1);
    learn(&mut data, "recipe-00000002", 5);
    data.validate().unwrap();
    let rows = rank(candidates(), &data, &[], "2026-11-01").unwrap();
    assert_eq!(rows[0].candidate.id, "recipe-00000002");
    assert_eq!(rows[0].explanation.baseline_rank, 2);
    assert_eq!(rows[0].explanation.confidence, "limited_history");
    assert_eq!(rows[2].candidate.id, "recipe-00000030");
    let mut pool = candidates();
    pool[1].dislikes = 1;
    let rows = rank(pool, &data, &[], "2026-11-01").unwrap();
    assert_eq!(rows.last().unwrap().candidate.id, "recipe-00000002");
}
#[test]
fn revision_support_and_recent_variety_are_explicit_and_reset_returns_to_rules() {
    let mut data = Data {
        policy: Policy {
            enabled: true,
            member_ids: BTreeSet::new(),
        },
        ..Data::default()
    };
    learn(&mut data, "recipe-00000001", 5);
    let mut pool = candidates();
    pool[0].revision = 2;
    let old = rank(pool, &data, &[], "2026-11-01").unwrap();
    assert_eq!(old[0].explanation.evidence.earlier_revision_cooks, 1);
    let recent = rank(candidates(), &data, &[], "2026-10-02").unwrap();
    assert!(recent[0].explanation.score < old[0].explanation.score);
    assert_eq!(recent[0].explanation.evidence.recent_cooks, 1);
    data.policy = Policy::default();
    data.stats.clear();
    let reset = rank(candidates(), &data, &[], "2026-11-01").unwrap();
    assert_eq!(reset[0].candidate.id, "recipe-00000001");
    assert!(
        reset
            .iter()
            .all(|r| r.explanation.model_version == "rules-v1")
    );
}
