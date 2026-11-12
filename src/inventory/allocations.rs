use super::model::{Allocation, AllocationState, Lot};
use crate::shopping::model::amount;
use crate::shopping::{
    model::{Data as Shopping, Origin, Requirement, Source},
    products::gtin,
};
use std::collections::BTreeSet;

pub(crate) fn same_origin(a: &Origin, b: &Origin) -> bool {
    match (a, b) {
        (
            Origin::Recipe {
                recipe_id: a,
                revision: ar,
                ..
            },
            Origin::Recipe {
                recipe_id: b,
                revision: br,
                ..
            },
        ) => a == b && ar == br,
        (
            Origin::Meal {
                date: a,
                meal: am,
                recipe_id: ai,
                revision: ar,
                ..
            },
            Origin::Meal {
                date: b,
                meal: bm,
                recipe_id: bi,
                revision: br,
                ..
            },
        ) => a == b && am == bm && ai == bi && ar == br,
        (Origin::Manual { .. }, Origin::Manual { .. }) => true,
        (Origin::Product { product: a, .. }, Origin::Product { product: b, .. }) => {
            gtin(&a.barcode) == gtin(&b.barcode)
        }
        (Origin::Offer { offer: a, .. }, Origin::Offer { offer: b, .. }) => a.id == b.id,
        _ => false,
    }
}
pub(crate) fn coverage(
    data: &Shopping,
    source: &Source,
    key: &str,
    state: AllocationState,
) -> u128 {
    data.inventory
        .allocations
        .iter()
        .filter(|a| {
            a.source_id == source.id
                && a.key == key
                && a.state == state
                && same_origin(&a.origin, &source.origin)
        })
        .map(|a| a.amount.atoms())
        .sum()
}
pub(crate) fn covered(data: &Shopping, source: &Source, key: &str) -> u128 {
    coverage(data, source, key, AllocationState::Held)
        + coverage(data, source, key, AllocationState::Consumed)
}
pub(crate) fn release_source(data: &mut Shopping, id: &str) {
    for a in &mut data.inventory.allocations {
        if a.source_id == id && a.state == AllocationState::Held {
            a.state = AllocationState::Released;
        }
    }
}
pub(crate) fn matches(lot: &Lot, source: &Source, requirement: &Requirement) -> bool {
    if lot.archived || lot.unit != requirement.item.unit {
        return false;
    }
    let product = match &source.origin {
        Origin::Product { product, .. } => Some(product),
        Origin::Offer { offer, .. } => offer.binding.product.as_ref(),
        _ => None,
    };
    if let Some(product) = product {
        return lot
            .binding
            .product
            .as_ref()
            .is_some_and(|p| gtin(&p.barcode) == gtin(&product.barcode));
    }
    requirement.item.ingredient_id.is_some()
        && lot.binding.ingredient_id == requirement.item.ingredient_id
}
pub(crate) fn reconcile(data: &mut Shopping) -> Result<(), &'static str> {
    let mut release = BTreeSet::new();
    for source in &data.sources {
        for a in data
            .inventory
            .allocations
            .iter()
            .filter(|a| a.source_id == source.id && a.state == AllocationState::Held)
        {
            let lot = data
                .inventory
                .lots
                .iter()
                .find(|l| l.id == a.lot_id)
                .ok_or("inventory_data_invalid")?;
            let req = source.requirements.iter().find(|r| r.item.key == a.key);
            if !source.active
                || !same_origin(&a.origin, &source.origin)
                || lot
                    .metadata
                    .expiry
                    .as_ref()
                    .is_some_and(|date| date < &a.use_on)
                || req.is_none_or(|r| !matches(lot, source, r))
            {
                release.insert(a.id.clone());
            }
        }
        for req in &source.requirements {
            let pending = req.amount.atoms().saturating_sub(
                data.purchased(&source.id, &req.item.key)
                    + coverage(data, source, &req.item.key, AllocationState::Consumed),
            );
            if coverage(data, source, &req.item.key, AllocationState::Held) > pending {
                release.extend(
                    data.inventory
                        .allocations
                        .iter()
                        .filter(|a| {
                            a.source_id == source.id
                                && a.key == req.item.key
                                && a.state == AllocationState::Held
                        })
                        .map(|a| a.id.clone()),
                );
            }
        }
    }
    for a in &mut data.inventory.allocations {
        if release.contains(&a.id) {
            a.state = AllocationState::Released;
        }
    }
    Ok(())
}
pub(super) fn ordered_lots(data: &Shopping) -> Vec<Lot> {
    let mut lots = data
        .inventory
        .lots
        .iter()
        .filter(|l| !l.archived)
        .cloned()
        .collect::<Vec<_>>();
    lots.sort_by(|a, b| {
        a.metadata
            .expiry
            .is_none()
            .cmp(&b.metadata.expiry.is_none())
            .then_with(|| a.metadata.expiry.cmp(&b.metadata.expiry))
            .then_with(|| a.id.cmp(&b.id))
    });
    lots
}
pub(super) fn selection(ids: &[String], max: usize) -> Result<(), &'static str> {
    if ids.is_empty() || ids.len() > max || ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
        return Err("inventory_selection_invalid");
    }
    Ok(())
}
pub(super) fn reserve(
    data: &mut Shopping,
    ids: &[String],
    as_of: &str,
) -> Result<(), &'static str> {
    selection(ids, 100)?;
    if !crate::planning::date(as_of) {
        return Err("inventory_date_invalid");
    }
    let sources = ids
        .iter()
        .map(|id| {
            data.sources
                .iter()
                .find(|s| s.id == *id && s.active)
                .cloned()
                .ok_or("shopping_source_not_found")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let balances = data.inventory.balances()?;
    let lots = ordered_lots(data);
    let before = data.inventory.allocations.len();
    for source in &sources {
        let use_on = match &source.origin {
            Origin::Meal { date, .. } if date.as_str() > as_of => date.as_str(),
            _ => as_of,
        };
        for requirement in &source.requirements {
            let mut pending = requirement.amount.atoms().saturating_sub(
                data.purchased(&source.id, &requirement.item.key)
                    + covered(data, source, &requirement.item.key),
            );
            for lot in &lots {
                if pending == 0 {
                    break;
                }
                if !matches(lot, source, requirement)
                    || lot
                        .metadata
                        .expiry
                        .as_ref()
                        .is_some_and(|date| date.as_str() < use_on)
                {
                    continue;
                }
                let available = balances
                    .get(&lot.id)
                    .copied()
                    .unwrap_or(0)
                    .saturating_sub(data.inventory.held(&lot.id));
                let n = pending.min(available);
                if n == 0 {
                    continue;
                }
                let id = data.allocate("allocation")?;
                data.inventory.allocations.push(Allocation {
                    id,
                    lot_id: lot.id.clone(),
                    source_id: source.id.clone(),
                    key: requirement.item.key.clone(),
                    origin: source.origin.clone(),
                    use_on: use_on.to_owned(),
                    amount: amount(n)?,
                    state: AllocationState::Held,
                    movement_id: None,
                });
                pending -= n;
            }
        }
    }
    if before == data.inventory.allocations.len() {
        return Err("inventory_no_stock");
    }
    Ok(())
}
