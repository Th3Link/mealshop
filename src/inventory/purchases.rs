use super::{
    allocations, lots,
    model::{Binding, Data, Entry, Metadata, MovementKind},
};
use crate::shopping::model::{Data as Shopping, Origin, Quantity};
impl Data {
    pub(crate) fn purchase_booked(&self, id: &str) -> bool {
        self.movements.iter().any(|m|!self.reversed(&m.id)&&matches!(&m.kind,MovementKind::Purchase{purchases} if purchases.iter().any(|p|p.id==id)))
    }
}
pub(super) fn receive(
    data: &mut Shopping,
    ids: &[String],
    metadata: &Metadata,
) -> Result<(MovementKind, Vec<Entry>), &'static str> {
    allocations::selection(ids, 100)?;
    let purchases = ids
        .iter()
        .map(|id| {
            data.purchases
                .iter()
                .find(|p| p.id == *id && !p.reversed)
                .cloned()
                .ok_or("inventory_purchase_unavailable")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut entries = Vec::new();
    for purchase in &purchases {
        if data.inventory.purchase_booked(&purchase.id) {
            return Err("inventory_purchase_booked");
        }
        let product = match &purchase.origin {
            Origin::Product { product, .. } => Some(product.clone()),
            Origin::Offer { offer, .. } => offer.binding.product.clone(),
            _ => None,
        };
        let binding = Binding {
            ingredient_id: purchase.item.ingredient_id.clone(),
            name: purchase.item.name.clone(),
            translations: purchase.item.translations.clone(),
            product,
        };
        entries.push(lots::create(
            data,
            binding,
            Quantity {
                amount: purchase.amount.clone(),
                unit: purchase.item.unit,
            },
            metadata.clone(),
        )?);
    }
    Ok((MovementKind::Purchase { purchases }, entries))
}
