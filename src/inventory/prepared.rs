use super::{
    model::{AllocationState, Command},
    operations::Context,
};
use crate::shopping::model::{Data, text};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Storage {
    location: String,
    condition: Condition,
    declared_on: String,
    use_by: Option<String>,
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Condition {
    Ambient,
    Chilled,
    Frozen,
    Other,
}
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Source {
    pub(crate) date: String,
    pub(crate) meal: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    recipe_id: String,
    revision: u64,
    portions: u32,
    produced_on: String,
    storage: Storage,
    source: Option<Source>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Portion {
    pub(crate) id: String,
    pub(crate) date: String,
    pub(crate) meal: String,
    pub(crate) portions: u32,
    pub(crate) member_ids: Vec<String>,
    pub(crate) state: AllocationState,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Batch {
    pub(crate) id: String,
    pub(crate) recipe_id: String,
    pub(crate) revision: u64,
    title: String,
    pub(crate) original_portions: u32,
    produced_on: String,
    storage: Storage,
    pub(crate) source: Option<Source>,
    pub(crate) planned_portions: Option<u32>,
    pub(crate) allocations: Vec<Portion>,
    pub(crate) reversed: bool,
}
impl Batch {
    fn allocated(&self) -> u32 {
        self.allocations
            .iter()
            .filter(|a| a.state != AllocationState::Released)
            .map(|a| a.portions)
            .sum()
    }
    fn validate(&self) -> bool {
        let mut ids = BTreeSet::new();
        crate::recipes::valid_id(&self.recipe_id)
            && self.revision > 0
            && text(&self.title, 200)
            && (1..=1_000_000).contains(&self.original_portions)
            && crate::planning::date(&self.produced_on)
            && storage_valid(&self.storage, &self.produced_on)
            && self.source.as_ref().is_none_or(source_valid)
            && self.allocations.len() <= 100
            && self.allocations.iter().all(|a| {
                ids.insert(&a.id)
                    && text(&a.id, 80)
                    && crate::planning::date(&a.date)
                    && a.date >= self.produced_on
                    && meal(&a.meal)
                    && a.portions > 0
                    && a.portions <= self.original_portions
                    && !a.member_ids.is_empty()
                    && a.member_ids.len() <= 100
                    && a.member_ids.iter().all(|m| text(m, 100))
                    && a.member_ids.iter().collect::<BTreeSet<_>>().len() == a.member_ids.len()
            })
            && self.allocated() <= self.original_portions
            && (!self.reversed || self.allocated() == 0)
    }
}
fn source_valid(s: &Source) -> bool {
    crate::planning::date(&s.date) && text(&s.meal, 40)
}
fn meal(value: &str) -> bool {
    matches!(value, "breakfast" | "lunch" | "snack" | "dinner")
}
fn storage_valid(s: &Storage, produced: &str) -> bool {
    text(&s.location, 100)
        && crate::planning::date(&s.declared_on)
        && s.declared_on.as_str() >= produced
        && s.use_by
            .as_ref()
            .is_none_or(|d| crate::planning::date(d) && d.as_str() >= produced)
}
pub(super) fn rows(data: &Data) -> Vec<Value> {
    data.inventory.prepared.iter().map(|b| {
        let held: u32=b.allocations.iter().filter(|a|a.state==AllocationState::Held).map(|a|a.portions).sum();
        let used: u32=b.allocations.iter().filter(|a|a.state==AllocationState::Consumed).map(|a|a.portions).sum();
        json!({"batch":b,"held":held,"used":used,"available":if b.reversed {0} else {b.original_portions-b.allocated()},
            "storage_is_user_declared":true,"safety_guarantee":false})
    }).collect()
}
pub(super) fn history(data: &Data, offset: usize, limit: usize) -> (Vec<Value>, usize) {
    let events = data
        .inventory
        .operations
        .iter()
        .rev()
        .filter(|op| {
            matches!(
                op.command,
                Command::RecordPrepared { .. }
                    | Command::StorePrepared { .. }
                    | Command::AllocatePrepared { .. }
                    | Command::ReleasePrepared { .. }
                    | Command::UsePrepared { .. }
                    | Command::UndoPreparedUse { .. }
                    | Command::ReversePrepared { .. }
            )
        })
        .collect::<Vec<_>>();
    let rows = events
        .iter()
        .skip(offset)
        .take(limit)
        .map(|op| {
            json!({
            "id":op.id,"actor":op.actor,"recorded_at":op.recorded_at,"command":op.command})
        })
        .collect();
    (rows, events.len())
}
pub(crate) fn validate(data: &Data) -> Result<(), &'static str> {
    let mut ids = BTreeSet::new();
    let mut sources = BTreeSet::new();
    if data.inventory.prepared.len() > 500 {
        return Err("inventory_capacity");
    }
    for b in &data.inventory.prepared {
        if !ids.insert(&b.id)
            || !b.validate()
            || !data.inventory.operations.iter().any(|op| op.id == b.id && matches!(&op.command,
                Command::RecordPrepared { input } if input.recipe_id == b.recipe_id && input.revision == b.revision
                    && input.portions == b.original_portions && input.produced_on == b.produced_on && input.source == b.source))
            || b.source
                .as_ref()
                .is_some_and(|s| !b.reversed && !sources.insert((&s.date, &s.meal)))
        {
            return Err("prepared_data_invalid");
        }
        for a in &b.allocations {
            if !data.inventory.operations.iter().any(|op| op.id == a.id && matches!(&op.command,
                Command::AllocatePrepared {batch_id,date,meal,portions,member_ids}
                    if *batch_id==b.id && *date==a.date && *meal==a.meal && *portions==a.portions && *member_ids==a.member_ids)) {
                return Err("prepared_data_invalid");
            }
        }
    }
    Ok(())
}
pub(super) fn execute(
    data: &mut Data,
    ctx: &Context<'_>,
    operation: &str,
    command: &Command,
) -> Result<(), &'static str> {
    if let Command::RecordPrepared { input } = command {
        return record(data, ctx, operation, input);
    }
    let (Command::StorePrepared { batch_id, .. }
    | Command::AllocatePrepared { batch_id, .. }
    | Command::ReleasePrepared { batch_id, .. }
    | Command::UsePrepared { batch_id, .. }
    | Command::UndoPreparedUse { batch_id, .. }
    | Command::ReversePrepared { batch_id, .. }) = command
    else {
        return Err("inventory_request_invalid");
    };
    if let Command::AllocatePrepared {
        date,
        meal,
        member_ids,
        ..
    } = command
    {
        if data
            .inventory
            .prepared
            .iter()
            .filter(|b| !b.reversed)
            .flat_map(|b| &b.allocations)
            .any(|a| {
                a.date == *date
                    && a.meal == *meal
                    && a.state != AllocationState::Released
                    && member_ids.iter().any(|id| a.member_ids.contains(id))
            })
        {
            return Err("prepared_slot_conflict");
        }
    }
    let b = data
        .inventory
        .prepared
        .iter_mut()
        .find(|b| b.id == *batch_id && !b.reversed)
        .ok_or("prepared_unavailable")?;
    match command {
        Command::StorePrepared { storage, .. } => {
            if !storage_valid(storage, &b.produced_on) {
                return Err("prepared_storage_invalid");
            }
            b.storage = storage.clone();
        }
        Command::AllocatePrepared { .. } => allocate(b, ctx, operation, command)?,
        Command::ReleasePrepared { allocation_id, .. }
        | Command::UsePrepared { allocation_id, .. }
        | Command::UndoPreparedUse { allocation_id, .. } => {
            let a = b
                .allocations
                .iter()
                .find(|a| a.id == *allocation_id)
                .cloned()
                .ok_or("prepared_unavailable")?;
            let next = match command {
                Command::ReleasePrepared { .. } if a.state == AllocationState::Held => {
                    AllocationState::Released
                }
                Command::UsePrepared { .. } if a.state == AllocationState::Held => {
                    review(ctx, b, &a.member_ids, &a.date, &a.meal)?;
                    AllocationState::Consumed
                }
                Command::UndoPreparedUse { reason, .. }
                    if a.state == AllocationState::Consumed && text(reason, 500) =>
                {
                    AllocationState::Held
                }
                _ => return Err("prepared_unavailable"),
            };
            b.allocations
                .iter_mut()
                .find(|a| a.id == *allocation_id)
                .ok_or("prepared_unavailable")?
                .state = next;
        }
        Command::ReversePrepared { reason, .. } => {
            if !text(reason, 500) || b.allocated() != 0 {
                return Err("prepared_reserved");
            }
            b.reversed = true;
        }
        _ => return Err("inventory_request_invalid"),
    }
    if !b.validate() {
        return Err("prepared_data_invalid");
    }
    Ok(())
}
fn allocate(
    b: &mut Batch,
    ctx: &Context<'_>,
    operation: &str,
    command: &Command,
) -> Result<(), &'static str> {
    let Command::AllocatePrepared {
        date,
        meal,
        portions,
        member_ids,
        ..
    } = command
    else {
        return Err("inventory_request_invalid");
    };
    if !crate::planning::date(date)
        || date < &b.produced_on
        || !self::meal(meal)
        || *portions == 0
        || *portions > b.original_portions - b.allocated()
        || b.storage.use_by.as_ref().is_some_and(|d| d < date)
        || member_ids.is_empty()
        || member_ids.len() > 100
        || member_ids.iter().collect::<BTreeSet<_>>().len() != member_ids.len()
        || b.allocations.len() >= 100
    {
        return Err("prepared_allocation_invalid");
    }
    review(ctx, b, member_ids, date, meal)?;
    b.allocations.push(Portion {
        id: operation.to_owned(),
        date: date.clone(),
        meal: meal.clone(),
        portions: *portions,
        member_ids: member_ids.clone(),
        state: AllocationState::Held,
    });
    Ok(())
}
fn record(
    data: &mut Data,
    ctx: &Context<'_>,
    operation: &str,
    input: &Input,
) -> Result<(), &'static str> {
    if !(1..=1_000_000).contains(&input.portions)
        || !crate::planning::date(&input.produced_on)
        || !storage_valid(&input.storage, &input.produced_on)
    {
        return Err("prepared_data_invalid");
    }
    let recipe = ctx
        .recipes
        .ok_or("recipe_store_unavailable")?
        .get(&input.recipe_id, Some(input.revision))
        .map_err(|_| "recipe_revision_not_found")?;
    let planned_portions = if let Some(source) = &input.source {
        if !source_valid(source)
            || data
                .inventory
                .prepared
                .iter()
                .any(|b| !b.reversed && b.source.as_ref() == Some(source))
        {
            return Err("prepared_production_already_recorded");
        }
        Some(
            ctx.plans
                .production(&source.date, &source.meal, &input.recipe_id, input.revision)
                .ok_or("prepared_source_changed")?,
        )
    } else {
        None
    };
    let b = Batch {
        id: operation.to_owned(),
        recipe_id: recipe.id,
        revision: recipe.revision,
        title: recipe.recipe.title,
        original_portions: input.portions,
        produced_on: input.produced_on.clone(),
        storage: input.storage.clone(),
        source: input.source.clone(),
        planned_portions,
        allocations: Vec::new(),
        reversed: false,
    };
    if !b.validate() {
        return Err("prepared_data_invalid");
    }
    data.inventory.prepared.push(b);
    Ok(())
}
fn review(
    ctx: &Context<'_>,
    batch: &Batch,
    people: &[String],
    date: &str,
    meal: &str,
) -> Result<(), &'static str> {
    if people.iter().any(|id| !ctx.plans.attending(id, date, meal))
        || !crate::household::people::planning_conflicts(ctx.profiles, people, date, meal, 0)
            .is_empty()
    {
        return Err("attendance_excluded");
    }
    let recipe = ctx
        .recipes
        .ok_or("recipe_store_unavailable")?
        .get(&batch.recipe_id, Some(batch.revision))
        .map_err(|_| "recipe_revision_not_found")?;
    let evidence = crate::household::review_snapshot(
        ctx.state,
        ctx.profiles,
        &crate::household::Evaluation {
            member_ids: people.to_vec(),
            ingredient_ids: recipe
                .recipe
                .lines
                .iter()
                .map(|l| l.ingredient_id.clone())
                .collect(),
        },
        ctx.ingredients,
        Some(&recipe.catalogue),
    )?;
    if evidence["status"] == "blocked" {
        return Err("dietary_exclusion");
    }
    Ok(())
}
