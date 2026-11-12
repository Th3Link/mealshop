use super::{
    lots,
    model::{Binding, Offer, OfferInput},
};
use crate::household::{Evaluation, Household, Profiles};
use crate::shopping::model::{Data as Shopping, Item, Origin, Requirement, Source, amount};

pub(super) fn safety(
    state: &Household,
    profiles: &Profiles,
    ingredients: &crate::ingredients::Store,
    binding: &Binding,
    member_ids: &[String],
) -> Result<serde_json::Value, &'static str> {
    if member_ids.is_empty() && !profiles.members.is_empty() {
        return Err("inventory_profiles_required");
    }
    if member_ids.len() > 100
        || member_ids
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != member_ids.len()
    {
        return Err("inventory_selection_invalid");
    }
    let review = crate::household::review_snapshot(
        state,
        profiles,
        &Evaluation {
            member_ids: member_ids.to_vec(),
            ingredient_ids: binding.ingredient_id.iter().cloned().collect(),
        },
        ingredients,
        None,
    )?;
    Ok(
        serde_json::json!({"status":review["status"],"unknown":true,"profile_revision":profiles.revision}),
    )
}
pub(super) fn save(
    data: &mut Shopping,
    input: OfferInput,
    catalogue: &crate::nutrition::catalogue::Catalogue,
) -> Result<(), &'static str> {
    let shop = data
        .shops
        .entries
        .iter()
        .find(|s| s.id == input.shop_id)
        .ok_or("shopping_shop_not_found")?;
    let name = shop.name.clone();
    let binding = lots::binding(&input.target, data, catalogue)?;
    let id = if let Some(id) = input.id {
        if !data.inventory.offers.iter().any(|o| o.id == id) {
            return Err("inventory_offer_not_found");
        }
        id
    } else {
        data.allocate("offer")?
    };
    let offer = Offer {
        id: id.clone(),
        shop_id: input.shop_id,
        shop_name: name,
        binding,
        quantity: input.quantity,
        price: input.price,
        currency: input.currency,
        valid_from: input.valid_from,
        valid_until: input.valid_until,
        source: input.source,
        source_url: input.source_url,
        archived: false,
    };
    if !super::validation::offer(&offer) {
        return Err("inventory_offer_invalid");
    }
    if let Some(current) = data.inventory.offers.iter_mut().find(|o| o.id == id) {
        *current = offer;
    } else {
        data.inventory.offers.push(offer);
    }
    Ok(())
}
pub(super) fn add(
    data: &mut Shopping,
    id: &str,
    packages: u32,
    as_of: &str,
) -> Result<(), &'static str> {
    if !(1..=1_000_000).contains(&packages) || !crate::planning::date(as_of) {
        return Err("inventory_offer_invalid");
    }
    let offer = data
        .inventory
        .offers
        .iter()
        .find(|o| o.id == id && !o.archived)
        .ok_or("inventory_offer_not_found")?
        .clone();
    if as_of < offer.valid_from.as_str() || as_of > offer.valid_until.as_str() {
        return Err("inventory_offer_inactive");
    }
    if !data.shops.entries.iter().any(|s| s.id == offer.shop_id) {
        return Err("shopping_shop_not_found");
    }
    let (unit, per_package) = offer.quantity.normalized()?;
    let n = amount(per_package.atoms() * u128::from(packages))?;
    let source_id = data.allocate("source")?;
    let key = offer.binding.ingredient_id.as_ref().map_or_else(
        || format!("manual:{source_id}:{}", unit.key()),
        |id| format!("ingredient:{id}:{}", unit.key()),
    );
    data.sources.push(Source {
        id: source_id,
        active: true,
        origin: Origin::Offer {
            offer: Box::new(offer.clone()),
            packages,
        },
        requirements: vec![Requirement {
            item: Item {
                key: key.clone(),
                ingredient_id: offer.binding.ingredient_id,
                name: offer.binding.name,
                translations: offer.binding.translations,
                unit,
            },
            amount: n,
        }],
    });
    data.shops.assignments.insert(key, Some(offer.shop_id));
    Ok(())
}

pub(super) fn add_checked(
    data: &mut Shopping,
    ctx: &super::operations::Context<'_>,
    id: &str,
    packages: u32,
    as_of: &str,
    members: &[String],
) -> Result<(), &'static str> {
    let offer = data
        .inventory
        .offers
        .iter()
        .find(|o| o.id == id)
        .ok_or("inventory_offer_not_found")?;
    if offer
        .binding
        .ingredient_id
        .as_ref()
        .is_some_and(|id| ctx.ingredients.catalogue().find(id).is_none())
    {
        return Err("inventory_offer_unavailable");
    }
    if safety(
        ctx.state,
        ctx.profiles,
        ctx.ingredients,
        &offer.binding,
        members,
    )?["status"]
        == "blocked"
    {
        return Err("inventory_offer_blocked");
    }
    add(data, id, packages, as_of)
}
