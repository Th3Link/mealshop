use super::{
    model::{Run, Stage, Step, Timer, Timing},
    time,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn order(stages: &[Stage]) -> Result<Vec<usize>, &'static str> {
    if stages.is_empty() || stages.len() > 200 {
        return Err("cooking_stage_invalid");
    }
    let ids: BTreeMap<_, _> = stages
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.as_str(), i))
        .collect();
    if ids.len() != stages.len() {
        return Err("cooking_dependencies_invalid");
    }
    let mut result = Vec::new();
    let mut seen = BTreeSet::new();
    while result.len() < stages.len() {
        let before = result.len();
        for (i, stage) in stages.iter().enumerate() {
            if seen.contains(&i) {
                continue;
            }
            if stage.dependencies.iter().collect::<BTreeSet<_>>().len() != stage.dependencies.len()
            {
                return Err("cooking_dependencies_invalid");
            }
            let dependencies = stage
                .dependencies
                .iter()
                .map(|id| {
                    ids.get(id.as_str())
                        .copied()
                        .ok_or("cooking_dependencies_invalid")
                })
                .collect::<Result<Vec<_>, _>>()?;
            if dependencies.iter().all(|i| seen.contains(i)) {
                seen.insert(i);
                result.push(i);
            }
        }
        if before == result.len() {
            return Err("cooking_dependencies_invalid");
        }
    }
    Ok(result)
}
pub(crate) fn expand(stages: &[Stage]) -> Result<Vec<Step>, &'static str> {
    expand_limit(stages, 604_800)
}
pub(crate) fn expand_limit(stages: &[Stage], maximum: u32) -> Result<Vec<Step>, &'static str> {
    order(stages)?;
    let last: BTreeMap<_, _> = stages
        .iter()
        .map(|s| (s.id.as_str(), format!("{}-{}", s.id, s.repeat)))
        .collect();
    let mut steps = Vec::new();
    for stage in stages {
        stage.validate_limit(maximum)?;
        for cycle in 1..=stage.repeat {
            let mut value = stage.clone();
            value.id = format!("{}-{cycle}", stage.id);
            value.dependencies = stage
                .dependencies
                .iter()
                .filter_map(|id| last.get(id.as_str()).cloned())
                .collect();
            if cycle > 1 {
                value
                    .dependencies
                    .push(format!("{}-{}", stage.id, cycle - 1));
            }
            value.validate_limit(maximum)?;
            steps.push(Step {
                stage: value,
                cycle,
                lag_seconds: if cycle > 1 {
                    stage.wait_between_seconds
                } else {
                    0
                },
                not_before: None,
                timer: Timer::Pending,
                generation: 1,
            });
        }
    }
    order(&steps.iter().map(|s| s.stage.clone()).collect::<Vec<_>>())?;
    Ok(steps)
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct Warning {
    pub(crate) code: &'static str,
    pub(crate) steps: Vec<String>,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct Proposal {
    pub(crate) schedule: Vec<Timing>,
    pub(crate) warnings: Vec<Warning>,
    pub(crate) finish: i64,
}
fn backward(
    steps: &[Step],
    sorted: &[usize],
    target: i64,
    bound: i8,
) -> Result<Vec<i64>, &'static str> {
    let positions: BTreeMap<_, _> = steps
        .iter()
        .enumerate()
        .map(|(i, s)| (s.stage.id.as_str(), i))
        .collect();
    let mut starts = vec![target; steps.len()];
    for &index in sorted.iter().rev() {
        let step = &steps[index];
        let finish = steps
            .iter()
            .enumerate()
            .filter(|(_, child)| child.stage.dependencies.contains(&step.stage.id))
            .map(|(i, child)| time::add(starts[i], -i64::from(child.lag_seconds)))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .min()
            .unwrap_or(target);
        let passive = match bound {
            -1 => step.stage.passive_min_seconds,
            1 => step.stage.passive_max_seconds,
            _ => step.stage.passive_seconds,
        };
        starts[index] = time::add(
            finish,
            -i64::from(step.stage.active_seconds) - i64::from(passive),
        )?;
        for dependency in &step.stage.dependencies {
            if !positions.contains_key(dependency.as_str()) {
                return Err("cooking_dependencies_invalid");
            }
        }
    }
    Ok(starts)
}
pub(crate) fn schedule(
    run: &Run,
    now: i64,
    delay: Option<(&str, i64)>,
) -> Result<Proposal, &'static str> {
    let stages: Vec<_> = run.steps.iter().map(|s| s.stage.clone()).collect();
    let sorted = order(&stages)?;
    let target = run
        .target
        .as_ref()
        .map(super::time::LocalTime::resolve)
        .transpose()?;
    let base = if let Some(at) = target {
        backward(&run.steps, &sorted, at, 0)?
    } else {
        vec![now; run.steps.len()]
    };
    let early = if let Some(at) = target {
        backward(&run.steps, &sorted, at, 1)?
    } else {
        base.clone()
    };
    let late = if let Some(at) = target {
        backward(&run.steps, &sorted, at, -1)?
    } else {
        base.clone()
    };
    let positions: BTreeMap<_, _> = run
        .steps
        .iter()
        .enumerate()
        .map(|(i, s)| (s.stage.id.as_str(), i))
        .collect();
    let mut times: Vec<Option<Timing>> = vec![None; run.steps.len()];
    let mut warnings = vec![];
    for &index in &sorted {
        let step = &run.steps[index];
        let mut ready = now;
        let anchor = step
            .stage
            .start_at
            .as_ref()
            .map(super::time::LocalTime::resolve)
            .transpose()?;
        let mut start = base[index]
            .max(now)
            .max(step.not_before.unwrap_or(now))
            .max(anchor.unwrap_or(now));
        for dependency in &step.stage.dependencies {
            let previous = times[*positions
                .get(dependency.as_str())
                .ok_or("cooking_dependencies_invalid")?]
            .as_ref()
            .ok_or("cooking_dependencies_invalid")?;
            ready = ready.max(time::add(previous.finish, i64::from(step.lag_seconds))?);
            start = start.max(ready);
        }
        if let Some((id, at)) = delay {
            if step.stage.id == id {
                start = start.max(at);
            }
        }
        let (start, active_until, finish) = actual_timing(step, start, now, &mut warnings)?;
        times[index] = Some(Timing {
            id: step.stage.id.clone(),
            start,
            active_until,
            finish,
            earliest: if matches!(step.timer, Timer::Pending) {
                early[index]
                    .max(ready)
                    .max(step.not_before.unwrap_or(now))
                    .min(start)
            } else {
                start
            },
            latest: if matches!(step.timer, Timer::Pending) {
                late[index].max(start)
            } else {
                start
            },
        });
    }
    let schedule: Vec<_> = times
        .into_iter()
        .collect::<Option<_>>()
        .ok_or("cooking_dependencies_invalid")?;
    let finish = schedule
        .iter()
        .map(|s| s.finish)
        .max()
        .ok_or("cooking_stage_invalid")?;
    if target.is_some_and(|target| finish > target) {
        warnings.push(Warning {
            code: "cooking_target_missed",
            steps: vec![],
        });
    }
    overlaps(run, &schedule, &mut warnings);
    Ok(Proposal {
        schedule,
        warnings,
        finish,
    })
}
fn actual_timing(
    step: &Step,
    start: i64,
    now: i64,
    warnings: &mut Vec<Warning>,
) -> Result<(i64, i64, i64), &'static str> {
    Ok(match step.timer {
        Timer::Pending => (
            start,
            time::add(start, i64::from(step.stage.active_seconds))?,
            time::add(start, step.stage.seconds())?,
        ),
        Timer::Running {
            started_at,
            active_until,
            deadline,
        } => (started_at, active_until, deadline),
        Timer::Paused {
            active_remaining,
            remaining,
            ..
        } => {
            warnings.push(Warning {
                code: "cooking_paused_projection",
                steps: vec![step.stage.id.clone()],
            });
            (
                now,
                time::add(now, i64::from(active_remaining))?,
                time::add(now, i64::from(remaining))?,
            )
        }
        Timer::Completed {
            started_at,
            completed_at,
        } => (
            started_at.unwrap_or(completed_at),
            completed_at,
            completed_at,
        ),
        Timer::Cancelled => return Err("cooking_run_inactive"),
    })
}
fn overlaps(run: &Run, schedule: &[Timing], warnings: &mut Vec<Warning>) {
    for (i, a) in schedule.iter().enumerate() {
        for (j, b) in schedule.iter().enumerate().skip(i + 1) {
            let steps = vec![a.id.clone(), b.id.clone()];
            if a.start < b.active_until && b.start < a.active_until {
                warnings.push(Warning {
                    code: "cooking_parallel_work",
                    steps: steps.clone(),
                });
            }
            if a.start < b.finish
                && b.start < a.finish
                && run.steps[i]
                    .stage
                    .equipment
                    .iter()
                    .any(|e| run.steps[j].stage.equipment.contains(e))
            {
                warnings.push(Warning {
                    code: "cooking_equipment_overlap",
                    steps,
                });
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::cooking::{
        model::{Kind, RecipeRef, RunState, StageOrigin},
        operations::{self, PlanChange, TimerAction},
        time::{Fold, LocalTime},
    };
    fn stage(id: &str, dependencies: Vec<String>, active: u32, passive: u32) -> Stage {
        Stage {
            start_after_work: false,
            start_at: None,
            id: id.to_owned(),
            origin: StageOrigin::Custom {
                note: "Test preparation".to_owned(),
            },
            name: id.to_owned(),
            instruction: format!("Prepare {id}"),
            dependencies,
            active_seconds: active,
            passive_seconds: passive,
            passive_min_seconds: passive,
            passive_max_seconds: passive,
            equipment: vec![],
            lead_seconds: 0,
            repeat: 1,
            wait_between_seconds: 0,
            uncertainty: String::new(),
        }
    }
    fn run(stages: &[Stage], target: LocalTime, now: i64) -> Run {
        Run {
            plan_slot: None,
            id: "cook-1".to_owned(),
            name: "Dough schedule".to_owned(),
            kind: Kind::Bread,
            source: crate::cooking::run_source::Source::Food {
                recipe: RecipeRef {
                    id: "recipe-1".to_owned(),
                    revision: 1,
                    title: "Dough".to_owned(),
                    language: "en".to_owned(),
                    reference_servings: 1,
                },
                servings: 1,
            },
            timezone: target.timezone.clone(),
            target: Some(target),
            created_at: now,
            status: RunState::Active,
            steps: expand(stages).unwrap(),
            schedule: vec![],
            subscriptions: BTreeMap::new(),
        }
    }
    #[test]
    fn calendar_anchor_and_actual_completion_preserve_passive_deadlines() {
        let anchor = LocalTime {
            local: "2026-11-11T07:00".into(),
            timezone: "Europe/Berlin".into(),
            fold: Fold::Reject,
        };
        let at = anchor.resolve().unwrap();
        let mut mix = stage("mix", vec![], 120, 2700);
        mix.start_after_work = true;
        mix.start_at = Some(anchor.clone());
        let next = stage("main", vec!["mix".into()], 0, 2700);
        let mut value = run(&[mix, next], anchor, at - 3600);
        value.kind = Kind::Cooking;
        value.target = None;
        value.schedule = schedule(&value, at - 3600, None).unwrap().schedule;
        assert_eq!(value.steps[0].not_before, None);
        assert_eq!(value.schedule[0].start, at);
        let mut data = crate::cooking::model::Data::default();
        data.runs.push(value);
        assert!(super::super::source::ensure_ready(&data.runs[0], "mix-1", at - 1).is_ok());
        operations::timer(&mut data, "cook-1", "mix-1", TimerAction::Start, at + 480).unwrap();
        assert!(
            matches!(data.runs[0].steps[0].timer, Timer::Running { deadline, .. } if deadline == at + 480 + 2700)
        );
        assert!(
            operations::timer(
                &mut data,
                "cook-1",
                "main-1",
                TimerAction::Start,
                at + 480 + 2699
            )
            .is_err()
        );
        operations::timer(
            &mut data,
            "cook-1",
            "main-1",
            TimerAction::Start,
            at + 480 + 2700,
        )
        .unwrap();
        assert!(data.runs[0].steps[0].timer.complete());
        assert!(
            matches!(data.runs[0].steps[1].timer, Timer::Running { started_at, deadline, .. } if started_at == at + 480 + 2700 && deadline == started_at + 2700)
        );
    }
    #[test]
    fn dependency_windows_cycles_and_dst_use_absolute_elapsed_time() {
        let now = LocalTime {
            local: "2026-03-26T08:00".to_owned(),
            timezone: "Europe/Berlin".to_owned(),
            fold: Fold::Reject,
        }
        .resolve()
        .unwrap();
        let target = LocalTime {
            local: "2026-03-29T12:00".to_owned(),
            timezone: "Europe/Berlin".to_owned(),
            fold: Fold::Reject,
        };
        let mut ferment = stage("ferment", vec![], 300, 172_800);
        ferment.passive_min_seconds = 169_200;
        ferment.passive_max_seconds = 176_400;
        let mut folds = stage("fold", vec!["ferment".to_owned()], 60, 0);
        folds.repeat = 3;
        folds.wait_between_seconds = 1800;
        let heat = stage("preheat", vec![], 0, 1800);
        let bake = stage(
            "bake",
            vec!["fold".to_owned(), "preheat".to_owned()],
            120,
            2400,
        );
        let run = run(&[ferment, folds, heat, bake], target.clone(), now);
        let proposal = schedule(&run, now, None).unwrap();
        let ferment = proposal
            .schedule
            .iter()
            .find(|t| t.id == "ferment-1")
            .unwrap();
        assert_eq!(ferment.finish - ferment.start, 173_100);
        assert_eq!(ferment.start - ferment.earliest, 3600);
        assert_eq!(ferment.latest - ferment.start, 3600);
        assert!(
            time::display(ferment.start, "Europe/Berlin")
                .unwrap()
                .contains("+01:00")
        );
        assert!(
            time::display(proposal.finish, "Europe/Berlin")
                .unwrap()
                .contains("+02:00")
        );
        assert_eq!(proposal.finish, target.resolve().unwrap());
        let first = proposal.schedule.iter().find(|t| t.id == "fold-1").unwrap();
        let second = proposal.schedule.iter().find(|t| t.id == "fold-2").unwrap();
        assert_eq!(second.start - first.finish, 1800);
        assert!(
            proposal
                .schedule
                .iter()
                .find(|t| t.id == "bake-1")
                .unwrap()
                .start
                >= proposal
                    .schedule
                    .iter()
                    .find(|t| t.id == "fold-3")
                    .unwrap()
                    .finish
        );
    }
    #[test]
    fn delayed_dependencies_preview_a_later_finish_without_moving_completed_work() {
        let now = LocalTime {
            local: "2026-10-23T08:00".to_owned(),
            timezone: "Europe/Berlin".to_owned(),
            fold: Fold::Reject,
        }
        .resolve()
        .unwrap();
        let target = LocalTime {
            local: "2026-10-25T12:00".to_owned(),
            timezone: "Europe/Berlin".to_owned(),
            fold: Fold::Reject,
        };
        let mut run = run(
            &[
                stage("mix", vec![], 60, 0),
                stage("rest", vec!["mix".to_owned()], 0, 7200),
                stage("bake", vec!["rest".to_owned()], 60, 1800),
            ],
            target.clone(),
            now,
        );
        run.steps[0].timer = Timer::Completed {
            started_at: Some(now),
            completed_at: now + 60,
        };
        run.schedule = schedule(&run, now + 60, None).unwrap().schedule;
        let completed = run.steps[0].clone();
        let later = LocalTime {
            local: "2026-10-25T13:00".to_owned(),
            ..target
        };
        let (updated, proposal) = operations::replan(
            run,
            PlanChange::Delay {
                step_id: "rest-1".to_owned(),
                start_after: later.clone(),
            },
            now + 60,
        )
        .unwrap();
        assert_eq!(updated.steps[0], completed);
        assert_eq!(proposal.schedule[0].finish, now + 60);
        assert!(proposal.finish > later.resolve().unwrap());
        assert!(
            proposal
                .warnings
                .iter()
                .any(|w| w.code == "cooking_target_missed")
        );
        assert!(proposal.schedule[1].start >= later.resolve().unwrap());
    }
    #[test]
    fn passive_equipment_conflicts_and_invalid_graphs_are_explicit() {
        let now = 1_800_000_000;
        let target = LocalTime {
            local: "2027-01-15T12:00".to_owned(),
            timezone: "UTC".to_owned(),
            fold: Fold::Reject,
        };
        let mut a = stage("one", vec![], 0, 3600);
        let mut b = stage("two", vec![], 0, 3600);
        a.equipment = vec!["Oven".to_owned()];
        b.equipment = a.equipment.clone();
        let run = run(&[a.clone(), b], target, now);
        let proposal = schedule(&run, now, None).unwrap();
        assert!(
            proposal
                .warnings
                .iter()
                .any(|w| w.code == "cooking_equipment_overlap")
        );
        a.dependencies = vec!["one".to_owned()];
        assert_eq!(order(&[a]), Err("cooking_dependencies_invalid"));
    }
    #[test]
    fn repeated_steps_enforce_waits_after_actual_completion_and_explicit_start_delays() {
        let now = 1_800_000_000;
        let target = LocalTime {
            local: "2027-10-31T08:00".to_owned(),
            timezone: "UTC".to_owned(),
            fold: Fold::Reject,
        };
        let mut repeated = stage("fold", vec![], 10, 20);
        repeated.repeat = 2;
        repeated.wait_between_seconds = 3600;
        let mut run = run(&[repeated], target, now);
        run.schedule = schedule(&run, now, None).unwrap().schedule;
        let mut data = crate::cooking::model::Data {
            next_id: 2,
            runs: vec![run],
            ..Default::default()
        };
        operations::timer(&mut data, "cook-1", "fold-1", TimerAction::Start, now).unwrap();
        operations::timer(
            &mut data,
            "cook-1",
            "fold-1",
            TimerAction::Complete,
            now + 30,
        )
        .unwrap();
        assert_eq!(
            operations::timer(&mut data, "cook-1", "fold-2", TimerAction::Start, now + 30),
            Err("cooking_wait_not_elapsed")
        );
        data.runs[0].steps[1].not_before = Some(now + 4000);
        assert_eq!(
            operations::timer(
                &mut data,
                "cook-1",
                "fold-2",
                TimerAction::Start,
                now + 3630
            ),
            Err("cooking_wait_not_elapsed")
        );
        operations::timer(
            &mut data,
            "cook-1",
            "fold-2",
            TimerAction::Start,
            now + 4000,
        )
        .unwrap();
    }
    #[test]
    fn paused_projection_resumes_now_even_with_a_future_baking_target() {
        let now = 1_800_000_000;
        let target = LocalTime {
            local: "2027-10-31T08:00".to_owned(),
            timezone: "Europe/Berlin".to_owned(),
            fold: Fold::Reject,
        };
        let mut run = run(&[stage("one", vec![], 10, 20)], target, now);
        run.steps[0].timer = Timer::Paused {
            started_at: now - 100,
            paused_at: now - 90,
            active_remaining: 5,
            remaining: 20,
        };
        let proposed = schedule(&run, now, None).unwrap();
        assert_eq!(proposed.schedule[0].start, now);
        assert_eq!(proposed.schedule[0].finish, now + 20);
        assert!(
            proposed
                .warnings
                .iter()
                .any(|w| w.code == "cooking_paused_projection")
        );
    }
    #[test]
    fn overdue_pause_resume_does_not_restart_a_timer_duration() {
        let now = 1_800_000_000;
        let target = LocalTime {
            local: "2027-01-15T12:00".to_owned(),
            timezone: "UTC".to_owned(),
            fold: Fold::Reject,
        };
        let mut run = run(&[stage("one", vec![], 10, 20)], target, now);
        run.schedule = schedule(&run, now, None).unwrap().schedule;
        let mut data = crate::cooking::model::Data {
            next_id: 2,
            runs: vec![run],
            ..Default::default()
        };
        operations::timer(&mut data, "cook-1", "one-1", TimerAction::Start, now).unwrap();
        operations::timer(&mut data, "cook-1", "one-1", TimerAction::Pause, now + 90).unwrap();
        assert!(matches!(
            data.runs[0].steps[0].timer,
            Timer::Paused { remaining: 0, .. }
        ));
        operations::timer(
            &mut data,
            "cook-1",
            "one-1",
            TimerAction::Resume,
            now + 3600,
        )
        .unwrap();
        assert!(
            matches!(data.runs[0].steps[0].timer,Timer::Running {deadline,..} if deadline==now+3600)
        );
    }
}
