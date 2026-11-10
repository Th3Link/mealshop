use super::{
    model::{Draft, Kind, RecipeRef, Run, RunState, Stage, StageOrigin, Timer},
    plan, time,
};
use crate::recipes::{Record, Store};
use std::collections::BTreeMap;
pub(crate) fn recipe(store: &Store, id: &str, revision: u64) -> Result<Record, &'static str> {
    store
        .get(id, Some(revision))
        .map_err(|_| "recipe_revision_not_found")
}
pub(crate) fn template(
    record: &Record,
    servings: u32,
    timezone: String,
) -> Result<Draft, &'static str> {
    if record.recipe.steps.is_empty() {
        return Err("cooking_recipe_has_no_steps");
    }
    time::zone(&timezone)?;
    let mut last_part = BTreeMap::new();
    let mut previous = BTreeMap::new();
    for (i, step) in record.recipe.steps.iter().enumerate() {
        if let Some(part) = &step.part_id {
            last_part.insert(part.as_str(), format!("s{}", i + 1));
        }
    }
    let mut stages = vec![];
    for (i, step) in record.recipe.steps.iter().enumerate() {
        let id = format!("s{}", i + 1);
        let key = step.part_id.as_deref();
        let mut dependencies = vec![];
        if let Some(previous) = previous.insert(key, id.clone()) {
            dependencies.push(previous);
        } else if let Some(part) =
            key.and_then(|id| record.recipe.parts.iter().find(|p| p.id == id))
        {
            dependencies.extend(
                part.uses_parts
                    .iter()
                    .filter_map(|id| last_part.get(id.as_str()).cloned()),
            );
        }
        dependencies.extend(
            step.prepared_parts
                .iter()
                .filter_map(|id| last_part.get(id.as_str()).cloned()),
        );
        dependencies.sort();
        dependencies.dedup();
        let mut name = String::new();
        for c in step.instruction.lines().next().unwrap_or("").chars() {
            if name.len() + c.len_utf8() > 180 {
                break;
            }
            name.push(c);
        }
        stages.push(Stage {
            id,
            origin: StageOrigin::RecipeStep { index: i },
            name,
            instruction: step.instruction.clone(),
            dependencies,
            active_seconds: step.active_seconds,
            passive_seconds: step.passive_seconds,
            passive_min_seconds: step.passive_seconds,
            passive_max_seconds: step.passive_seconds,
            equipment: vec![],
            lead_seconds: 300,
            repeat: 1,
            wait_between_seconds: 0,
            uncertainty: String::new(),
        });
    }
    let draft = Draft {
        recipe_id: record.id.clone(),
        recipe_revision: record.revision,
        servings,
        name: record.recipe.title.clone(),
        kind: Kind::Cooking,
        timezone,
        target: None,
        stages,
    };
    validate(&draft, record)?;
    Ok(draft)
}
fn validate(draft: &Draft, record: &Record) -> Result<(), &'static str> {
    if !(1..=1_000_000).contains(&draft.servings)
        || !super::model::text(&draft.name, 200)
        || draft.stages.len() > 100
        || draft.stages.is_empty()
        || time::zone(&draft.timezone).is_err()
        || draft
            .target
            .as_ref()
            .is_some_and(|t| t.timezone != draft.timezone)
        || (!matches!(draft.kind, Kind::Cooking) && draft.target.is_none())
    {
        return Err("cooking_draft_invalid");
    }
    if let Some(target) = &draft.target {
        target.resolve()?;
    }
    for stage in &draft.stages {
        stage.validate()?;
        if let StageOrigin::RecipeStep { index } = stage.origin {
            if record.recipe.steps.get(index).is_none() {
                return Err("cooking_stage_invalid");
            }
        }
    }
    Ok(())
}
pub(crate) fn create(
    draft: Draft,
    record: &Record,
    now: i64,
) -> Result<(Run, plan::Proposal), &'static str> {
    validate(&draft, record)?;
    let mut run = Run {
        id: String::new(),
        name: draft.name,
        kind: draft.kind,
        recipe: RecipeRef {
            id: record.id.clone(),
            revision: record.revision,
            title: record.recipe.title.clone(),
            language: record.recipe.language.clone(),
            reference_servings: record.recipe.servings,
        },
        servings: draft.servings,
        timezone: draft.timezone,
        target: draft.target,
        created_at: now,
        status: RunState::Active,
        steps: plan::expand(&draft.stages)?,
        schedule: vec![],
        subscriptions: BTreeMap::new(),
    };
    let mut proposal = plan::schedule(&run, now, None)?;
    proposal.warnings.push(plan::Warning {
        code: "cooking_duration_estimate",
        steps: vec![],
    });
    run.schedule.clone_from(&proposal.schedule);
    Ok((run, proposal))
}
pub(crate) fn ingredients(
    record: &Record,
    servings: u32,
) -> Result<serde_json::Value, &'static str> {
    let lines=record.recipe.lines.iter().map(|line| {
        let ingredient=record.catalogue.find(&line.ingredient_id).ok_or("unknown_ingredient")?;
        let grams=line.grams.0.scaled(servings,record.recipe.servings).map_err(|_|"cooking_quantity_invalid")?;
        Ok(serde_json::json!({"ingredient_id":line.ingredient_id,"name":ingredient.name,"translations":ingredient.translations,"grams":grams,"original_quantity":line.quantity,"part_id":line.part_id}))
    }).collect::<Result<Vec<_>,&'static str>>()?;
    Ok(serde_json::json!(lines))
}
pub(crate) fn ensure_ready(run: &Run, id: &str, now: i64) -> Result<usize, &'static str> {
    if run.status != RunState::Active {
        return Err("cooking_run_inactive");
    }
    let index = run
        .steps
        .iter()
        .position(|s| s.stage.id == id)
        .ok_or("cooking_step_not_found")?;
    if !run.steps[index].stage.dependencies.iter().all(|id| {
        run.steps
            .iter()
            .any(|s| s.stage.id == *id && s.timer.complete())
    }) {
        return Err("cooking_dependencies_incomplete");
    }
    if matches!(run.steps[index].timer, Timer::Pending)
        && available_at(run, &run.steps[index])?.is_some_and(|at| at > now)
    {
        return Err("cooking_wait_not_elapsed");
    }
    Ok(index)
}
pub(crate) fn started(timer: &Timer) -> Option<i64> {
    match *timer {
        Timer::Running { started_at, .. } | Timer::Paused { started_at, .. } => Some(started_at),
        _ => None,
    }
}

pub(crate) fn available_at(
    run: &Run,
    step: &super::model::Step,
) -> Result<Option<i64>, &'static str> {
    let mut earliest = run
        .created_at
        .max(step.not_before.unwrap_or(run.created_at));
    for id in &step.stage.dependencies {
        let dependency = run
            .steps
            .iter()
            .find(|s| &s.stage.id == id)
            .ok_or("cooking_dependencies_invalid")?;
        let Timer::Completed { completed_at, .. } = dependency.timer else {
            return Ok(None);
        };
        earliest = earliest.max(time::add(completed_at, i64::from(step.lag_seconds))?);
    }
    Ok(Some(earliest))
}
