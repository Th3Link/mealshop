use super::{
    lots,
    model::{AllocationState, Binding, MovementKind, Offer},
};
use crate::shopping::{
    model::{Data as Shopping, text},
    products,
};
use std::collections::BTreeSet;
pub(crate) fn binding(value: &Binding) -> bool {
    text(&value.name, 200)
        && value.translations.len() <= 20
        && value
            .translations
            .iter()
            .all(|(k, v)| text(k, 20) && text(v, 200))
        && value.ingredient_id.as_ref().is_none_or(|id| text(id, 120))
        && value
            .product
            .as_ref()
            .is_none_or(|p| products::product_valid(p) && p.ingredient_id == value.ingredient_id)
}
pub(crate) fn offer(value: &Offer) -> bool {
    text(&value.id, 80)
        && text(&value.shop_id, 80)
        && text(&value.shop_name, 120)
        && binding(&value.binding)
        && (value.binding.ingredient_id.is_some() || value.binding.product.is_some())
        && value.quantity.normalized().is_ok()
        && value.price.atoms() > 0
        && value.price.atoms() % 10_000 == 0
        && matches!(value.currency.as_str(), "EUR" | "GBP" | "USD" | "CHF")
        && crate::planning::date(&value.valid_from)
        && crate::planning::date(&value.valid_until)
        && value.valid_from <= value.valid_until
        && text(&value.source, 1000)
        && value.source_url.as_ref().is_none_or(|url| {
            (crate::recipe_origin::Origin::Imported {
                source_url: url.clone(),
            })
            .validate()
            .is_ok()
        })
}
pub(crate) fn validate(data: &Shopping) -> Result<(), &'static str> {
    super::prepared::validate(data)?;
    let state = &data.inventory;
    if state.lots.len() > 500
        || state.allocations.len() > 2000
        || state.movements.len() > 1000
        || state.operations.len() > 2000
        || state.offers.len() > 200
    {
        return Err("inventory_capacity");
    }

    let mut ids = BTreeSet::new();
    for lot in &state.lots {
        if !ids.insert(&lot.id)
            || !valid(data, &lot.id, "lot-")
            || !binding(&lot.binding)
            || !lots::metadata(&lot.metadata)
            || lot.original_quantity.normalized().is_err()
            || lot.unit != lot.original_quantity.unit.base().0
        {
            return Err("inventory_data_invalid");
        }
    }
    let mut movement_ids = BTreeSet::new();
    let mut reversed = BTreeSet::new();
    for m in &state.movements {
        if !movement_ids.insert(&m.id)
            || !valid(data, &m.id, "stock-op-")
            || m.entries.is_empty()
            || m.entries.len() > 200
            || m.entries
                .iter()
                .any(|e| !ids.contains(&e.lot_id) || e.amount.atoms() == 0)
        {
            return Err("inventory_data_invalid");
        }
        if let MovementKind::Reversal {
            movement_id,
            reason,
        } = &m.kind
        {
            let original = state
                .movements
                .iter()
                .take_while(|old| old.id != m.id)
                .find(|old| old.id == *movement_id)
                .ok_or("inventory_data_invalid")?;
            if !text(reason, 500)
                || !reversed.insert(movement_id)
                || matches!(original.kind, MovementKind::Reversal { .. })
                || original.entries.len() != m.entries.len()
                || original.entries.iter().zip(&m.entries).any(|(a, b)| {
                    a.lot_id != b.lot_id || a.direction == b.direction || a.amount != b.amount
                })
            {
                return Err("inventory_data_invalid");
            }
        }
        if let MovementKind::Adjustment { reason, observed } = &m.kind {
            if !text(reason, 500) || m.entries.len() != 1 || lots::normalize(observed).is_err() {
                return Err("inventory_data_invalid");
            }
        }
    }
    super::ledger::validate(data)?;
    let balances = state.balances()?;
    if state.lots.iter().any(|l| {
        state.held(&l.id) > *balances.get(&l.id).unwrap_or(&0)
            || (l.archived && *balances.get(&l.id).unwrap_or(&0) != 0)
    }) {
        return Err("inventory_reserved");
    }
    validate_allocations(data, &ids, &movement_ids)?;
    let mut canonical = data.clone();
    super::allocations::reconcile(&mut canonical)?;
    if state
        .allocations
        .iter()
        .zip(&canonical.inventory.allocations)
        .any(|(a, b)| a.state != b.state)
    {
        return Err("inventory_data_invalid");
    }
    validate_receipts(data)?;
    let mut offers = BTreeSet::new();
    if state
        .offers
        .iter()
        .any(|o| !offers.insert(&o.id) || !valid(data, &o.id, "offer-") || !offer(o))
    {
        return Err("inventory_data_invalid");
    }
    Ok(())
}

fn validate_receipts(data: &Shopping) -> Result<(), &'static str> {
    let state = &data.inventory;
    let mut operation_ids = BTreeSet::new();
    let mut revisions = BTreeSet::new();
    let mut keys = BTreeSet::new();
    for op in &state.operations {
        if !operation_ids.insert(&op.id)
            || !valid(data, &op.id, "stock-op-")
            || !keys.insert((&op.actor, &op.key))
            || !crate::membership::valid_user(&op.actor)
            || !crate::recipes::Creation::valid_key(&op.key)
            || op.recorded_at == 0
            || op.expected_revision >= data.revision
            || !revisions.insert(op.expected_revision)
        {
            return Err("inventory_data_invalid");
        }
    }
    if state
        .movements
        .iter()
        .map(|m| &m.id)
        .any(|id| !operation_ids.contains(id))
    {
        return Err("inventory_data_invalid");
    }
    Ok(())
}

fn valid(data: &Shopping, id: &str, prefix: &str) -> bool {
    id.strip_prefix(prefix)
        .and_then(|n| n.parse::<u64>().ok())
        .is_some_and(|n| n > 0 && n < data.next_id && id == format!("{prefix}{n}"))
}

fn validate_allocations(
    data: &Shopping,
    ids: &BTreeSet<&String>,
    movement_ids: &BTreeSet<&String>,
) -> Result<(), &'static str> {
    let state = &data.inventory;
    let mut allocation_ids = BTreeSet::new();
    for a in &state.allocations {
        if !allocation_ids.insert(&a.id)
            || !valid(data, &a.id, "allocation-")
            || !ids.contains(&a.lot_id)
            || !data.sources.iter().any(|s| s.id == a.source_id)
            || a.amount.atoms() == 0
            || !text(&a.key, 300)
            || !crate::planning::date(&a.use_on)
            || a.movement_id
                .as_ref()
                .is_some_and(|id| !movement_ids.contains(id))
            || (a.state == AllocationState::Consumed && a.movement_id.is_none())
            || (a.state == AllocationState::Held && a.movement_id.is_some())
            || !crate::shopping::store::origin_valid(&a.origin)
        {
            return Err("inventory_data_invalid");
        }
    }
    Ok(())
}
