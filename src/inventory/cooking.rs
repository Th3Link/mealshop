use super::{
    allocations::{self, coverage},
    model::{Allocation, AllocationState, Data, Direction, Entry, MovementKind},
};
use crate::{
    cooking::{Run, RunState},
    shopping::{
        model::{Data as Shopping, Origin, Requirement, Source, Unit, amount},
        sources::{self, Selection},
    },
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn eligible(source: &Source, run: &Run, required: &[Requirement]) -> bool {
    if !source.active
        || !source
            .requirements
            .iter()
            .any(|r| required.iter().any(|need| need.item.key == r.item.key))
    {
        return false;
    }
    match &source.origin {
        Origin::Recipe {
            recipe_id,
            revision,
            ..
        } => *recipe_id == run.recipe.id && *revision == run.recipe.revision,
        Origin::Meal {
            date,
            meal,
            recipe_id,
            revision,
            ..
        } => {
            *recipe_id == run.recipe.id
                && *revision == run.recipe.revision
                && run
                    .plan_slot
                    .as_ref()
                    .is_some_and(|s| s.date == *date && s.meal == *meal)
        }
        Origin::Manual { .. } | Origin::Product { .. } | Origin::Offer { .. } => true,
    }
}
impl Data {
    pub(crate) fn cooking_booked(&self, id: &str) -> bool {
        self.movements.iter().any(|m| {
            !self.reversed(&m.id)
                && matches!(&m.kind,MovementKind::Cooking{run_id,..} if run_id==id)
        })
    }
}
fn use_held(
    data: &mut Shopping,
    index: usize,
    n: u128,
    movement_id: &str,
    use_on: &str,
) -> Result<(), &'static str> {
    let mut used = data.inventory.allocations[index].clone();
    if n < used.amount.atoms() {
        data.inventory.allocations[index].amount = amount(used.amount.atoms() - n)?;
        used.id = data.allocate("allocation")?;
        used.amount = amount(n)?;
        use_on.clone_into(&mut used.use_on);
        used.state = AllocationState::Consumed;
        used.movement_id = Some(movement_id.to_owned());
        data.inventory.allocations.push(used);
    } else {
        use_on.clone_into(&mut data.inventory.allocations[index].use_on);
        data.inventory.allocations[index].state = AllocationState::Consumed;
        data.inventory.allocations[index].movement_id = Some(movement_id.to_owned());
    }
    Ok(())
}

