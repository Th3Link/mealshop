use super::{
    allocations, cooking, lots,
    model::{AllocationState, Command, Data, Movement, Operation},
    offers, purchases,
};
use crate::{
    household::{Household, Profiles},
    shopping::model::Data as Shopping,
};

pub(super) struct Context<'a> {
    pub(super) state: &'a Household,
    pub(super) profiles: &'a Profiles,
    pub(super) ingredients: &'a crate::ingredients::Store,
    pub(super) recipes: Option<&'a crate::recipes::Store>,
    pub(super) cooking: &'a crate::cooking::store::Store,
    pub(super) now: u64,
}
impl Data {
    pub(crate) fn replay(
        &self,
        actor: &str,
        key: &str,
        expected: u64,
        command: &Command,
    ) -> Result<Option<String>, &'static str> {
        let Some(op) = self
            .operations
            .iter()
            .find(|op| op.actor == actor && op.key == key)
        else {
            return Ok(None);
        };
        if op.expected_revision != expected
            || serde_json::to_value(&op.command).map_err(|_| "inventory_storage_failed")?
                != serde_json::to_value(command).map_err(|_| "inventory_storage_failed")?
        {
            return Err("inventory_request_conflict");
        }
        Ok(Some(op.id.clone()))
    }
}
pub(super) fn execute(
    data: &mut Shopping,
    ctx: &Context<'_>,
    actor: &str,
    key: &str,
    expected: u64,
    command: Command,
) -> Result<String, &'static str> {
    if !crate::recipes::Creation::valid_key(key) {
        return Err("inventory_key_invalid");
    }
    let id = data.allocate("stock-op")?;
    let mut movement = None;
    match &command {
        Command::CreateLot { .. }
        | Command::UpdateLot { .. }
        | Command::AdjustLot { .. }
        | Command::ArchiveLot { .. } => {
            movement = lots::execute(data, &ctx.ingredients.catalogue(), &command)?;
        }
        Command::Reserve { source_ids, as_of } => allocations::reserve(data, source_ids, as_of)?,
        Command::Release { allocation_ids } => {
            allocations::selection(allocation_ids, 100)?;
            for id in allocation_ids {
                let allocation = data
                    .inventory
                    .allocations
                    .iter_mut()
                    .find(|a| a.id == *id && a.state == AllocationState::Held)
                    .ok_or("inventory_allocation_unavailable")?;
                allocation.state = AllocationState::Released;
            }
        }
        Command::Receive {
            purchase_ids,
            metadata,
        } => movement = Some(purchases::receive(data, purchase_ids, metadata)?),
        Command::Consume {
            run_id,
            source_ids,
            include_unreserved,
            as_of,
        } => {
            let run = ctx.cooking.data.run(run_id)?;
            movement = Some(cooking::consume(
                data,
                ctx.recipes.ok_or("recipe_store_unavailable")?,
                run,
                source_ids,
                *include_unreserved,
                &id,
                as_of,
            )?);
        }
        Command::Reverse {
            movement_id,
            reason,
        } => movement = Some(lots::reverse(data, movement_id, reason.clone())?),
        Command::SaveOffer { offer } => {
            offers::save(data, offer.clone(), &ctx.ingredients.catalogue())?;
        }
        Command::ArchiveOffer { offer_id } => {
            data.inventory
                .offers
                .iter_mut()
                .find(|o| o.id == *offer_id)
                .ok_or("inventory_offer_not_found")?
                .archived = true;
        }
        Command::AddOffer {
            offer_id,
            packages,
            as_of,
            member_ids,
        } => offers::add_checked(data, ctx, offer_id, *packages, as_of, member_ids)?,
    }
    if let Some((kind, entries)) = movement {
        data.inventory.movements.push(Movement {
            id: id.clone(),
            kind,
            entries,
        });
    }
    data.inventory.operations.push(Operation {
        id: id.clone(),
        actor: actor.to_owned(),
        key: key.to_owned(),
        expected_revision: expected,
        command,
        recorded_at: ctx.now,
    });
    Ok(id)
}
