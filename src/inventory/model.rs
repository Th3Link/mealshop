use crate::quantities::Amount;
use crate::shopping::{
    model::{Origin, Purchase, Quantity, Requirement, Unit},
    products::Product,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Target {
    Ingredient { ingredient_id: String },
    Product { barcode: String },
    Unmatched { label: String },
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Binding {
    pub(crate) ingredient_id: Option<String>,
    pub(crate) name: String,
    pub(crate) translations: BTreeMap<String, String>,
    pub(crate) product: Option<Product>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Metadata {
    pub(crate) location: String,
    pub(crate) batch: String,
    pub(crate) expiry: Option<String>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Lot {
    pub(crate) id: String,
    pub(crate) binding: Binding,
    pub(crate) original_quantity: Quantity,
    pub(crate) unit: Unit,
    pub(crate) metadata: Metadata,
    pub(crate) archived: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AllocationState {
    Held,
    Consumed,
    Released,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Allocation {
    pub(crate) id: String,
    pub(crate) lot_id: String,
    pub(crate) source_id: String,
    pub(crate) key: String,
    pub(crate) origin: Origin,
    pub(crate) use_on: String,
    pub(crate) amount: Amount,
    pub(crate) state: AllocationState,
    pub(crate) movement_id: Option<String>,
}
#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Direction {
    In,
    Out,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Entry {
    pub(crate) lot_id: String,
    pub(crate) direction: Direction,
    pub(crate) amount: Amount,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum MovementKind {
    Opening,
    Adjustment {
        reason: String,
        observed: Quantity,
    },
    Purchase {
        purchases: Vec<Purchase>,
    },
    Cooking {
        run_id: String,
        recipe_id: String,
        revision: u64,
        servings: u32,
        requirements: Vec<Requirement>,
        unbooked: Vec<Requirement>,
        use_on: String,
    },
    Reversal {
        movement_id: String,
        reason: String,
    },
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Movement {
    pub(crate) id: String,
    pub(crate) kind: MovementKind,
    pub(crate) entries: Vec<Entry>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Offer {
    pub(crate) id: String,
    pub(crate) shop_id: String,
    pub(crate) shop_name: String,
    pub(crate) binding: Binding,
    pub(crate) quantity: Quantity,
    pub(crate) price: Amount,
    pub(crate) currency: String,
    pub(crate) valid_from: String,
    pub(crate) valid_until: String,
    pub(crate) source: String,
    pub(crate) source_url: Option<String>,
    pub(crate) archived: bool,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OfferInput {
    pub(crate) id: Option<String>,
    pub(crate) shop_id: String,
    pub(crate) target: Target,
    pub(crate) quantity: Quantity,
    pub(crate) price: Amount,
    pub(crate) currency: String,
    pub(crate) valid_from: String,
    pub(crate) valid_until: String,
    pub(crate) source: String,
    pub(crate) source_url: Option<String>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Command {
    RecordPrepared {
        input: super::prepared::Input,
    },
    StorePrepared {
        batch_id: String,
        storage: super::prepared::Storage,
    },
    AllocatePrepared {
        batch_id: String,
        date: String,
        meal: String,
        portions: u32,
        member_ids: Vec<String>,
    },
    ReleasePrepared {
        batch_id: String,
        allocation_id: String,
    },
    UsePrepared {
        batch_id: String,
        allocation_id: String,
    },
    UndoPreparedUse {
        batch_id: String,
        allocation_id: String,
        reason: String,
    },
    ReversePrepared {
        batch_id: String,
        reason: String,
    },
    CreateLot {
        target: Target,
        quantity: Quantity,
        metadata: Metadata,
    },
    UpdateLot {
        lot_id: String,
        metadata: Metadata,
    },
    AdjustLot {
        lot_id: String,
        quantity: Quantity,
        reason: String,
    },
    ArchiveLot {
        lot_id: String,
    },
    Reserve {
        source_ids: Vec<String>,
        as_of: String,
    },
    Release {
        allocation_ids: Vec<String>,
    },
    Receive {
        purchase_ids: Vec<String>,
        metadata: Metadata,
    },
    Consume {
        run_id: String,
        source_ids: Vec<String>,
        include_unreserved: bool,
        as_of: String,
    },
    Reverse {
        movement_id: String,
        reason: String,
    },
    SaveOffer {
        offer: OfferInput,
    },
    ArchiveOffer {
        offer_id: String,
    },
    AddOffer {
        offer_id: String,
        packages: u32,
        as_of: String,
        member_ids: Vec<String>,
    },
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Operation {
    pub(crate) id: String,
    pub(crate) actor: String,
    pub(crate) key: String,
    pub(crate) expected_revision: u64,
    pub(crate) command: Command,
    pub(crate) recorded_at: u64,
}
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Data {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) prepared: Vec<super::prepared::Batch>,
    pub(crate) lots: Vec<Lot>,
    pub(crate) allocations: Vec<Allocation>,
    pub(crate) movements: Vec<Movement>,
    pub(crate) offers: Vec<Offer>,
    pub(crate) operations: Vec<Operation>,
}

impl Data {
    pub(crate) fn is_empty(&self) -> bool {
        self.lots.is_empty()
            && self.prepared.is_empty()
            && self.allocations.is_empty()
            && self.movements.is_empty()
            && self.offers.is_empty()
            && self.operations.is_empty()
    }
}
