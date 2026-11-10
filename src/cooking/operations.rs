use super::{
    model::{Data, Permission, Run, RunState, Subscription, Timer},
    plan,
    time::{self, LocalTime},
};
use serde::Deserialize;
#[derive(Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum PlanChange {
    Target {
        target: LocalTime,
    },
    Delay {
        step_id: String,
        start_after: LocalTime,
    },
    Timer {
        step_id: String,
        deadline: LocalTime,
    },
}
pub(crate) fn replan(
    mut run: Run,
    change: PlanChange,
    now: i64,
) -> Result<(Run, plan::Proposal), &'static str> {
    if run.status != RunState::Active {
        return Err("cooking_run_inactive");
    }

    match change {
        PlanChange::Target { target } => {
            if target.timezone != run.timezone {
                return Err("cooking_timezone_invalid");
            }
            target.resolve()?;
            run.target = Some(target);
        }
        PlanChange::Delay {
            step_id,
            start_after,
        } => {
            if start_after.timezone != run.timezone {
                return Err("cooking_timezone_invalid");
            }
            let step = run
                .steps
                .iter_mut()
                .find(|s| s.stage.id == step_id)
                .ok_or("cooking_step_not_found")?;
            if !matches!(step.timer, Timer::Pending) {
                return Err("cooking_timer_transition_invalid");
            }
            step.not_before = Some(start_after.resolve()?);
        }
        PlanChange::Timer { step_id, deadline } => {
            if deadline.timezone != run.timezone {
                return Err("cooking_timezone_invalid");
            }
            let at = deadline.resolve()?;
            if at < now || at - now > 1_209_600 {
                return Err("cooking_time_invalid");
            }
            let step = run
                .steps
                .iter_mut()
                .find(|s| s.stage.id == step_id)
                .ok_or("cooking_step_not_found")?;
            step.timer = match step.timer {
                Timer::Running {
                    started_at,
                    active_until,
                    ..
                } => Timer::Running {
                    started_at,
                    active_until: active_until.min(at),
                    deadline: at,
                },
                Timer::Paused {
                    started_at,
                    paused_at,
                    active_remaining,
                    ..
                } => {
                    let remaining = u32::try_from(at - now).map_err(|_| "cooking_time_invalid")?;
                    Timer::Paused {
                        started_at,
                        paused_at,
                        active_remaining: active_remaining.min(remaining),
                        remaining,
                    }
                }
                _ => return Err("cooking_timer_transition_invalid"),
            };
        }
    }
    let mut proposal = plan::schedule(&run, now, None)?;
    proposal.warnings.push(plan::Warning {
        code: "cooking_duration_estimate",
        steps: vec![],
    });
    for step in &mut run.steps {
        let before = run.schedule.iter().find(|t| t.id == step.stage.id);
        let after = proposal.schedule.iter().find(|t| t.id == step.stage.id);
        if before != after || matches!(step.timer, Timer::Running { .. } | Timer::Paused { .. }) {
            step.generation = step.generation.checked_add(1).ok_or("cooking_capacity")?;
        }
    }
    run.schedule.clone_from(&proposal.schedule);
    Ok((run, proposal))
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TimerAction {
    Start,
    Pause,
    Resume,
    Complete,
}
pub(crate) fn timer(
    data: &mut Data,
    run_id: &str,
    step_id: &str,
    action: TimerAction,
    now: i64,
) -> Result<(), &'static str> {
    let run = data.run_mut(run_id)?;
    let index = super::source::ensure_ready(run, step_id, now)?;
    let step = &mut run.steps[index];
    let started = super::source::started(&step.timer);
    step.timer = match (action, &step.timer) {
        (TimerAction::Start, Timer::Pending) => {
            if step.stage.seconds() == 0 {
                Timer::Completed {
                    started_at: Some(now),
                    completed_at: now,
                }
            } else {
                Timer::Running {
                    started_at: now,
                    active_until: time::add(now, i64::from(step.stage.active_seconds))?,
                    deadline: time::add(now, step.stage.seconds())?,
                }
            }
        }
        (
            TimerAction::Pause,
            Timer::Running {
                started_at,
                active_until,
                deadline,
            },
        ) => {
            if now < *started_at {
                return Err("cooking_clock_invalid");
            }
            Timer::Paused {
                started_at: *started_at,
                paused_at: now,
                active_remaining: u32::try_from((active_until - now).max(0))
                    .map_err(|_| "cooking_time_invalid")?,
                remaining: u32::try_from((deadline - now).max(0))
                    .map_err(|_| "cooking_time_invalid")?,
            }
        }
        (
            TimerAction::Resume,
            Timer::Paused {
                started_at,
                active_remaining,
                remaining,
                ..
            },
        ) => {
            if now < *started_at {
                return Err("cooking_clock_invalid");
            }
            Timer::Running {
                started_at: *started_at,
                active_until: if *active_remaining == 0 {
                    *started_at
                } else {
                    time::add(now, i64::from(*active_remaining))?
                },
                deadline: time::add(now, i64::from(*remaining))?,
            }
        }
        (TimerAction::Complete, Timer::Pending | Timer::Running { .. } | Timer::Paused { .. }) => {
            if started.is_some_and(|at| at > now) {
                return Err("cooking_clock_invalid");
            }
            Timer::Completed {
                started_at: started,
                completed_at: now,
            }
        }
        _ => return Err("cooking_timer_transition_invalid"),
    };
    step.generation = step.generation.checked_add(1).ok_or("cooking_capacity")?;
    if run.steps.iter().all(|s| s.timer.complete()) {
        run.status = RunState::Completed;
    }
    super::delivery::cancel_obsolete(data);
    Ok(())
}
pub(crate) fn cancel(data: &mut Data, id: &str) -> Result<(), &'static str> {
    let run = data.run_mut(id)?;
    if run.status != RunState::Active {
        return Err("cooking_run_inactive");
    }
    run.status = RunState::Cancelled;
    for step in &mut run.steps {
        if !step.timer.complete() {
            step.timer = Timer::Cancelled;
            step.generation = step.generation.checked_add(1).ok_or("cooking_capacity")?;
        }
    }
    super::delivery::cancel_obsolete(data);
    Ok(())
}
pub(crate) fn subscribe(
    data: &mut Data,
    id: &str,
    actor: &super::Actor,
    enabled: bool,
    language: String,
    permission: Permission,
    now: i64,
) -> Result<(), &'static str> {
    if enabled && !actor.notifications_available {
        return Err("cooking_notifications_unavailable");
    }
    if !matches!(language.as_str(), "en" | "de") {
        return Err("cooking_request_invalid");
    }
    let run = data.run_mut(id)?;
    if run.status != RunState::Active {
        return Err("cooking_run_inactive");
    }
    run.subscriptions.insert(
        actor.user.clone(),
        Subscription {
            enabled,
            language,
            permission,
            reported_at: now,
        },
    );
    super::delivery::cancel_obsolete(data);
    Ok(())
}
