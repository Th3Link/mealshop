use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Event {
    Prepare,
    Start,
    ActiveEnded,
    TimerEnded,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Delivery {
    Queued { retry_at: i64 },
    Sending { attempted_at: i64 },
    Accepted { accepted_at: i64 },
    Failed { code: String, failed_at: i64 },
    Unknown { attempted_at: i64 },
    Missed { detected_at: i64 },
    Cancelled,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Notice {
    pub(crate) id: String,
    pub(crate) run_id: String,
    pub(crate) step_id: String,
    pub(crate) generation: u64,
    pub(crate) user: String,
    pub(crate) event: Event,
    pub(crate) due: i64,
    pub(crate) attempts: u8,
    pub(crate) delivery: Delivery,
    pub(crate) acknowledged_at: Option<i64>,
}

pub(crate) fn cancel_obsolete(data: &mut super::model::Data) {
    let obsolete = data
        .notices
        .iter()
        .enumerate()
        .filter(|(_, n)| matches!(n.delivery, Delivery::Queued { .. }) && !current(data, n))
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    for i in obsolete {
        data.notices[i].delivery = Delivery::Cancelled;
    }
}
pub(crate) fn validate(data: &super::model::Data) -> Result<(), &'static str> {
    let mut ids = std::collections::BTreeSet::new();
    let mut events = std::collections::BTreeSet::new();
    for notice in &data.notices {
        if !ids.insert(&notice.id)
            || !super::store::allocated(data, &notice.id, "notice-")
            || !super::actor_valid(&notice.user)
            || !super::time::valid(notice.due)
            || notice.attempts > 3
            || !valid_delivery(&notice.delivery, notice.attempts)
            || notice.generation == 0
            || notice
                .acknowledged_at
                .is_some_and(|at| !super::time::valid(at))
        {
            return Err("cooking_data_invalid");
        }
        let run = data.run(&notice.run_id)?;
        if !run
            .steps
            .iter()
            .any(|s| s.stage.id == notice.step_id && s.generation >= notice.generation)
            || !events.insert((
                &notice.run_id,
                &notice.step_id,
                &notice.user,
                notice.generation,
                format!("{:?}", notice.event),
            ))
        {
            return Err("cooking_data_invalid");
        }
    }
    Ok(())
}

pub(crate) fn retry(
    data: &mut super::model::Data,
    index: usize,
    accepted_risk: bool,
    now: i64,
) -> Result<(), &'static str> {
    let notice = &data.notices[index];
    if !current(data, notice) || notice.attempts >= 3 || notice.acknowledged_at.is_some() {
        return Err("cooking_notice_retry_invalid");
    }
    match notice.delivery {
        Delivery::Failed { .. } => {}
        Delivery::Unknown { .. } if accepted_risk => {}
        Delivery::Unknown { .. } => return Err("cooking_duplicate_risk_required"),
        _ => return Err("cooking_notice_retry_invalid"),
    }
    data.notices[index].delivery = Delivery::Queued { retry_at: now };
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Destination {
    #[default]
    Cooking,
    Brewing,
}
impl Destination {
    pub(crate) fn path(self) -> &'static str {
        match self {
            Self::Cooking => "cooking",
            Self::Brewing => "brewing/batches",
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Job {
    #[serde(default)]
    pub(crate) destination: Destination,
    pub(crate) id: String,
    pub(crate) run_id: String,
    pub(crate) step_id: String,
    pub(crate) user: String,
    pub(crate) event: Event,
    pub(crate) action: String,
    pub(crate) equipment: Vec<String>,
    pub(crate) language: String,
    pub(crate) attempted_at: i64,
    pub(crate) due: i64,
}
fn events(run: &super::model::Run, step: &super::model::Step) -> Vec<(Event, i64)> {
    use super::model::Timer;
    match step.timer {
        Timer::Pending => run
            .schedule
            .iter()
            .find(|t| t.id == step.stage.id)
            .map_or_else(Vec::new, |t| {
                let mut events = vec![(Event::Start, t.start)];
                if step.stage.lead_seconds > 0 {
                    events.insert(
                        0,
                        (Event::Prepare, t.start - i64::from(step.stage.lead_seconds)),
                    );
                }
                events
            }),
        Timer::Running {
            started_at,
            active_until,
            deadline,
        } => {
            if run.kind.is_brewing() && step.stage.timer_seconds() == 0 && deadline == started_at {
                return vec![];
            }
            let mut events = vec![];
            if active_until > started_at && active_until < deadline {
                events.push((Event::ActiveEnded, active_until));
            }
            events.push((Event::TimerEnded, deadline));
            events
        }
        _ => vec![],
    }
}
fn current(data: &super::model::Data, notice: &Notice) -> bool {
    data.runs
        .iter()
        .find(|r| r.id == notice.run_id)
        .is_some_and(|run| {
            run.status == super::model::RunState::Active
                && run
                    .subscriptions
                    .get(&notice.user)
                    .is_some_and(|s| s.enabled)
                && run.steps.iter().any(|step| {
                    step.stage.id == notice.step_id
                        && step.generation == notice.generation
                        && events(run, step)
                            .iter()
                            .any(|&(event, due)| event == notice.event && due == notice.due)
                })
        })
}
fn enqueue_due(data: &mut super::model::Data, now: i64) -> Result<(), &'static str> {
    cancel_obsolete(data);
    let existing = data
        .notices
        .iter()
        .map(|n| {
            (
                n.run_id.as_str(),
                n.step_id.as_str(),
                n.generation,
                n.user.as_str(),
                n.event,
            )
        })
        .collect::<std::collections::BTreeSet<_>>();
    let mut missing = vec![];
    for run in data
        .runs
        .iter()
        .filter(|r| r.status == super::model::RunState::Active)
    {
        for user in run
            .subscriptions
            .iter()
            .filter(|(_, s)| s.enabled)
            .map(|(user, _)| user)
        {
            for step in &run.steps {
                for (event, due) in events(run, step).into_iter().filter(|(_, due)| *due <= now) {
                    if !existing.contains(&(
                        run.id.as_str(),
                        step.stage.id.as_str(),
                        step.generation,
                        user.as_str(),
                        event,
                    )) {
                        if data.notices.len() + missing.len() >= 2000 {
                            return Err("cooking_capacity");
                        }
                        missing.push((
                            run.id.clone(),
                            step.stage.id.clone(),
                            step.generation,
                            user.clone(),
                            event,
                            due,
                        ));
                    }
                }
            }
        }
    }
    for (run_id, step_id, generation, user, event, due) in missing {
        if data.notices.len() >= 2000 {
            return Err("cooking_capacity");
        }
        let id = data.allocate("notice")?;
        data.notices.push(Notice {
            id,
            run_id,
            step_id,
            generation,
            user,
            event,
            due,
            attempts: 0,
            delivery: if now - due > 86_400 {
                Delivery::Missed { detected_at: now }
            } else {
                Delivery::Queued { retry_at: now }
            },
            acknowledged_at: None,
        });
    }
    Ok(())
}
pub(crate) fn claim(data: &mut super::model::Data, now: i64) -> Result<Vec<Job>, &'static str> {
    enqueue_due(data, now)?;
    let indexes = data
        .notices
        .iter()
        .enumerate()
        .filter(|(_, n)| {
            matches!(n.delivery,Delivery::Queued {retry_at} if retry_at<=now)
                && current(data, n)
                && n.acknowledged_at.is_none()
        })
        .map(|(i, _)| i)
        .take(1)
        .collect::<Vec<_>>();
    let mut jobs = vec![];
    for i in indexes {
        let notice = &data.notices[i];
        let run = data.run(&notice.run_id)?;
        let step = run
            .steps
            .iter()
            .find(|s| s.stage.id == notice.step_id)
            .ok_or("cooking_data_invalid")?;
        let subscription = run
            .subscriptions
            .get(&notice.user)
            .ok_or("cooking_data_invalid")?;
        jobs.push(Job {
            destination: if run.kind.is_brewing() {
                Destination::Brewing
            } else {
                Destination::Cooking
            },
            id: notice.id.clone(),
            run_id: run.id.clone(),
            step_id: step.stage.id.clone(),
            user: notice.user.clone(),
            event: notice.event,
            action: step.stage.name.clone(),
            equipment: step
                .stage
                .equipment
                .iter()
                .map(|id| {
                    data.brewing
                        .batches
                        .get(&run.id)
                        .and_then(|b| b.equipment.iter().find(|e| &e.id == id))
                        .map_or_else(|| id.clone(), |e| e.details.name.clone())
                })
                .collect(),
            language: subscription.language.clone(),
            attempted_at: now,
            due: notice.due,
        });
        let notice = &mut data.notices[i];
        notice.attempts = notice.attempts.checked_add(1).ok_or("cooking_capacity")?;
        notice.delivery = Delivery::Sending { attempted_at: now };
    }
    Ok(jobs)
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Outcome {
    Accepted,
    ConnectionFailed,
    Rejected,
    Unavailable,
    Unknown,
    Revoked,
}
pub(crate) fn settle(
    data: &mut super::model::Data,
    id: &str,
    attempted_at: i64,
    outcome: Outcome,
    now: i64,
) -> Result<(), &'static str> {
    let index = data
        .notices
        .iter()
        .position(|n| n.id == id)
        .ok_or("cooking_notice_not_found")?;
    if !matches!(data.notices[index].delivery,Delivery::Sending {attempted_at:at} if at==attempted_at)
    {
        return Err("cooking_notice_retry_invalid");
    }
    let still_current = current(data, &data.notices[index]);
    let notice = &mut data.notices[index];
    notice.delivery = match outcome {
        Outcome::Accepted => Delivery::Accepted { accepted_at: now },
        Outcome::ConnectionFailed if still_current && notice.attempts < 3 => Delivery::Queued {
            retry_at: super::time::add(now, 30 * i64::from(notice.attempts))?,
        },
        Outcome::ConnectionFailed => Delivery::Failed {
            code: "cooking_delivery_connection_failed".to_owned(),
            failed_at: now,
        },
        Outcome::Rejected => Delivery::Failed {
            code: "cooking_delivery_rejected".to_owned(),
            failed_at: now,
        },
        Outcome::Unavailable => Delivery::Failed {
            code: "cooking_notifications_unavailable".to_owned(),
            failed_at: now,
        },
        Outcome::Unknown => Delivery::Unknown { attempted_at },
        Outcome::Revoked => Delivery::Cancelled,
    };
    Ok(())
}

fn valid_delivery(delivery: &Delivery, attempts: u8) -> bool {
    use super::time::valid;
    match delivery {
        Delivery::Queued { retry_at } => valid(*retry_at) && attempts < 3,
        Delivery::Sending { attempted_at } | Delivery::Unknown { attempted_at } => {
            valid(*attempted_at) && attempts > 0
        }
        Delivery::Accepted { accepted_at } => valid(*accepted_at) && attempts > 0,
        Delivery::Failed { code, failed_at } => {
            valid(*failed_at)
                && attempts > 0
                && matches!(
                    code.as_str(),
                    "cooking_delivery_connection_failed"
                        | "cooking_delivery_rejected"
                        | "cooking_notifications_unavailable"
                )
        }
        Delivery::Missed { detected_at } => valid(*detected_at) && attempts == 0,
        Delivery::Cancelled => true,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::cooking::{
        model::{Data, Timer},
        store::Store,
    };
    fn fixture() -> Data {
        serde_json::from_value(serde_json::json!({"schema_version":1,"revision":1,"delivery_revision":0,"next_id":2,"notices":[],"runs":[{"id":"cook-1","name":"Test session","kind":"bread","recipe":{"id":"recipe-00000001","revision":1,"title":"Test recipe","language":"en","reference_servings":1},"servings":1,"timezone":"UTC","target":null,"created_at":1_800_000_000,"status":"active","subscriptions":{"alice":{"enabled":true,"language":"en","permission":"denied","reported_at":1_800_000_000}},"schedule":[{"id":"mix-1","start":1_800_000_300,"active_until":1_800_000_360,"finish":1_800_000_420,"earliest":1_800_000_300,"latest":1_800_000_300}],"steps":[{"stage":{"id":"mix-1","name":"Prepare dough","instruction":"Mix test ingredients","origin":{"kind":"custom","note":"Test fixture"},"dependencies":[],"active_seconds":60,"passive_seconds":60,"passive_min_seconds":60,"passive_max_seconds":60,"equipment":["Bowl"],"lead_seconds":300,"repeat":1,"wait_between_seconds":0,"uncertainty":"Unverified room temperature"},"cycle":1,"lag_seconds":0,"not_before":null,"timer":{"state":"pending"},"generation":1}]}]})).unwrap()
    }
    #[test]
    fn reminders_are_personal_idempotent_bounded_and_do_not_claim_device_delivery() {
        let mut data = fixture();
        let now = 1_800_000_000;
        let jobs = claim(&mut data, now).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].user, "alice");
        assert_eq!(jobs[0].equipment, vec!["Bowl"]);
        assert_eq!(jobs[0].event, Event::Prepare);
        assert!(claim(&mut data, now).unwrap().is_empty());
        settle(&mut data, &jobs[0].id, now, Outcome::ConnectionFailed, now).unwrap();
        assert!(claim(&mut data, now + 29).unwrap().is_empty());
        let second = claim(&mut data, now + 30).unwrap();
        assert_eq!(second.len(), 1);
        settle(
            &mut data,
            &second[0].id,
            now + 30,
            Outcome::Unknown,
            now + 30,
        )
        .unwrap();
        assert!(claim(&mut data, now + 90).unwrap().is_empty());
        assert_eq!(
            retry(&mut data, 0, false, now + 90),
            Err("cooking_duplicate_risk_required")
        );
        retry(&mut data, 0, true, now + 90).unwrap();
        let last = claim(&mut data, now + 90).unwrap();
        settle(
            &mut data,
            &last[0].id,
            now + 90,
            Outcome::Accepted,
            now + 90,
        )
        .unwrap();
        assert!(matches!(
            data.notices[0].delivery,
            Delivery::Accepted { .. }
        ));
        assert_eq!(
            retry(&mut data, 0, true, now + 90),
            Err("cooking_notice_retry_invalid")
        );
        validate(&data).unwrap();
        data.runs[0].steps[0].timer = Timer::Running {
            started_at: now,
            active_until: now + 60,
            deadline: now + 120,
        };
        data.runs[0].steps[0].generation += 1;
        let phase = claim(&mut data, now + 60).unwrap();
        assert_eq!(phase[0].event, Event::ActiveEnded);
        settle(
            &mut data,
            &phase[0].id,
            now + 60,
            Outcome::Accepted,
            now + 60,
        )
        .unwrap();
        let end = claim(&mut data, now + 120).unwrap();
        assert_eq!(end[0].event, Event::TimerEnded);
        settle(
            &mut data,
            &end[0].id,
            now + 120,
            Outcome::Accepted,
            now + 120,
        )
        .unwrap();
        assert!(claim(&mut data, now + 180).unwrap().is_empty());
    }
    #[test]
    fn interrupted_attempt_is_unknown_after_restart_and_cannot_be_silently_resent() {
        let root =
            std::env::temp_dir().join(format!("mealshop-cooking-outbox-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let mut store = Store::open(&root).unwrap();
        store
            .change(Some(0), |data| {
                *data = fixture();
                Ok(())
            })
            .unwrap();
        let mut id = String::new();
        store
            .change(None, |data| {
                id = claim(data, 1_800_000_000)?[0].id.clone();
                Ok(())
            })
            .unwrap();
        let revision = store.data.revision;
        drop(store);
        let mut store = Store::open(&root).unwrap();
        assert_eq!(store.data.revision, revision);
        assert!(matches!(
            store.data.notices[0].delivery,
            Delivery::Unknown { .. }
        ));
        assert!(claim(&mut store.data, 1_800_000_001).unwrap().is_empty());
        store
            .change(None, |data| {
                data.runs[0].steps[0].timer = Timer::Paused {
                    started_at: 1_800_000_000,
                    paused_at: 1_800_000_001,
                    active_remaining: 59,
                    remaining: 119,
                };
                Ok(())
            })
            .unwrap();
        assert_eq!(
            retry(&mut store.data, 0, true, 1_800_000_002),
            Err("cooking_notice_retry_invalid")
        );
        assert_eq!(store.data.notices[0].id, id);
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn missed_events_are_reported_and_revoked_subscriptions_cancel_queued_retries() {
        let mut data = fixture();
        let now = 1_800_000_000;
        let job = claim(&mut data, now).unwrap().remove(0);
        settle(&mut data, &job.id, now, Outcome::ConnectionFailed, now).unwrap();
        data.runs[0].subscriptions.get_mut("alice").unwrap().enabled = false;
        assert!(claim(&mut data, now + 30).unwrap().is_empty());
        assert!(matches!(data.notices[0].delivery, Delivery::Cancelled));
        let mut data = fixture();
        assert!(claim(&mut data, now + 172_800).unwrap().is_empty());
        assert_eq!(data.notices.len(), 2);
        assert!(
            data.notices
                .iter()
                .all(|n| matches!(n.delivery, Delivery::Missed { .. }))
        );
        validate(&data).unwrap();
        data.notices[0].delivery = Delivery::Accepted { accepted_at: now };
        assert_eq!(validate(&data), Err("cooking_data_invalid"));
    }
}
