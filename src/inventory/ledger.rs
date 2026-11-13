use super::{
    lots,
    model::{AllocationState, Direction, MovementKind},
};
use crate::shopping::model::{Data as Shopping, Unit};
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn validate(data: &Shopping) -> Result<(), &'static str> {
    let state = &data.inventory;
    let mut created = BTreeSet::new();
    let mut balances: BTreeMap<String, i128> =
        state.lots.iter().map(|l| (l.id.clone(), 0)).collect();
    let mut receipts = BTreeSet::new();
    let mut runs = BTreeSet::new();
    for movement in &state.movements {
        let opening = matches!(
            movement.kind,
            MovementKind::Opening | MovementKind::Purchase { .. }
        );
        if matches!(movement.kind, MovementKind::Opening) && movement.entries.len() != 1 {
            return Err("inventory_data_invalid");
        }
        for e in &movement.entries {
            let lot = state
                .lots
                .iter()
                .find(|l| l.id == e.lot_id)
                .ok_or("inventory_data_invalid")?;
            if opening {
                if !created.insert(&e.lot_id)
                    || e.direction != Direction::In
                    || lots::normalize(&lot.original_quantity)?.1.atoms() != e.amount.atoms()
                {
                    return Err("inventory_data_invalid");
                }
            } else if !created.contains(&e.lot_id) {
                return Err("inventory_data_invalid");
            }
            let n = i128::try_from(e.amount.atoms()).map_err(|_| "inventory_data_invalid")?;
            let balance = balances
                .get_mut(&e.lot_id)
                .ok_or("inventory_data_invalid")?;
            *balance += if e.direction == Direction::In { n } else { -n };
            if !(0..=1_000_000_000_000_000).contains(balance) {
                return Err("inventory_insufficient");
            }
        }
        match &movement.kind {
            MovementKind::Adjustment { observed, .. } => {
                let entry = movement.entries.first().ok_or("inventory_data_invalid")?;
                let lot = state
                    .lots
                    .iter()
                    .find(|l| l.id == entry.lot_id)
                    .ok_or("inventory_data_invalid")?;
                let (unit, n) = lots::normalize(observed)?;
                if unit != lot.unit
                    || balances.get(&lot.id).copied() != i128::try_from(n.atoms()).ok()
                {
                    return Err("inventory_data_invalid");
                }
            }
            MovementKind::Purchase { .. } => validate_purchases(data, movement, &mut receipts)?,
            MovementKind::Cooking { .. } => validate_cooking(state, movement, &mut runs)?,
            _ => {}
        }
    }
    if created.len() != state.lots.len() {
        return Err("inventory_data_invalid");
    }
    validate_coverage(state)
}
fn validate_cooking(
    state: &super::model::Data,
    movement: &super::model::Movement,
    runs: &mut BTreeSet<String>,
) -> Result<(), &'static str> {
    let MovementKind::Cooking {
        run_id,
        recipe_id,
        revision,
        servings,
        requirements,
        unbooked,
        use_on,
    } = &movement.kind
    else {
        return Err("inventory_data_invalid");
    };

    if !crate::planning::date(use_on)
        || !crate::recipes::valid_id(recipe_id)
        || *revision == 0
        || !(1..=1_000_000).contains(servings)
        || requirements.is_empty()
        || requirements.len() > 100
        || (!state.reversed(&movement.id) && !runs.insert(run_id.clone()))
    {
        return Err("inventory_data_invalid");
    }
    let mut consumed: BTreeMap<&str, u128> = BTreeMap::new();
    for entry in &movement.entries {
        let lot = state
            .lots
            .iter()
            .find(|l| l.id == entry.lot_id)
            .ok_or("inventory_data_invalid")?;
        let ingredient = lot
            .binding
            .ingredient_id
            .as_deref()
            .ok_or("inventory_data_invalid")?;
        if entry.direction != Direction::Out || lot.unit != Unit::G {
            return Err("inventory_data_invalid");
        }
        *consumed.entry(ingredient).or_default() += entry.amount.atoms();
    }
    let mut ingredients = BTreeSet::new();
    for req in requirements {
        let ingredient = req
            .item
            .ingredient_id
            .as_deref()
            .ok_or("inventory_data_invalid")?;
        if !ingredients.insert(ingredient)
            || req.item.unit != Unit::G
            || req.item.key != format!("ingredient:{ingredient}:g")
        {
            return Err("inventory_data_invalid");
        }
        let used = consumed.remove(ingredient).unwrap_or(0);
        let missing = unbooked
            .iter()
            .find(|r| r.item.key == req.item.key)
            .map_or(0, |r| r.amount.atoms());
        if used + missing != req.amount.atoms() {
            return Err("inventory_data_invalid");
        }
    }
    if !consumed.is_empty()
        || unbooked.len() > requirements.len()
        || unbooked
            .iter()
            .any(|r| !requirements.iter().any(|req| req.item.key == r.item.key))
    {
        return Err("inventory_data_invalid");
    }
    Ok(())
}
fn validate_purchases(
    data: &Shopping,
    movement: &super::model::Movement,
    receipts: &mut BTreeSet<String>,
) -> Result<(), &'static str> {
    let state = &data.inventory;
    let MovementKind::Purchase { purchases } = &movement.kind else {
        return Err("inventory_data_invalid");
    };

    if purchases.len() != movement.entries.len() {
        return Err("inventory_data_invalid");
    }
    for (purchase, entry) in purchases.iter().zip(&movement.entries) {
        let current = data
            .purchases
            .iter()
            .find(|p| p.id == purchase.id)
            .ok_or("inventory_data_invalid")?;
        let lot = state
            .lots
            .iter()
            .find(|l| l.id == entry.lot_id)
            .ok_or("inventory_data_invalid")?;
        if purchase.amount != entry.amount
            || purchase.item.unit != lot.unit
            || purchase.item.ingredient_id != lot.binding.ingredient_id
            || purchase.reversed
            || purchase.amount != current.amount
            || purchase.source_id != current.source_id
            || serde_json::to_value(&purchase.origin).ok()
                != serde_json::to_value(&current.origin).ok()
            || (!state.reversed(&movement.id)
                && (current.reversed || !receipts.insert(purchase.id.clone())))
        {
            return Err("inventory_data_invalid");
        }
    }
    Ok(())
}
fn validate_coverage(state: &super::model::Data) -> Result<(), &'static str> {
    let mut booked: BTreeMap<(&str, &str), u128> = BTreeMap::new();
    for a in &state.allocations {
        if a.state != AllocationState::Consumed {
            continue;
        }
        let id = a.movement_id.as_deref().ok_or("inventory_data_invalid")?;
        let movement = state
            .movements
            .iter()
            .find(|m| m.id == id)
            .ok_or("inventory_data_invalid")?;
        let MovementKind::Cooking { .. } = &movement.kind else {
            return Err("inventory_data_invalid");
        };
        let lot = state
            .lots
            .iter()
            .find(|l| l.id == a.lot_id)
            .ok_or("inventory_data_invalid")?;
        if state.reversed(id)
            || lot.unit != Unit::G
            || lot
                .binding
                .ingredient_id
                .as_ref()
                .is_none_or(|id| a.key != format!("ingredient:{id}:g"))
        {
            return Err("inventory_data_invalid");
        }
        *booked.entry((id, &a.lot_id)).or_default() += a.amount.atoms();
    }
    for ((movement, lot), n) in booked {
        let physical = state
            .movements
            .iter()
            .filter(|m| m.id == movement)
            .flat_map(|m| &m.entries)
            .filter(|e| e.lot_id == lot && e.direction == Direction::Out)
            .map(|e| e.amount.atoms())
            .sum::<u128>();
        if n > physical {
            return Err("inventory_data_invalid");
        }
    }
    Ok(())
}
