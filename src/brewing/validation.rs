use super::{
    model::{Beverage, EquipmentInput, Mash, Measurement, Method, ReadingUnit, Recipe},
    templates,
};
use crate::{
    cooking::{
        self,
        model::{Data as Processes, StageOrigin},
    },
    shopping::model::{Quantity, Unit},
};
use std::collections::{BTreeMap, BTreeSet};
fn text(value: &str, max: usize) -> bool {
    !value.trim().is_empty()
        && value.len() <= max
        && !value.chars().any(|c| c.is_control() && c != '\n')
}
fn optional_text(value: &str, max: usize) -> bool {
    value.is_empty() || text(value, max)
}
pub(super) fn volume(value: &Quantity) -> bool {
    matches!(value.unit, Unit::Ml | Unit::L) && value.normalized().is_ok()
}
pub(super) fn equipment(e: &EquipmentInput) -> Result<(), &'static str> {
    if !text(&e.name, 120)
        || !optional_text(&e.model, 200)
        || !optional_text(&e.notes, 2000)
        || e.capacity.as_ref().is_some_and(|q| !volume(q))
    {
        return Err("brewing_equipment_invalid");
    }
    Ok(())
}
pub(super) fn measurement(value: &Measurement) -> Result<(), &'static str> {
    let negative = value.value.starts_with('-');
    let magnitude = value.value.strip_prefix('-').unwrap_or(&value.value);
    let amount = crate::quantities::Amount::try_from(magnitude.to_owned())
        .map_err(|_| "brewing_measurement_invalid")?;
    if negative
        && match value.unit {
            ReadingUnit::Celsius => amount.atoms() > 273_150_000,
            ReadingUnit::Fahrenheit => amount.atoms() > 459_670_000,
            _ => true,
        }
    {
        return Err("brewing_measurement_invalid");
    }
    Ok(())
}
fn method(r: &Recipe) -> Result<(), &'static str> {
    match &r.method {
        Method::Beer { strategy, mash, .. } if r.beverage == Beverage::Beer => {
            if matches!(
                strategy,
                templates::Strategy::PartialMash | templates::Strategy::AllGrain
            ) == (*mash == Mash::None)
            {
                return Err("brewing_method_invalid");
            }
        }
        Method::Fermentation { process } if process.beverage() == r.beverage => {}
        _ => return Err("brewing_method_invalid"),
    }
    Ok(())
}
pub(super) fn recipe(r: &Recipe, fresh: bool) -> Result<(), &'static str> {
    if !text(&r.title, 200)
        || !matches!(r.language.as_str(), "en" | "de")
        || !volume(&r.volume)
        || r.phases.is_empty()
        || r.phases.len() > 100
        || r.ingredients.len() > 200
        || !optional_text(&r.notes, 4000)
        || r.template.sources.is_empty()
        || r.template.sources.len() > 20
        || !text(&r.template.id, 80)
        || r.template.version == 0
    {
        return Err("brewing_recipe_invalid");
    }
    if let Some(p) = &r.yeast_preparation {
        let probe = cooking::LocalTime {
            local: format!("2026-11-13T{}", p.local_time),
            timezone: "UTC".into(),
            fold: p.fold,
        };
        if p.days_before > 30
            || p.local_time.len() != 5
            || probe.resolve().is_err()
            || !text(&p.instruction, 4000)
        {
            return Err("brewing_preparation_invalid");
        }
    }
    method(r)?;
    let ids = r
        .phases
        .iter()
        .map(|p| p.stage.id.as_str())
        .collect::<BTreeSet<_>>();
    if ids.len() != r.phases.len() {
        return Err("brewing_recipe_invalid");
    }
    for p in &r.phases {
        p.stage.validate_limit(31_536_000)?;
        if r.beverage != Beverage::Beer
            && matches!(
                p.kind,
                templates::PhaseKind::Steep
                    | templates::PhaseKind::Mash
                    | templates::PhaseKind::Lauter
                    | templates::PhaseKind::BagLift
                    | templates::PhaseKind::BasketLift
                    | templates::PhaseKind::MaltPipeLift
                    | templates::PhaseKind::Sparge
                    | templates::PhaseKind::Extract
            )
        {
            return Err("brewing_phase_invalid");
        }
        if !matches!(p.stage.origin, StageOrigin::Custom { .. })
            || p.stage.start_at.is_some()
            || (!p.timing_known
                && (p.stage.active_seconds != 0
                    || p.stage.passive_seconds != 0
                    || p.stage.passive_max_seconds != 0))
            || p.volume.as_ref().is_some_and(|v| !volume(v))
        {
            return Err("brewing_phase_invalid");
        }
        if let Some(t) = &p.temperature {
            if !matches!(t.unit, ReadingUnit::Celsius | ReadingUnit::Fahrenheit) {
                return Err("brewing_measurement_invalid");
            }
            measurement(t)?;
        }
    }
    cooking::plan::order(&r.phases.iter().map(|p| p.stage.clone()).collect::<Vec<_>>())?;
    let mut line_ids = BTreeSet::new();
    for line in &r.ingredients {
        if !line_ids.insert(&line.id)
            || !cooking::model::id(&line.id)
            || !text(&line.name, 200)
            || line.ingredient_id.as_ref().is_some_and(|v| !text(v, 120))
            || line
                .quantity
                .as_ref()
                .is_some_and(|q| q.normalized().is_err())
            || line
                .phase_id
                .as_ref()
                .is_some_and(|id| !ids.contains(id.as_str()))
            || !optional_text(&line.note, 1000)
        {
            return Err("brewing_ingredient_invalid");
        }
    }
    sources(r, fresh)
}
fn sources(r: &Recipe, fresh: bool) -> Result<(), &'static str> {
    for s in &r.template.sources {
        if !text(&s.id, 80)
            || !text(&s.publisher, 200)
            || !text(&s.title, 500)
            || !crate::planning::date(&s.retrieved_on)
            || (crate::recipe_origin::Origin::Imported {
                source_url: s.url.clone(),
            })
            .validate()
            .is_err()
        {
            return Err("brewing_source_invalid");
        }
    }
    if fresh {
        let c = templates::catalogue()?;
        let t = c
            .templates
            .iter()
            .find(|t| t.id == r.template.id && t.version == r.template.version)
            .ok_or("brewing_source_invalid")?;
        if t.beverage != r.beverage
            || match &r.method {
                Method::Fermentation { process } => t.process != Some(*process),
                Method::Beer { .. } => false,
            }
        {
            return Err("brewing_method_invalid");
        }
        let expected = c
            .sources
            .iter()
            .filter(|s| t.sources.contains(&s.id))
            .map(|s| (&s.id, s))
            .collect::<BTreeMap<_, _>>();
        let actual = r
            .template
            .sources
            .iter()
            .map(|s| (&s.id, s))
            .collect::<BTreeMap<_, _>>();
        if actual.len() != r.template.sources.len() || actual != expected {
            return Err("brewing_source_invalid");
        }
    }
    Ok(())
}
pub(crate) fn validate(data: &Processes) -> Result<(), &'static str> {
    let b = &data.brewing;
    if b.recipes.len() > 200
        || b.equipment.len() > 100
        || b.batches.len() > 100
        || b.readings.len() > 2000
        || b.operations.len() > 2000
    {
        return Err("brewing_capacity");
    }
    let mut revisions = BTreeMap::new();
    for r in &b.recipes {
        if !cooking::store::allocated(data, &r.id, "brew-recipe-")
            || !cooking::actor_valid(&r.actor)
            || !cooking::time::valid(r.recorded_at)
            || r.revision != revisions.get(&r.id).copied().unwrap_or(0) + 1
        {
            return Err("brewing_data_invalid");
        }
        recipe(&r.recipe, false)?;
        revisions.insert(&r.id, r.revision);
    }
    if b.archived_recipes.iter().collect::<BTreeSet<_>>().len() != b.archived_recipes.len()
        || b.archived_recipes
            .iter()
            .any(|id| !revisions.contains_key(id))
    {
        return Err("brewing_data_invalid");
    }
    let mut ids = BTreeSet::new();
    for e in &b.equipment {
        if !ids.insert(&e.id) || !cooking::store::allocated(data, &e.id, "brew-equipment-") {
            return Err("brewing_data_invalid");
        }
        equipment(&e.details)?;
    }
    receipts(data, &revisions, &ids)?;
    for (id, batch) in &b.batches {
        if !cooking::store::allocated(data, id, "cook-") {
            return Err("brewing_data_invalid");
        }
        self::batch(data, data.run(id)?, batch)?;
    }
    if data
        .runs
        .iter()
        .any(|r| (r.kind.is_brewing()) != b.batches.contains_key(&r.id))
    {
        return Err("brewing_data_invalid");
    }
    super::readings::validate(data)?;
    super::resources::validate_holds(data)?;
    Ok(())
}

