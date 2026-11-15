//! A bounded agenda of canonical preparation starts and actual timer deadlines.
use super::{
    model::{Data, RunState, Timer},
    time,
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Range {
    from: i64,
    until: i64,
    #[serde(default)]
    offset: usize,
    limit: Option<usize>,
}
pub(super) fn agenda(data: &Data, range: &Range, now: i64) -> Result<Value, &'static str> {
    let limit = range.limit.unwrap_or(100);
    if !time::valid(range.from)
        || !time::valid(range.until)
        || range.until <= range.from
        || range.until - range.from > 604_800
        || !(1..=200).contains(&limit)
        || range.offset > 100_000
    {
        return Err("cooking_calendar_range_invalid");
    }
    let mut rows = Vec::new();
    for run in &data.runs {
        if run.status != RunState::Active
            || data
                .brewing
                .batches
                .get(&run.id)
                .is_some_and(|b| b.archived)
        {
            continue;
        }
        for step in &run.steps {
            let (at, event) = match step.timer {
                Timer::Pending => {
                    let Some(timing) = run.schedule.iter().find(|t| t.id == step.stage.id) else {
                        continue;
                    };
                    (timing.start, "start")
                }
                Timer::Running {
                    started_at,
                    deadline,
                    ..
                } if deadline > started_at => (deadline, "deadline"),
                _ => continue,
            };
            if at >= range.from && at < range.until {
                rows.push((at, &run.id, &step.stage.id, json!({"run_id":run.id,"run_name":run.name,"kind":run.kind,"step_id":step.stage.id,"step_name":step.stage.name,"event":event,"at":at,"local":time::display(at,&run.timezone)?,"timezone":run.timezone,"plan_slot":run.plan_slot})));
            }
        }
    }
    rows.sort_by(|a, b| (&a.0, a.1, a.2).cmp(&(&b.0, b.1, b.2)));
    let total = rows.len();
    Ok(
        json!({"revision":data.revision,"server_now":now,"total":total,"offset":range.offset,"limit":limit,"entries":rows.into_iter().skip(range.offset).take(limit).map(|r|r.3).collect::<Vec<_>>()}),
    )
}
