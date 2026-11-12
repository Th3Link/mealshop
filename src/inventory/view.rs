use super::offers;
use crate::{
    cooking::RunState,
    shopping::model::{Data as Shopping, decimal},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(super) fn lot_rows(data: &Shopping) -> Result<Vec<Value>, &'static str> {
    let balances = data.inventory.balances()?;
    Ok(data.inventory.lots.iter().map(|lot|{
        let balance=balances.get(&lot.id).copied().unwrap_or(0);let held=data.inventory.held(&lot.id);
        json!({"id":lot.id,"binding":lot.binding,"original_quantity":lot.original_quantity,"unit":lot.unit,"metadata":lot.metadata,"archived":lot.archived,"remaining":decimal(balance),"reserved":decimal(held),"available":decimal(balance.saturating_sub(held))})
    }).collect())
}
pub(super) fn view(
    data: &Shopping,
    ctx: &super::operations::Context<'_>,
    offset: usize,
    limit: usize,
    history_offset: usize,
    history_limit: usize,
) -> Result<Value, &'static str> {
    let rows = lot_rows(data)?;
    let members = ctx
        .profiles
        .members
        .iter()
        .map(|m| m.id.clone())
        .collect::<Vec<_>>();
    let offers=data.inventory.offers.iter().map(|offer|Ok(json!({"offer":offer,"safety":offers::safety(ctx.state,ctx.profiles,ctx.ingredients,&offer.binding,&members)?,"shop_available":data.shops.entries.iter().any(|s|s.id==offer.shop_id)}))).collect::<Result<Vec<Value>,&'static str>>()?;
    let movements=data.inventory.movements.iter().rev().take(100).map(|m|{
        let op=data.inventory.operations.iter().find(|op|op.id==m.id);
        json!({"id":m.id,"kind":m.kind,"entries":m.entries,"reversed":data.inventory.reversed(&m.id),"actor":op.map(|o|&o.actor),"recorded_at":op.map(|o|o.recorded_at)})
    }).collect::<Vec<_>>();
    let purchases = data
        .purchases
        .iter()
        .filter(|p| !p.reversed && !data.inventory.purchase_booked(&p.id))
        .collect::<Vec<_>>();
    let runs=ctx.cooking.data.runs.iter().filter(|r|r.status==RunState::Completed).map(|r|json!({"id":r.id,"name":r.name,"recipe":r.recipe,"servings":r.servings,"plan_slot":r.plan_slot,"booked":data.inventory.cooking_booked(&r.id)})).collect::<Vec<_>>();
    Ok(
        json!({"revision":data.revision,"lots":rows.iter().skip(offset).take(limit).collect::<Vec<_>>(),"offset":offset,"limit":limit,"total":rows.len(),"allocations":data.inventory.allocations,"movements":movements,"movement_total":data.inventory.movements.len(),"movement_offset":history_offset,"movement_limit":history_limit,"offers":offers,"purchases":purchases,"completed_runs":runs,"cooking_selection":null}),
    )
}
pub(super) fn preview(before: &Shopping, after: &Shopping) -> Result<Value, &'static str> {
    let old = lot_rows(before)?
        .into_iter()
        .map(|lot| (lot["id"].as_str().unwrap_or("").to_owned(), lot))
        .collect::<BTreeMap<_, _>>();
    let changes = lot_rows(after)?
        .into_iter()
        .filter(|lot| old.get(lot["id"].as_str().unwrap_or("")) != Some(lot))
        .map(|lot| json!({"before":old.get(lot["id"].as_str().unwrap_or("")),"after":lot}))
        .collect::<Vec<_>>();
    let prior = before
        .rows()
        .into_iter()
        .map(|row| (row["key"].as_str().unwrap_or("").to_owned(), row))
        .collect::<BTreeMap<_, _>>();
    let shopping = after
        .rows()
        .into_iter()
        .filter(|row| prior.get(row["key"].as_str().unwrap_or("")) != Some(row))
        .map(|row| json!({"before":prior.get(row["key"].as_str().unwrap_or("")),"after":row}))
        .collect::<Vec<_>>();
    let allocations = after
        .inventory
        .allocations
        .iter()
        .filter(|a| {
            before
                .inventory
                .allocations
                .iter()
                .find(|old| old.id == a.id)
                .is_none_or(|old| old.state != a.state || old.amount != a.amount)
        })
        .collect::<Vec<_>>();
    Ok(
        json!({"expected_revision":before.revision,"changes":changes,"shopping":shopping,"allocations":allocations,"movements":after.inventory.movements.iter().skip(before.inventory.movements.len()).collect::<Vec<_>>(),"offers":after.inventory.offers.iter().filter(|o|before.inventory.offers.iter().find(|old|old.id==o.id).is_none_or(|old|serde_json::to_value(old).ok()!=serde_json::to_value(o).ok())).collect::<Vec<_>>()}),
    )
}