fn receipts(
    data: &Processes,
    revisions: &BTreeMap<&String, u64>,
    ids: &BTreeSet<&String>,
) -> Result<(), &'static str> {
    let b = &data.brewing;
    let mut operation_ids = BTreeSet::new();
    let mut operation_keys = BTreeSet::new();
    for receipt in &b.operations {
        if !operation_ids.insert(&receipt.id)
            || !operation_keys.insert((&receipt.actor, &receipt.key))
            || !cooking::store::allocated(data, &receipt.id, "brew-operation-")
            || !cooking::actor_valid(&receipt.actor)
            || !crate::recipes::Creation::valid_key(&receipt.key)
            || receipt.expected_revision >= data.revision
            || !cooking::time::valid(receipt.recorded_at)
        {
            return Err("brewing_data_invalid");
        }
        let command: super::operations::Command =
            serde_json::from_value(receipt.command.clone()).map_err(|_| "brewing_data_invalid")?;
        let result = receipt.result_id.as_ref().ok_or("brewing_data_invalid")?;
        match command {
            super::operations::Command::CreateBatch { draft } => {
                let batch = b.batches.get(result).ok_or("brewing_data_invalid")?;
                if batch.draft != draft
                    || batch.actor != receipt.actor
                    || batch.recorded_at != receipt.recorded_at
                {
                    return Err("brewing_data_invalid");
                }
            }
            super::operations::Command::AddReading {
                run_id,
                reading,
                corrects,
                reason,
            } => {
                let r = b
                    .readings
                    .iter()
                    .find(|r| &r.id == result)
                    .ok_or("brewing_data_invalid")?;
                if r.run_id != run_id
                    || r.input != reading
                    || r.corrects != corrects
                    || r.reason != reason
                    || r.actor != receipt.actor
                    || r.recorded_at != receipt.recorded_at
                {
                    return Err("brewing_data_invalid");
                }
            }
            super::operations::Command::SaveRecipe {
                id,
                expected_recipe_revision,
                recipe,
            } => {
                let revision = expected_recipe_revision
                    .unwrap_or(0)
                    .checked_add(1)
                    .ok_or("brewing_data_invalid")?;
                let record = b.recipe(result, revision)?;
                if id.as_ref().is_some_and(|id| id != result)
                    || record.recipe != recipe
                    || record.actor != receipt.actor
                    || record.recorded_at != receipt.recorded_at
                {
                    return Err("brewing_data_invalid");
                }
            }
            super::operations::Command::ArchiveRecipe { id, .. } => {
                if &id != result || !revisions.contains_key(&id) {
                    return Err("brewing_data_invalid");
                }
            }
            super::operations::Command::SaveEquipment { id, .. } => {
                if id.as_ref().is_some_and(|id| id != result) || !ids.contains(result) {
                    return Err("brewing_data_invalid");
                }
            }
            super::operations::Command::ArchiveEquipment { id, .. } => {
                if &id != result || !ids.contains(result) {
                    return Err("brewing_data_invalid");
                }
            }
            command => super::controls::validate(data, &command, result, &receipt.actor)?,
        }
    }
    Ok(())
}

