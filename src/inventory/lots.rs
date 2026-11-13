use super::model::{
    AllocationState, Binding, Data, Direction, Entry, Lot, Metadata, MovementKind, Target,
};
use crate::shopping::{
    model::{Data as Shopping, Quantity, Unit, amount, text},
    products,
};
use std::collections::BTreeMap;

pub(super) fn normalize(
    quantity: &Quantity,
) -> Result<(Unit, crate::quantities::Amount), &'static str> {
    let (unit, factor) = quantity.unit.base();
    Ok((unit, amount(quantity.amount.atoms() * factor)?))
}
pub(super) fn metadata(value: &Metadata) -> bool {
    text(&value.location, 100)
        && (value.batch.is_empty() || text(&value.batch, 100))
        && value
            .expiry
            .as_ref()
            .is_none_or(|date| crate::planning::date(date))
}
pub(super) fn binding(
    target: &Target,
    data: &Shopping,
    catalogue: &crate::nutrition::catalogue::Catalogue,
) -> Result<Binding, &'static str> {
    match target {
        Target::Ingredient { ingredient_id } => {
            let item = catalogue.find(ingredient_id).ok_or("unknown_ingredient")?;
            Ok(Binding {
                ingredient_id: Some(item.id.clone()),
                name: item.name.clone(),
                translations: item
                    .translations
                    .iter()
                    .map(|(lang, tr)| (lang.clone(), tr.name.clone()))
                    .collect(),
                product: None,
            })
        }
        Target::Product { barcode } => {
            let product = data
                .products
                .records
                .get(&products::gtin(barcode)?)
                .ok_or("shopping_product_not_found")?
                .clone();
            Ok(Binding {
                ingredient_id: product.ingredient_id.clone(),
                name: product.name.clone(),
                translations: BTreeMap::new(),
                product: Some(product),
            })
        }
        Target::Unmatched { label } if text(label, 200) => Ok(Binding {
            ingredient_id: None,
            name: label.clone(),
            translations: BTreeMap::new(),
            product: None,
        }),
        Target::Unmatched { .. } => Err("inventory_target_invalid"),
    }
}
impl Data {
    pub(crate) fn balances(&self) -> Result<BTreeMap<String, u128>, &'static str> {
        let mut balances: BTreeMap<_, i128> =
            self.lots.iter().map(|lot| (lot.id.clone(), 0)).collect();
        for movement in &self.movements {
            for entry in &movement.entries {
                let balance = balances
                    .get_mut(&entry.lot_id)
                    .ok_or("inventory_data_invalid")?;
                let atoms =
                    i128::try_from(entry.amount.atoms()).map_err(|_| "inventory_data_invalid")?;
                *balance += if entry.direction == Direction::In {
                    atoms
                } else {
                    -atoms
                };
                if *balance < 0 || *balance > 1_000_000_000_000_000 {
                    return Err("inventory_insufficient");
                }
            }
        }
        balances
            .into_iter()
            .map(|(id, n)| {
                u128::try_from(n)
                    .map(|n| (id, n))
                    .map_err(|_| "inventory_data_invalid")
            })
            .collect()
    }
    pub(crate) fn held(&self, lot: &str) -> u128 {
        self.allocations
            .iter()
            .filter(|a| a.lot_id == lot && a.state == AllocationState::Held)
            .map(|a| a.amount.atoms())
            .sum()
    }
    pub(crate) fn reversed(&self, id: &str) -> bool {
        self.movements
            .iter()
            .any(|m| matches!(&m.kind,MovementKind::Reversal {movement_id,..} if movement_id==id))
    }
}
pub(super) fn create(
    data: &mut Shopping,
    target: Binding,
    quantity: Quantity,
    info: Metadata,
) -> Result<Entry, &'static str> {
    let (unit, normalized) = normalize(&quantity)?;
    if normalized.atoms() == 0 || !metadata(&info) {
        return Err("inventory_lot_invalid");
    }
    let id = data.allocate("lot")?;
    data.inventory.lots.push(Lot {
        id: id.clone(),
        binding: target,
        original_quantity: quantity,
        unit,
        metadata: info,
        archived: false,
    });
    Ok(Entry {
        lot_id: id,
        direction: Direction::In,
        amount: normalized,
    })
}
pub(super) fn adjust(
    data: &mut Shopping,
    id: &str,
    quantity: Quantity,
    reason: String,
) -> Result<(MovementKind, Vec<Entry>), &'static str> {
    if !text(&reason, 500) {
        return Err("inventory_reason_required");
    }
    let lot = data
        .inventory
        .lots
        .iter()
        .find(|l| l.id == id && !l.archived)
        .ok_or("inventory_lot_not_found")?;
    let (unit, next) = normalize(&quantity)?;
    if unit != lot.unit {
        return Err("inventory_dimension_mismatch");
    }
    let balance = *data
        .inventory
        .balances()?
        .get(id)
        .ok_or("inventory_lot_not_found")?;
    if next.atoms() < data.inventory.held(id) {
        return Err("inventory_reserved");
    }
    if next.atoms() == balance {
        return Err("inventory_no_change");
    }
    let (direction, delta) = if next.atoms() > balance {
        (Direction::In, next.atoms() - balance)
    } else {
        (Direction::Out, balance - next.atoms())
    };
    Ok((
        MovementKind::Adjustment {
            reason,
            observed: quantity,
        },
        vec![Entry {
            lot_id: id.to_owned(),
            direction,
            amount: amount(delta)?,
        }],
    ))
}
pub(super) fn reverse(
    data: &mut Shopping,
    id: &str,
    reason: String,
) -> Result<(MovementKind, Vec<Entry>), &'static str> {
    if !text(&reason, 500) {
        return Err("inventory_reason_required");
    }
    let movement = data
        .inventory
        .movements
        .iter()
        .find(|m| m.id == id)
        .ok_or("inventory_movement_not_found")?;
    if matches!(movement.kind, MovementKind::Reversal { .. }) || data.inventory.reversed(id) {
        return Err("inventory_already_reversed");
    }
    let entries = movement
        .entries
        .iter()
        .map(|e| Entry {
            lot_id: e.lot_id.clone(),
            amount: e.amount.clone(),
            direction: if e.direction == Direction::In {
                Direction::Out
            } else {
                Direction::In
            },
        })
        .collect();
    for entry in &movement.entries {
        if entry.direction == Direction::Out {
            if let Some(lot) = data
                .inventory
                .lots
                .iter_mut()
                .find(|l| l.id == entry.lot_id)
            {
                lot.archived = false;
            }
        }
    }
    for allocation in &mut data.inventory.allocations {
        if allocation.movement_id.as_deref() == Some(id) {
            allocation.state = AllocationState::Released;
        }
    }
    Ok((
        MovementKind::Reversal {
            movement_id: id.to_owned(),
            reason,
        },
        entries,
    ))
}

