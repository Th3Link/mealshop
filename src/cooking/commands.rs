use super::{
    Actor,
    model::{Data, Permission, RunState},
    operations::{self, TimerAction},
    store::Prepared,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Command {
    Apply {
        preview_id: String,
        reviewed: bool,
    },
    Timer {
        run_id: String,
        step_id: String,
        action: TimerAction,
    },
    Cancel {
        run_id: String,
    },
    Remove {
        run_id: String,
    },
    Subscribe {
        run_id: String,
        enabled: bool,
        language: String,
        permission: Permission,
    },
    Acknowledge {
        notice_id: String,
    },
    Retry {
        notice_id: String,
        duplicate_risk_accepted: bool,
    },
}

pub(super) fn execute(
    data: &mut Data,
    command: Command,
    actor: &Actor,
    at: i64,
    prepared: Option<Prepared>,
) -> Result<Option<String>, &'static str> {
    match &command {
        Command::Timer { run_id, .. }
        | Command::Cancel { run_id }
        | Command::Remove { run_id }
        | Command::Subscribe { run_id, .. } => {
            data.run(run_id)?.food()?;
        }
        _ => {}
    }
    match command {
        Command::Apply { .. } => {
            let value = prepared.ok_or("cooking_preview_expired")?;
            let mut run = value.run;
            let id = if value.previous.is_some() {
                let id = run.id.clone();
                *data.run_mut(&id)? = run;
                id
            } else {
                run.id = data.allocate("cook")?;
                let id = run.id.clone();
                data.runs.push(run);
                id
            };
            super::delivery::cancel_obsolete(data);
            Ok(Some(id))
        }
        Command::Timer {
            run_id,
            step_id,
            action,
        } => {
            let run = data.run(&run_id)?;
            let step = run
                .steps
                .iter()
                .find(|s| s.stage.id == step_id)
                .ok_or("cooking_step_not_found")?;
            let previous = match step.timer {
                super::model::Timer::Running { started_at, .. } => started_at,
                super::model::Timer::Paused { paused_at, .. } => paused_at,
                _ => run.created_at,
            };
            if at < previous {
                return Err("cooking_clock_invalid");
            }
            operations::timer(data, &run_id, &step_id, action, at)?;
            Ok(Some(run_id))
        }
        Command::Cancel { run_id } => {
            operations::cancel(data, &run_id)?;
            Ok(Some(run_id))
        }
        Command::Remove { run_id } => {
            if data.run(&run_id)?.status == RunState::Active {
                return Err("cooking_run_active");
            }
            data.runs.retain(|r| r.id != run_id);
            data.notices.retain(|n| n.run_id != run_id);
            Ok(None)
        }
        Command::Subscribe {
            run_id,
            enabled,
            language,
            permission,
        } => {
            operations::subscribe(data, &run_id, actor, enabled, language, permission, at)?;
            Ok(Some(run_id))
        }
        Command::Acknowledge { notice_id } => notice(data, actor, &notice_id, false, false, at),
        Command::Retry {
            notice_id,
            duplicate_risk_accepted,
        } => notice(data, actor, &notice_id, true, duplicate_risk_accepted, at),
    }
}

fn notice(
    data: &mut Data,
    actor: &Actor,
    id: &str,
    retry: bool,
    accepted: bool,
    now: i64,
) -> Result<Option<String>, &'static str> {
    let index = data
        .notices
        .iter()
        .position(|n| n.id == id && n.user == actor.user)
        .ok_or("cooking_notice_not_found")?;
    let run_id = data.notices[index].run_id.clone();
    data.run(&run_id)?.food()?;
    if retry {
        super::delivery::retry(data, index, accepted, now)?;
    } else {
        data.notices[index].acknowledged_at = Some(now);
        if matches!(
            data.notices[index].delivery,
            super::delivery::Delivery::Queued { .. }
        ) {
            data.notices[index].delivery = super::delivery::Delivery::Cancelled;
        }
    }
    Ok(Some(run_id))
}
