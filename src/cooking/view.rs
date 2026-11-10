use super::{
    model::{Data, Run, RunState, Timer},
    store::Prepared,
    time,
};
use serde_json::{Value, json};
pub(crate) fn run(value: &Run, actor: &super::Actor, now: i64) -> Result<Value, &'static str> {
    let mut body = serde_json::to_value(value).map_err(|_| "cooking_storage_failed")?;
    if let Some(map) = body.as_object_mut() {
        map.remove("subscriptions");
        map.insert(
            "subscription".to_owned(),
            json!(value.subscriptions.get(&actor.user)),
        );
    }
    body["local_schedule"]=json!(value.schedule.iter().map(|t|Ok(json!({"id":t.id,"start":time::display(t.start,&value.timezone)?,"active_until":time::display(t.active_until,&value.timezone)?,"finish":time::display(t.finish,&value.timezone)?,"earliest":time::display(t.earliest,&value.timezone)?,"latest":time::display(t.latest,&value.timezone)?}))).collect::<Result<Vec<_>,&'static str>>()?);
    body["steps"] = json!(
        value
            .steps
            .iter()
            .map(|step| {
                let mut body = serde_json::to_value(step).map_err(|_| "cooking_storage_failed")?;
                let available = if matches!(step.timer, Timer::Pending) {
                    super::source::available_at(value, step)?
                } else {
                    None
                };
                body["available_at"] = json!(available);
                body["ready"] = json!(
                    value.status == RunState::Active
                        && matches!(step.timer, Timer::Pending)
                        && available.is_some_and(|at| at <= now)
                );
                Ok(body)
            })
            .collect::<Result<Vec<_>, &'static str>>()?
    );
    Ok(body)
}
pub(crate) fn prepared(value: &Prepared, actor: &super::Actor) -> Result<Value, &'static str> {
    Ok(
        json!({"preview_id":value.preview_id,"revision":value.revision,"expires_at":value.expires_at,"run":run(&value.run,actor,time::now()?)?,"previous":value.previous.as_ref().map(|r|run(r,actor,time::now()?)).transpose()?,"proposal":value.proposal}),
    )
}
pub(crate) fn list(data: &Data, actor: &super::Actor, now: i64) -> Value {
    let mut runs=data.runs.iter().map(|run| {
        let upcoming=run.schedule.iter().filter(|t|run.steps.iter().any(|s|s.stage.id==t.id&&matches!(s.timer,Timer::Pending))).min_by_key(|t|t.start);
        let running=run.steps.iter().filter(|s|matches!(s.timer,Timer::Running {..})).count();
        let overdue=run.steps.iter().filter(|s|matches!(s.timer,Timer::Running {deadline,..} if deadline<=now)).count();
        json!({"plan_slot":run.plan_slot,"calendar_entries":run.schedule.iter().filter(|t| run.status == RunState::Active && run.steps.iter().any(|s| s.stage.id == t.id && !matches!(s.timer, Timer::Completed {..} | Timer::Cancelled))).map(|t| { let name=run.steps.iter().find(|s| s.stage.id == t.id).map(|s| &s.stage.name); json!({"step_id":t.id,"at":t.start,"name":name}) }).collect::<Vec<_>>(),"id":run.id,"name":run.name,"kind":run.kind,"recipe":run.recipe,"servings":run.servings,"timezone":run.timezone,"status":run.status,"created_at":run.created_at,"running":running,"overdue":overdue,"next":upcoming.map(|t|json!({"step_id":t.id,"at":t.start,"name":run.steps.iter().find(|s|s.stage.id==t.id).map(|s|&s.stage.name)}))})
    }).collect::<Vec<_>>();
    runs.sort_by_key(|run| {
        (
            run["status"] != "active",
            std::cmp::Reverse(run["created_at"].as_i64().unwrap_or_default()),
        )
    });
    json!({"revision":data.revision,"server_now":now,"notification_transport":actor.notifications_available,"runs":runs})
}
pub(crate) fn detail(
    data: &Data,
    id: &str,
    actor: &super::Actor,
    now: i64,
    ingredients: &Value,
) -> Result<Value, &'static str> {
    let notices = data
        .notices
        .iter()
        .filter(|n| n.run_id == id && n.user == actor.user)
        .map(|notice| {
            let mut value = serde_json::to_value(notice).map_err(|_| "cooking_storage_failed")?;
            if let Some(map) = value.as_object_mut() {
                map.remove("user");
            }
            Ok(value)
        })
        .collect::<Result<Vec<_>, &'static str>>()?;
    Ok(
        json!({"revision":data.revision,"server_now":now,"notification_transport":actor.notifications_available,"run":run(data.run(id)?,actor,now)?,"notices":notices,"ingredients":ingredients}),
    )
}