pub(super) fn execute(
    data: &mut Shopping,
    catalogue: &crate::nutrition::catalogue::Catalogue,
    command: &super::model::Command,
) -> Result<Option<(MovementKind, Vec<Entry>)>, &'static str> {
    use super::model::Command;
    match command {
        Command::CreateLot {
            target,
            quantity,
            metadata,
        } => {
            let binding = binding(target, data, catalogue)?;
            Ok(Some((
                MovementKind::Opening,
                vec![create(data, binding, quantity.clone(), metadata.clone())?],
            )))
        }
        Command::UpdateLot {
            lot_id,
            metadata: info,
        } => {
            if !metadata(info) {
                return Err("inventory_lot_invalid");
            }
            let lot = data
                .inventory
                .lots
                .iter_mut()
                .find(|l| l.id == *lot_id && !l.archived)
                .ok_or("inventory_lot_not_found")?;
            lot.metadata = info.clone();
            Ok(None)
        }
        Command::AdjustLot {
            lot_id,
            quantity,
            reason,
        } => adjust(data, lot_id, quantity.clone(), reason.clone()).map(Some),
        Command::ArchiveLot { lot_id } => {
            if data.inventory.balances()?.get(lot_id).copied() != Some(0)
                || data.inventory.held(lot_id) > 0
            {
                return Err("inventory_not_empty");
            }
            data.inventory
                .lots
                .iter_mut()
                .find(|l| l.id == *lot_id)
                .ok_or("inventory_lot_not_found")?
                .archived = true;
            Ok(None)
        }
        _ => Err("inventory_request_invalid"),
    }
}
