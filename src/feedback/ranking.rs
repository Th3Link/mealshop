use super::{
    Data,
    model::{RatingStats, Stats},
};
use serde::Serialize;
use std::collections::BTreeMap;

pub(crate) const BASELINE_VERSION: &str = "rules-v1";
pub(crate) const LEARNED_VERSION: &str = "history-v1";
#[derive(Clone)]
pub(crate) struct Candidate {
    pub(crate) id: String,
    pub(crate) revision: u64,
    pub(crate) dislikes: usize,
}
#[derive(Clone, Serialize)]
pub(crate) struct Evidence {
    pub(crate) cooks: u32,
    pub(crate) household_ratings: u32,
    pub(crate) personal_ratings: u64,
    pub(crate) recent_cooks: u32,
    pub(crate) earlier_revision_cooks: u32,
}
#[derive(Clone, Serialize)]
pub(crate) struct Explanation {
    pub(crate) model_version: &'static str,
    pub(crate) score: i32,
    pub(crate) baseline_rank: usize,
    pub(crate) confidence: &'static str,
    pub(crate) discovery: bool,
    pub(crate) evidence: Evidence,
    pub(crate) reasons: Vec<&'static str>,
}
#[derive(Clone)]
pub(crate) struct Ranked {
    pub(crate) candidate: Candidate,
    pub(crate) explanation: Explanation,
}
fn points(stats: &RatingStats) -> Result<i32, &'static str> {
    let value = (i64::from(stats.sum) + 6) * 100 / (i64::from(stats.count) + 2) - 300;
    i32::try_from(value).map_err(|_| "history_storage_failed")
}
fn explanation(
    candidate: &Candidate,
    history: &Data,
    people: &[String],
    meal_day: &str,
) -> Result<Explanation, &'static str> {
    let empty = Stats::default();
    let stats = if history.policy.enabled {
        history.stats.get(&candidate.id).unwrap_or(&empty)
    } else {
        &empty
    };
    let meal_day = meal_day
        .parse::<jiff::civil::Date>()
        .map_err(|_| "invalid_meal_plan")?;
    let cutoff = meal_day
        .checked_sub(jiff::SignedDuration::from_secs(6 * 86400))
        .map_err(|_| "invalid_meal_plan")?
        .to_string();
    let recent = stats
        .dates
        .range(cutoff..=meal_day.to_string())
        .map(|(_, count)| count)
        .sum::<u32>();
    let personal = people
        .iter()
        .filter_map(|id| stats.members.get(id))
        .collect::<Vec<_>>();
    let personal_points = personal
        .iter()
        .map(|s| points(s).map(i64::from))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .sum::<i64>();
    let personal_average = if personal.is_empty() {
        0
    } else {
        personal_points / i64::try_from(personal.len()).map_err(|_| "history_storage_failed")?
    };
    let score = i64::from(points(&stats.household)?)
        + personal_average
        + i64::from(stats.cooks.min(10)) * 5
        - i64::from(recent.min(3)) * 30;
    let mut reasons = Vec::new();
    if candidate.dislikes > 0 {
        reasons.push("declared_dislikes");
    }
    if stats.cooks > 0 {
        reasons.push("completed_cooks");
    }
    if stats.household.count > 0 {
        reasons.push("household_ratings");
    }
    if !personal.is_empty() {
        reasons.push("attendee_ratings");
    }
    if recent > 0 {
        reasons.push("recent_variety");
    }
    let earlier = stats
        .cooks
        .saturating_sub(*stats.revisions.get(&candidate.revision).unwrap_or(&0));
    if earlier > 0 {
        reasons.push("earlier_revisions");
    }
    Ok(Explanation {
        model_version: if history.policy.enabled {
            LEARNED_VERSION
        } else {
            BASELINE_VERSION
        },
        score: i32::try_from(score).map_err(|_| "history_storage_failed")?,
        baseline_rank: 0,
        confidence: match stats.cooks {
            0 => "cold_start",
            1..=2 => "limited_history",
            _ => "repeated_history",
        },
        discovery: false,
        evidence: Evidence {
            cooks: stats.cooks,
            household_ratings: stats.household.count,
            personal_ratings: personal.iter().map(|s| u64::from(s.count)).sum(),
            recent_cooks: recent,
            earlier_revision_cooks: earlier,
        },
        reasons,
    })
}
fn order(rows: &mut Vec<Ranked>, learned: bool) {
    rows.sort_by(|a, b| {
        a.candidate
            .dislikes
            .cmp(&b.candidate.dislikes)
            .then_with(|| {
                if learned {
                    b.explanation.score.cmp(&a.explanation.score)
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .then_with(|| a.candidate.id.cmp(&b.candidate.id))
            .then_with(|| a.candidate.revision.cmp(&b.candidate.revision))
    });
    let mut start = 0;
    while start < rows.len() {
        let end = start
            + rows[start..]
                .iter()
                .take_while(|r| r.candidate.dislikes == rows[start].candidate.dislikes)
                .count();
        let choice = (start..end)
            .filter(|i| !learned || rows[*i].explanation.evidence.cooks == 0)
            .max_by(|a, b| rows[*a].candidate.id.cmp(&rows[*b].candidate.id));
        if let Some(index) = choice {
            let target = (start + 2).min(end - 1);
            rows[index].explanation.discovery = true;
            if index > target {
                let row = rows.remove(index);
                rows.insert(target, row);
            }
        }
        start = end;
    }
}
pub(crate) fn rank(
    candidates: Vec<Candidate>,
    history: &Data,
    people: &[String],
    meal_day: &str,
) -> Result<Vec<Ranked>, &'static str> {
    let mut rows = candidates
        .into_iter()
        .map(|candidate| {
            explanation(&candidate, history, people, meal_day).map(|explanation| Ranked {
                candidate,
                explanation,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut baseline = rows.clone();
    order(&mut baseline, false);
    let positions: BTreeMap<_, _> = baseline
        .iter()
        .enumerate()
        .map(|(i, row)| (row.candidate.id.clone(), i + 1))
        .collect();
    order(&mut rows, history.policy.enabled);
    for row in &mut rows {
        row.explanation.baseline_rank = *positions
            .get(&row.candidate.id)
            .ok_or("history_storage_failed")?;
        if row.explanation.discovery {
            row.explanation.reasons.push("discovery");
        }
    }
    Ok(rows)
}