pub(super) fn batch(
    data: &Processes,
    run: &cooking::Run,
    batch: &super::model::Batch,
) -> Result<(), &'static str> {
    let draft = &batch.draft;
    let recipe = &data
        .brewing
        .recipe(&draft.recipe_id, draft.recipe_revision)?
        .recipe;
    let crate::cooking::run_source::Source::Brewing { recipe: reference } = &run.source else {
        return Err("brewing_data_invalid");
    };
    if !cooking::actor_valid(&batch.actor)
        || !cooking::time::valid(batch.recorded_at)
        || draft.brew_day.resolve().is_err()
        || run.created_at != batch.recorded_at
        || run.name != draft.name
        || !text(&draft.name, 200)
        || run.timezone != draft.brew_day.timezone
        || reference.id != draft.recipe_id
        || reference.revision != draft.recipe_revision
        || reference.title != recipe.title
        || reference.language != recipe.language
        || reference.volume != recipe.volume
        || reference.beverage != recipe.beverage
        || (batch.archived && run.status == cooking::RunState::Active)
    {
        return Err("brewing_data_invalid");
    }
    let used = draft.equipment.values().flatten().collect::<BTreeSet<_>>();
    let snapshots = batch
        .equipment
        .iter()
        .map(|e| &e.id)
        .collect::<BTreeSet<_>>();
    if used != snapshots
        || snapshots.len() != batch.equipment.len()
        || draft
            .equipment
            .keys()
            .any(|id| !recipe.phases.iter().any(|p| &p.stage.id == id))
        || draft
            .equipment
            .values()
            .any(|ids| ids.len() > 20 || ids.iter().collect::<BTreeSet<_>>().len() != ids.len())
    {
        return Err("brewing_data_invalid");
    }
    for e in &batch.equipment {
        equipment(&e.details)?;
        if e.archived
            || !data
                .brewing
                .equipment
                .iter()
                .any(|current| current.id == e.id)
        {
            return Err("brewing_data_invalid");
        }
    }
    let prep = super::batches::preparation(recipe, draft)?;
    if prep.as_ref().is_some_and(|p| {
        p.at.timezone != run.timezone
            || p.at.resolve().is_err()
            || p.at.resolve().is_ok_and(|at| {
                at < batch.recorded_at || at > draft.brew_day.resolve().unwrap_or(0)
            })
            || !text(&p.instruction, 4000)
    }) {
        return Err("brewing_data_invalid");
    }
    let stages = super::batches::stages(recipe, draft, prep)?;
    let expected = cooking::plan::expand_limit(&stages, 31_536_000)?;
    if expected.len() != run.steps.len()
        || expected.iter().zip(&run.steps).any(|(a, b)| {
            a.stage != b.stage || a.cycle != b.cycle || a.lag_seconds != b.lag_seconds
        })
    {
        return Err("brewing_data_invalid");
    }
    Ok(())
}
