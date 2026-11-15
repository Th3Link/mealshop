use super::{commands::Command, model::Data, time};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Receipt {
    pub(crate) actor: String,
    pub(crate) key: String,
    pub(crate) expected_revision: u64,
    pub(crate) command: serde_json::Value,
    pub(crate) occurred_at: Option<i64>,
    pub(crate) recorded_at: i64,
    pub(crate) run_id: Option<String>,
}

pub(super) fn replay<'a>(
    data: &'a Data,
    actor: &str,
    key: &str,
    expected: u64,
    command: &Command,
    occurred_at: Option<i64>,
) -> Result<Option<&'a Receipt>, &'static str> {
    let Some(r) = data
        .operations
        .iter()
        .find(|r| r.actor == actor && r.key == key)
    else {
        return Ok(None);
    };
    if r.expected_revision != expected
        || r.occurred_at != occurred_at
        || r.command != serde_json::to_value(command).map_err(|_| "cooking_storage_failed")?
    {
        return Err("cooking_request_conflict");
    }
    Ok(Some(r))
}

pub(super) fn record(data: &mut Data, receipt: Receipt) -> Result<(), &'static str> {
    if data.operations.len() >= 2000 {
        return Err("cooking_operation_capacity");
    }
    data.operations.push(receipt);
    Ok(())
}

pub(super) fn validate(data: &Data) -> Result<(), &'static str> {
    if data.operations.len() > 2000 {
        return Err("cooking_capacity");
    }
    let mut keys = BTreeSet::new();
    let mut revisions = BTreeSet::new();
    for r in &data.operations {
        let command: Command =
            serde_json::from_value(r.command.clone()).map_err(|_| "cooking_data_invalid")?;
        if !super::actor_valid(&r.actor)
            || !crate::recipes::Creation::valid_key(&r.key)
            || !keys.insert((&r.actor, &r.key))
            || !revisions.insert(r.expected_revision)
            || r.expected_revision >= data.revision
            || !time::valid(r.recorded_at)
            || r.run_id
                .as_ref()
                .is_some_and(|id| !super::store::allocated(data, id, "cook-"))
            || r.occurred_at.is_some_and(|at| {
                !matches!(command, Command::Timer { .. })
                    || !time::valid(at)
                    || at > r.recorded_at
                    || r.recorded_at - at > 604_800
            })
        {
            return Err("cooking_data_invalid");
        }
    }
    Ok(())
}
