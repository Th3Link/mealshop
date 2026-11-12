use super::{
    lots,
    model::{Allocation, AllocationState, Binding, Metadata, Movement, MovementKind},
};
use crate::shopping::model::{Data as Shopping, Quantity, Unit, amount};
fn quantity(value: &str, unit: Unit) -> Quantity {
    Quantity {
        amount: crate::quantities::Amount::try_from(value.to_owned()).unwrap(),
        unit,
    }
}
fn metadata() -> Metadata {
    Metadata {
        location: "Pantry".to_owned(),
        batch: String::new(),
        expiry: None,
    }
}
fn unmatched() -> Binding {
    Binding {
        ingredient_id: None,
        name: "Synthetic stock".to_owned(),
        translations: std::collections::BTreeMap::default(),
        product: None,
    }
}
#[test]
fn exact_lots_preserve_input_and_never_convert_between_dimensions() {
    let mut data = Shopping::default();
    let entry = lots::create(
        &mut data,
        unmatched(),
        quantity("0.100500", Unit::Kg),
        metadata(),
    )
    .unwrap();
    assert_eq!(entry.amount.atoms(), 100_500_000);
    assert_eq!(
        String::from(data.inventory.lots[0].original_quantity.amount.clone()),
        "0.100500"
    );
    let id = data.allocate("stock-op").unwrap();
    data.inventory.movements.push(Movement {
        id,
        kind: MovementKind::Opening,
        entries: vec![entry.clone()],
    });
    assert!(
        lots::adjust(
            &mut data,
            &entry.lot_id,
            quantity("100", Unit::Ml),
            "Observed count".to_owned()
        )
        .is_err()
    );
    let (kind, entries) = lots::adjust(
        &mut data,
        &entry.lot_id,
        quantity("0", Unit::G),
        "Empty container".to_owned(),
    )
    .unwrap();
    let id = data.allocate("stock-op").unwrap();
    data.inventory.movements.push(Movement {
        id: id.clone(),
        kind,
        entries,
    });
    assert_eq!(data.inventory.balances().unwrap()[&entry.lot_id], 0);
    let (kind, entries) = lots::reverse(&mut data, &id, "Count corrected".to_owned()).unwrap();
    let id = data.allocate("stock-op").unwrap();
    data.inventory
        .movements
        .push(Movement { id, kind, entries });
    assert_eq!(
        data.inventory.balances().unwrap()[&entry.lot_id],
        100_500_000
    );
}
#[test]
fn observed_stock_cannot_erase_another_reservation() {
    let mut data = Shopping::default();
    let entry = lots::create(&mut data, unmatched(), quantity("100", Unit::G), metadata()).unwrap();
    let id = data.allocate("stock-op").unwrap();
    data.inventory.movements.push(Movement {
        id,
        kind: MovementKind::Opening,
        entries: vec![entry.clone()],
    });
    data.inventory.allocations.push(Allocation {
        id: "allocation-3".to_owned(),
        lot_id: entry.lot_id.clone(),
        source_id: "source-4".to_owned(),
        key: "manual:source-4:g".to_owned(),
        origin: crate::shopping::model::Origin::Manual {
            label: "Synthetic demand".to_owned(),
            quantity: quantity("60", Unit::G),
        },
        use_on: "2026-11-12".to_owned(),
        amount: amount(60_000_000).unwrap(),
        state: AllocationState::Held,
        movement_id: None,
    });
    assert!(
        lots::adjust(
            &mut data,
            &entry.lot_id,
            quantity("59", Unit::G),
            "Observed quantity".to_owned()
        )
        .is_err()
    );
    assert_eq!(
        data.inventory.balances().unwrap()[&entry.lot_id],
        100_000_000
    );
}
