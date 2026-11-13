use super::{
    model::{Batch, BatchDraft, PreparationChoice, Recipe, YeastPreparation},
    validation,
};
use crate::cooking::{
    self,
    model::{Data, Kind, Run, RunState, Stage, StageOrigin},
    run_source::{BrewRef, Source},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn preparation(
    recipe: &Recipe,
    draft: &BatchDraft,
) -> Result<Option<YeastPreparation>, &'static str> {
    match &draft.yeast_preparation {
        PreparationChoice::Skip => Ok(None),
        PreparationChoice::Custom { at, instruction } => Ok(Some(YeastPreparation {
            at: at.clone(),
            instruction: instruction.clone(),
        })),
        PreparationChoice::Recipe => recipe
            .yeast_preparation
            .as_ref()
            .map(|p| {
                let day: jiff::civil::DateTime = draft
                    .brew_day
                    .local
                    .parse()
                    .map_err(|_| "cooking_time_invalid")?;
                let day = day
                    .checked_sub(jiff::Span::new().days(i64::from(p.days_before)))
                    .map_err(|_| "cooking_time_invalid")?;
                let at = cooking::LocalTime {
                    local: format!("{}T{}", day.date(), p.local_time),
                    timezone: draft.brew_day.timezone.clone(),
                    fold: p.fold,
                };
                at.resolve()?;
                Ok(YeastPreparation {
                    at,
                    instruction: p.instruction.clone(),
                })
            })
            .transpose(),
    }
}
pub(super) fn build(
    data: &Data,
    draft: &BatchDraft,
    actor: &str,
    now: i64,
) -> Result<(Run, Batch), &'static str> {
    if !cooking::model::text(&draft.name, 200) || draft.equipment.len() > 100 {
        return Err("brewing_batch_invalid");
    }
    draft.brew_day.resolve()?;
    let record = data
        .brewing
        .recipe(&draft.recipe_id, draft.recipe_revision)?;
    if data.brewing.archived_recipes.contains(&record.id) {
        return Err("brewing_recipe_archived");
    }
    let recipe = &record.recipe;
    let phase_ids = recipe
        .phases
        .iter()
        .map(|p| &p.stage.id)
        .collect::<BTreeSet<_>>();
    if draft.equipment.keys().any(|id| !phase_ids.contains(id)) {
        return Err("brewing_equipment_invalid");
    }
    let mut equipment = vec![];
    for ids in draft.equipment.values() {
        if ids.len() > 20 || ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
            return Err("brewing_equipment_invalid");
        }
        for id in ids {
            let e = data
                .brewing
                .equipment
                .iter()
                .find(|e| &e.id == id && !e.archived)
                .ok_or("brewing_equipment_not_found")?;
            if !equipment
                .iter()
                .any(|old: &super::model::Equipment| old.id == e.id)
            {
                equipment.push(e.clone());
            }
        }
    }
    let prep = preparation(recipe, draft)?;
    if prep.as_ref().is_some_and(|p| {
        p.at.timezone != draft.brew_day.timezone
            || p.at.resolve().is_err()
            || p.at
                .resolve()
                .is_ok_and(|at| at < now || at > draft.brew_day.resolve().unwrap_or(0))
            || !cooking::model::text(&p.instruction, 4000)
    }) {
        return Err("brewing_preparation_invalid");
    }
    let stages = stages(recipe, draft, prep)?;
    let mut steps = cooking::plan::expand_limit(&stages, 31_536_000)?;
    for s in &mut steps {
        s.not_before = s
            .stage
            .start_at
            .as_ref()
            .map(cooking::LocalTime::resolve)
            .transpose()?;
    }
    let mut run = Run {
        plan_slot: None,
        id: String::new(),
        name: draft.name.clone(),
        kind: Kind::Beer,
        source: Source::Beer {
            recipe: BrewRef {
                id: record.id.clone(),
                revision: record.revision,
                title: recipe.title.clone(),
                language: recipe.language.clone(),
                volume: recipe.volume.clone(),
            },
        },
        timezone: draft.brew_day.timezone.clone(),
        target: None,
        created_at: now,
        status: RunState::Active,
        steps,
        schedule: vec![],
        subscriptions: BTreeMap::new(),
    };
    run.schedule = cooking::plan::schedule(&run, now, None)?.schedule;
    let batch = Batch {
        draft: draft.clone(),
        equipment,
        actor: actor.to_owned(),
        recorded_at: now,
        archived: false,
    };
    validation::batch(data, &run, &batch)?;
    Ok((run, batch))
}

pub(super) fn stages(
    recipe: &Recipe,
    draft: &BatchDraft,
    prep: Option<YeastPreparation>,
) -> Result<Vec<Stage>, &'static str> {
    let mut stages = recipe
        .phases
        .iter()
        .map(|p| {
            let mut stage = p.stage.clone();
            stage.equipment = draft.equipment.get(&stage.id).cloned().unwrap_or_default();
            if stage.dependencies.is_empty() {
                stage.start_at = Some(draft.brew_day.clone());
            }
            stage
        })
        .collect::<Vec<_>>();
    if let Some(p) = prep {
        if stages.iter().any(|s| s.id == "yeast-preparation") {
            return Err("brewing_phase_invalid");
        }
        stages.insert(
            0,
            Stage {
                id: "yeast-preparation".into(),
                origin: StageOrigin::Custom {
                    note: "Recipe-specific preparation".into(),
                },
                name: p
                    .instruction
                    .lines()
                    .next()
                    .unwrap_or("")
                    .chars()
                    .take(80)
                    .collect(),
                instruction: p.instruction,
                start_at: Some(p.at),
                start_after_work: false,
                dependencies: vec![],
                active_seconds: 0,
                passive_seconds: 0,
                passive_min_seconds: 0,
                passive_max_seconds: 0,
                equipment: vec![],
                lead_seconds: 0,
                repeat: 1,
                wait_between_seconds: 0,
                uncertainty: String::new(),
            },
        );
    }
    Ok(stages)
}

pub(super) fn create(
    data: &mut Data,
    draft: &BatchDraft,
    actor: &str,
    now: i64,
) -> Result<String, &'static str> {
    let (mut run, batch) = build(data, draft, actor, now)?;
    super::resources::check(data, &run, &batch, now)?;
    let id = data.allocate("cook")?;
    run.id.clone_from(&id);
    data.runs.push(run);
    data.brewing.batches.insert(id.clone(), batch);
    Ok(id)
}
