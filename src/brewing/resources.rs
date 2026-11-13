use super::model::Batch;
use crate::cooking::model::{Data, Run, RunState, Timer};
use std::collections::BTreeSet;

pub(super) fn check(
    data: &Data,
    candidate: &Run,
    batch: &Batch,
    now: i64,
) -> Result<(), &'static str> {
    if candidate.status != RunState::Active {
        return Ok(());
    }
    capacity(data, batch)?;
    let runs = data
        .runs
        .iter()
        .filter(|r| {
            r.id != candidate.id
                && data.brewing.batches.contains_key(&r.id)
                && r.status == RunState::Active
        })
        .chain(std::iter::once(candidate))
        .collect::<Vec<_>>();
    let unfinished = |s: &crate::cooking::model::Step| {
        !matches!(s.timer, Timer::Completed { .. } | Timer::Cancelled)
    };
    let held = |s: &crate::cooking::model::Step| {
        matches!(s.timer, Timer::Running { .. } | Timer::Paused { .. })
    };
    for (i, step) in candidate
        .steps
        .iter()
        .enumerate()
        .filter(|(_, s)| unfinished(s))
    {
        for other in &runs {
            for (j, next) in other
                .steps
                .iter()
                .enumerate()
                .filter(|(_, s)| unfinished(s))
            {
                if (other.id == candidate.id && j <= i)
                    || !step
                        .stage
                        .equipment
                        .iter()
                        .any(|id| next.stage.equipment.contains(id))
                {
                    continue;
                }
                if held(step) && held(next) {
                    return Err("brewing_equipment_busy");
                }
                let a = candidate
                    .schedule
                    .iter()
                    .find(|t| t.id == step.stage.id)
                    .ok_or("brewing_data_invalid")?;
                let b = other
                    .schedule
                    .iter()
                    .find(|t| t.id == next.stage.id)
                    .ok_or("brewing_data_invalid")?;
                let end_a = if held(step) { i64::MAX } else { a.finish };
                let end_b = if held(next) { i64::MAX } else { b.finish };
                let start_a = if held(step) {
                    a.start.min(now)
                } else {
                    a.start
                };
                let start_b = if held(next) {
                    b.start.min(now)
                } else {
                    b.start
                };
                let related = other.id == candidate.id
                    && (depends(candidate, &step.stage.id, &next.stage.id)
                        || depends(candidate, &next.stage.id, &step.stage.id));
                if !related && start_a < end_b && start_b < end_a {
                    return Err("brewing_equipment_overlap");
                }
            }
        }
    }
    Ok(())
}
fn capacity(data: &Data, batch: &Batch) -> Result<(), &'static str> {
    let recipe = &data
        .brewing
        .recipe(&batch.draft.recipe_id, batch.draft.recipe_revision)?
        .recipe;
    for phase in &recipe.phases {
        for id in batch
            .draft
            .equipment
            .get(&phase.stage.id)
            .into_iter()
            .flatten()
        {
            let equipment = batch
                .equipment
                .iter()
                .find(|e| &e.id == id)
                .ok_or("brewing_equipment_invalid")?;
            if let (Some(volume), Some(capacity)) = (&phase.volume, &equipment.details.capacity) {
                if volume.normalized()?.1.atoms() > capacity.normalized()?.1.atoms() {
                    return Err("brewing_equipment_capacity");
                }
            }
        }
    }
    Ok(())
}

fn depends(run: &Run, child: &str, parent: &str) -> bool {
    let mut pending = vec![child];
    let mut seen = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        if let Some(s) = run.steps.iter().find(|s| s.stage.id == id) {
            if s.stage.dependencies.iter().any(|id| id == parent) {
                return true;
            }
            pending.extend(s.stage.dependencies.iter().map(String::as_str));
        }
    }
    false
}
pub(super) fn start(data: &Data, run_id: &str, step_id: &str) -> Result<(), &'static str> {
    let step = data
        .run(run_id)?
        .steps
        .iter()
        .find(|s| s.stage.id == step_id)
        .ok_or("cooking_step_not_found")?;
    for run in data
        .runs
        .iter()
        .filter(|r| data.brewing.batches.contains_key(&r.id) && r.status == RunState::Active)
    {
        if run.steps.iter().any(|s| {
            !(run.id == run_id && s.stage.id == step_id)
                && matches!(s.timer, Timer::Running { .. } | Timer::Paused { .. })
                && s.stage
                    .equipment
                    .iter()
                    .any(|id| step.stage.equipment.contains(id))
        }) {
            return Err("brewing_equipment_busy");
        }
    }
    Ok(())
}

pub(super) fn validate_holds(data: &Data) -> Result<(), &'static str> {
    let mut held = BTreeSet::new();
    for run in data
        .runs
        .iter()
        .filter(|r| r.status == RunState::Active && data.brewing.batches.contains_key(&r.id))
    {
        for step in run
            .steps
            .iter()
            .filter(|s| matches!(s.timer, Timer::Running { .. } | Timer::Paused { .. }))
        {
            for id in &step.stage.equipment {
                if !held.insert(id) {
                    return Err("brewing_data_invalid");
                }
            }
        }
    }
    Ok(())
}