struct Posting<'a> {
    balances: BTreeMap<String, u128>,
    held: BTreeMap<String, u128>,
    debits: BTreeMap<String, u128>,
    sources: Vec<Source>,
    movement_id: &'a str,
    use_on: &'a str,
}
impl Posting<'_> {
    fn debit(&mut self, lot: &str, n: u128, held: bool) -> Result<(), &'static str> {
        let balance = self.balances.get_mut(lot).ok_or("inventory_data_invalid")?;
        *balance = balance.checked_sub(n).ok_or("inventory_insufficient")?;
        if held {
            let reserved = self.held.get_mut(lot).ok_or("inventory_data_invalid")?;
            *reserved = reserved.checked_sub(n).ok_or("inventory_data_invalid")?;
        }
        *self.debits.entry(lot.to_owned()).or_default() += n;
        Ok(())
    }
    fn consume_holds(
        &mut self,
        data: &mut Shopping,
        requirement: &Requirement,
        mut pending: u128,
    ) -> Result<u128, &'static str> {
        let indices = data
            .inventory
            .allocations
            .iter()
            .enumerate()
            .filter(|(_, a)| {
                a.state == AllocationState::Held
                    && a.key == requirement.item.key
                    && self.sources.iter().any(|s| s.id == a.source_id)
            })
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        for index in indices {
            if pending == 0 {
                break;
            }
            let allocation = data.inventory.allocations[index].clone();
            let lot = data
                .inventory
                .lots
                .iter()
                .find(|l| l.id == allocation.lot_id)
                .ok_or("inventory_data_invalid")?;
            if lot
                .metadata
                .expiry
                .as_ref()
                .is_some_and(|date| date.as_str() < self.use_on)
            {
                continue;
            }
            let n = pending.min(allocation.amount.atoms());
            use_held(data, index, n, self.movement_id, self.use_on)?;
            self.debit(&allocation.lot_id, n, true)?;
            pending -= n;
        }
        Ok(pending)
    }
    fn consume_free(
        &mut self,
        data: &mut Shopping,
        requirement: &Requirement,
        mut pending: u128,
    ) -> Result<u128, &'static str> {
        let use_on = self.use_on;
        let lots = allocations::ordered_lots(data).into_iter().filter(|l| {
            l.unit == Unit::G
                && l.binding.ingredient_id == requirement.item.ingredient_id
                && l.metadata
                    .expiry
                    .as_ref()
                    .is_none_or(|date| date.as_str() >= use_on)
        });
        for lot in lots {
            if pending == 0 {
                break;
            }
            let available = self
                .balances
                .get(&lot.id)
                .copied()
                .unwrap_or(0)
                .saturating_sub(self.held.get(&lot.id).copied().unwrap_or(0));
            let n = pending.min(available);
            if n == 0 {
                continue;
            }
            self.debit(&lot.id, n, false)?;
            pending -= n;
            let mut left = n;
            for source in &self.sources {
                if left == 0 {
                    break;
                }
                let Some(req) = source.requirements.iter().find(|r| {
                    r.item.key == requirement.item.key && allocations::matches(&lot, source, r)
                }) else {
                    continue;
                };
                let needed = req.amount.atoms().saturating_sub(
                    data.purchased(&source.id, &req.item.key)
                        + coverage(data, source, &req.item.key, AllocationState::Consumed)
                        + coverage(data, source, &req.item.key, AllocationState::Held),
                );
                let used = left.min(needed);
                if used == 0 {
                    continue;
                }
                let id = data.allocate("allocation")?;
                data.inventory.allocations.push(Allocation {
                    id,
                    lot_id: lot.id.clone(),
                    source_id: source.id.clone(),
                    key: req.item.key.clone(),
                    origin: source.origin.clone(),
                    use_on: self.use_on.to_owned(),
                    amount: amount(used)?,
                    state: AllocationState::Consumed,
                    movement_id: Some(self.movement_id.to_owned()),
                });
                left -= used;
            }
        }
        Ok(pending)
    }
}
pub(super) fn consume(
    data: &mut Shopping,
    recipes: &crate::recipes::Store,
    run: &Run,
    ids: &[String],
    unreserved: bool,
    movement_id: &str,
    today: &str,
) -> Result<(MovementKind, Vec<Entry>), &'static str> {
    if !crate::planning::date(today) {
        return Err("inventory_date_invalid");
    }
    if run.status != RunState::Completed {
        return Err("inventory_cooking_incomplete");
    }
    if data.inventory.cooking_booked(&run.id) {
        return Err("inventory_cooking_booked");
    }
    if ids.len() > 100 || ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
        return Err("inventory_selection_invalid");
    }
    let requirements = sources::recipe(
        recipes,
        &Selection {
            recipe_id: run.recipe.id.clone(),
            revision: run.recipe.revision,
            servings: run.servings,
        },
    )?
    .requirements;
    let selected = ids
        .iter()
        .map(|id| {
            data.sources
                .iter()
                .find(|s| s.id == *id && eligible(s, run, &requirements))
                .cloned()
                .ok_or("inventory_source_mismatch")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut posting = Posting {
        balances: data.inventory.balances()?,
        held: data
            .inventory
            .lots
            .iter()
            .map(|l| (l.id.clone(), data.inventory.held(&l.id)))
            .collect(),
        debits: BTreeMap::new(),
        sources: selected,
        movement_id,
        use_on: today,
    };
    let mut unbooked = Vec::new();
    for requirement in &requirements {
        let mut pending = posting.consume_holds(data, requirement, requirement.amount.atoms())?;
        if unreserved {
            pending = posting.consume_free(data, requirement, pending)?;
        }
        if pending > 0 {
            unbooked.push(Requirement {
                item: requirement.item.clone(),
                amount: amount(pending)?,
            });
        }
    }
    if posting.debits.is_empty() {
        return Err("inventory_no_stock");
    }
    let entries = posting
        .debits
        .into_iter()
        .map(|(lot_id, n)| {
            Ok(Entry {
                lot_id,
                direction: Direction::Out,
                amount: amount(n)?,
            })
        })
        .collect::<Result<_, &'static str>>()?;
    Ok((
        MovementKind::Cooking {
            run_id: run.id.clone(),
            recipe_id: run.recipe.id.clone(),
            revision: run.recipe.revision,
            servings: run.servings,
            requirements,
            unbooked,
            use_on: today.to_owned(),
        },
        entries,
    ))
}

pub(super) fn selection(
    data: &Shopping,
    recipes: &crate::recipes::Store,
    run: &Run,
) -> Result<serde_json::Value, &'static str> {
    if run.status != RunState::Completed {
        return Err("inventory_cooking_incomplete");
    }
    let requirements = sources::recipe(
        recipes,
        &Selection {
            recipe_id: run.recipe.id.clone(),
            revision: run.recipe.revision,
            servings: run.servings,
        },
    )?
    .requirements;
    let ids = data
        .sources
        .iter()
        .filter(|s| eligible(s, run, &requirements))
        .map(|s| &s.id)
        .collect::<Vec<_>>();
    Ok(serde_json::json!({"run_id":run.id,"requirements":requirements,"source_ids":ids}))
}
