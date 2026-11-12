//! Versioned synthetic holdout evaluation; never populated from private household data.
use super::{
    ranking::{self, Candidate},
    *,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Deserialize)]
struct Fixture {
    schema_version: u32,
    baseline: String,
    learned: String,
    training_date: String,
    heldout_date: String,
    recipes: Vec<Food>,
    cases: Vec<Case>,
    expected: Expected,
}
#[derive(Deserialize)]
struct Food {
    id: String,
    eligible: bool,
}
#[derive(Deserialize)]
struct Observation {
    recipe: u32,
    rating: u8,
}
#[derive(Deserialize)]
struct Case {
    name: String,
    training: Vec<Observation>,
    heldout_choice: u32,
    heldout_ratings: BTreeMap<String, u8>,
}
#[derive(Deserialize)]
struct Expected {
    baseline_hits: u32,
    learned_hits: u32,
    baseline_variety: usize,
    learned_variety: usize,
    baseline_rating_points: u32,
    learned_rating_points: u32,
}
#[derive(Default, serde::Serialize)]
struct Metrics {
    queries: u32,
    heldout_acceptance_hits: u32,
    constraint_violations: u32,
    rating_points: u32,
    #[serde(skip)]
    distinct: BTreeSet<String>,
    distinct_recommendations: usize,
}
fn history(fixture: &Fixture, case: &Case) -> Data {
    let mut data = Data {
        policy: Policy {
            enabled: true,
            member_ids: BTreeSet::new(),
        },
        ..Data::default()
    };
    for observation in &case.training {
        let id = format!("recipe-{:08}", observation.recipe);
        let entry = Entry {
            id: format!("cook-{}", data.next_id),
            source: Source::Manual,
            recipe: Recipe {
                id,
                revision: 1,
                title: case.name.clone(),
                language: "en".to_owned(),
            },
            servings: 2,
            input: Input {
                date: fixture.training_date.clone(),
                timezone: "Europe/Berlin".to_owned(),
                attendees: vec!["person-a".to_owned()],
                active_seconds: None,
                household_rating: Some(observation.rating),
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
    data.validate().unwrap();
    data
}
fn observe(metrics: &mut Metrics, fixture: &Fixture, case: &Case, id: &str) {
    metrics.queries += 1;
    metrics.heldout_acceptance_hits +=
        u32::from(id == format!("recipe-{:08}", case.heldout_choice));
    metrics.constraint_violations +=
        u32::from(!fixture.recipes.iter().any(|r| r.id == id && r.eligible));
    let number = id
        .trim_start_matches("recipe-")
        .parse::<u32>()
        .unwrap()
        .to_string();
    metrics.rating_points += u32::from(case.heldout_ratings[&number]);
    metrics.distinct.insert(id.to_owned());
    metrics.distinct_recommendations = metrics.distinct.len();
}
#[test]
fn synthetic_holdout_compares_constraints_acceptance_variety_and_rating_utility() {
    let fixture: Fixture =
        serde_json::from_str(include_str!("../../tests/fixtures/recommendations-v1.json")).unwrap();
    assert_eq!(fixture.schema_version, 1);
    assert_eq!(fixture.baseline, ranking::BASELINE_VERSION);
    assert_eq!(fixture.learned, ranking::LEARNED_VERSION);
    assert!(fixture.training_date < fixture.heldout_date);
    let mut baseline = Metrics::default();
    let mut learned = Metrics::default();
    for case in &fixture.cases {
        let candidates = || {
            fixture
                .recipes
                .iter()
                .filter(|r| r.eligible)
                .map(|r| Candidate {
                    id: r.id.clone(),
                    revision: 1,
                    dislikes: 0,
                })
                .collect()
        };
        let rules =
            ranking::rank(candidates(), &Data::default(), &[], &fixture.heldout_date).unwrap();
        let preferences = ranking::rank(
            candidates(),
            &history(&fixture, case),
            &[],
            &fixture.heldout_date,
        )
        .unwrap();
        observe(&mut baseline, &fixture, case, &rules[0].candidate.id);
        observe(&mut learned, &fixture, case, &preferences[0].candidate.id);
    }
    assert_eq!(baseline.constraint_violations, 0);
    assert_eq!(learned.constraint_violations, 0);
    assert_eq!(
        baseline.heldout_acceptance_hits,
        fixture.expected.baseline_hits
    );
    assert_eq!(
        learned.heldout_acceptance_hits,
        fixture.expected.learned_hits
    );
    assert_eq!(
        baseline.distinct_recommendations,
        fixture.expected.baseline_variety
    );
    assert_eq!(
        learned.distinct_recommendations,
        fixture.expected.learned_variety
    );
    assert_eq!(
        baseline.rating_points,
        fixture.expected.baseline_rating_points
    );
    assert_eq!(
        learned.rating_points,
        fixture.expected.learned_rating_points
    );
    println!(
        "{}",
        serde_json::json!({"dataset":"synthetic-holdout-v1","baseline_version":fixture.baseline,"learned_version":fixture.learned,"baseline":baseline,"learned":learned,"notice":"Offline proxies only; not observed human satisfaction or causal effectiveness"})
    );
}
