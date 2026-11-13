use super::operations::Command;
use crate::cooking::{
    self,
    delivery::Delivery,
    model::{Data, RunState},
    operations::{self, TimerAction},
};
pub(super) fn execute(
    data: &mut Data,
    command: &Command,
    actor: &cooking::Actor,
    now: i64,
) -> Result<String, &'static str> {
    let run_id = match command {
        Command::Replan { run_id, .. }
        | Command::Timer { run_id, .. }
        | Command::Cancel { run_id }
        | Command::ArchiveBatch { run_id, .. }
        | Command::Subscribe { run_id, .. } => run_id.clone(),
        Command::Acknowledge { notice_id } | Command::Retry { notice_id, .. } => data
            .notices
            .iter()
            .find(|n| n.id == *notice_id && n.user == actor.user)
            .ok_or("cooking_notice_not_found")?
            .run_id
            .clone(),
        _ => return Err("brewing_request_invalid"),
    };
    if !data.brewing.batches.contains_key(&run_id) {
        return Err("brewing_batch_not_found");
    }
    match command {
        Command::Replan { change, .. } => {
            let (run, _) = operations::replan(data.run(&run_id)?.clone(), change.clone(), now)?;
            super::resources::check(data, &run, &data.brewing.batches[&run_id], now)?;
            *data.run_mut(&run_id)? = run;
        }
        Command::Timer {
            step_id, action, ..
        } => {
            if matches!(action, TimerAction::Start | TimerAction::Resume) {
                super::resources::start(data, &run_id, step_id)?;
            }
            operations::timer(data, &run_id, step_id, *action, now)?;
        }
        Command::Cancel { .. } => operations::cancel(data, &run_id)?,
        Command::ArchiveBatch { archived, .. } => {
            if data.run(&run_id)?.status == RunState::Active {
                return Err("cooking_run_active");
            }
            data.brewing
                .batches
                .get_mut(&run_id)
                .ok_or("brewing_batch_not_found")?
                .archived = *archived;
        }
        Command::Subscribe {
            enabled,
            language,
            permission,
            ..
        } => operations::subscribe(
            data,
            &run_id,
            actor,
            *enabled,
            language.clone(),
            *permission,
            now,
        )?,
        Command::Acknowledge { notice_id } | Command::Retry { notice_id, .. } => {
            let index = data
                .notices
                .iter()
                .position(|n| n.id == *notice_id && n.user == actor.user)
                .ok_or("cooking_notice_not_found")?;
            if let Command::Retry {
                duplicate_risk_accepted,
                ..
            } = command
            {
                cooking::delivery::retry(data, index, *duplicate_risk_accepted, now)?;
            } else {
                data.notices[index].acknowledged_at = Some(now);
                if matches!(data.notices[index].delivery, Delivery::Queued { .. }) {
                    data.notices[index].delivery = Delivery::Cancelled;
                }
            }
        }
        _ => return Err("brewing_request_invalid"),
    }
    cooking::delivery::cancel_obsolete(data);
    Ok(run_id)
}

pub(super) fn validate(
    data: &Data,
    command: &Command,
    result: &str,
    actor: &str,
) -> Result<(), &'static str> {
    let run_id = match command {
        Command::Replan { run_id, .. }
        | Command::Timer { run_id, .. }
        | Command::Cancel { run_id }
        | Command::ArchiveBatch { run_id, .. }
        | Command::Subscribe { run_id, .. } => run_id,
        Command::Acknowledge { notice_id } | Command::Retry { notice_id, .. } => {
            &data
                .notices
                .iter()
                .find(|n| n.id == *notice_id && n.user == actor)
                .ok_or("brewing_data_invalid")?
                .run_id
        }
        _ => return Err("brewing_data_invalid"),
    };
    if run_id != result || !data.brewing.batches.contains_key(run_id) {
        return Err("brewing_data_invalid");
    }
    Ok(())
}
