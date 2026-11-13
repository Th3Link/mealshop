use crate::cooking::{self, model::Data};
use serde_json::{Value, json};
pub(super) fn view(
    data: &Data,
    now: i64,
    actor: &cooking::Actor,
    health: &Value,
) -> Result<Value, &'static str> {
    let recipes = data
        .brewing
        .recipes
        .iter()
        .filter(|r| {
            !data
                .brewing
                .recipes
                .iter()
                .any(|new| new.id == r.id && new.revision > r.revision)
        })
        .map(|r| json!({"record":r,"archived":data.brewing.archived_recipes.contains(&r.id)}))
        .collect::<Vec<_>>();
    let batches = data.brewing.batches.iter().map(|(id, meta)| Ok(json!({"run":cooking::view::run(data.run(id)?,actor,now)?,"batch":meta,"reading_count":data.brewing.readings.iter().filter(|r| &r.run_id == id).count()}))).collect::<Result<Vec<_>, &'static str>>()?;
    Ok(
        json!({"revision":data.revision,"server_now":now,"notification_transport":actor.notifications_available,"scheduler":health,"recipes":recipes,"equipment":data.brewing.equipment,"batches":batches}),
    )
}
pub(super) fn readings(
    data: &Data,
    run_id: &str,
    before: Option<&str>,
) -> Result<Value, &'static str> {
    if !data.brewing.batches.contains_key(run_id) {
        return Err("brewing_batch_not_found");
    }
    let all = data
        .brewing
        .readings
        .iter()
        .rev()
        .filter(|r| r.run_id == run_id)
        .collect::<Vec<_>>();
    let start = before
        .map(|id| {
            all.iter()
                .position(|r| r.id == id)
                .map(|i| i + 1)
                .ok_or("brewing_reading_not_found")
        })
        .transpose()?
        .unwrap_or(0);
    let page = all.iter().skip(start).take(50).copied().collect::<Vec<_>>();
    let next = if start + page.len() < all.len() {
        page.last().map(|r| &r.id)
    } else {
        None
    };
    Ok(json!({"readings":page,"next":next,"total":all.len()}))
}
pub(super) fn detail(
    data: &Data,
    id: &str,
    now: i64,
    actor: &cooking::Actor,
    health: &Value,
) -> Result<Value, &'static str> {
    let meta = data
        .brewing
        .batches
        .get(id)
        .ok_or("brewing_batch_not_found")?;
    let recipe = data
        .brewing
        .recipe(&meta.draft.recipe_id, meta.draft.recipe_revision)?;
    let notices = data
        .notices
        .iter()
        .filter(|n| n.run_id == id && n.user == actor.user)
        .map(|n| {
            let mut value = serde_json::to_value(n).map_err(|_| "brewing_storage_failed")?;
            if let Some(map) = value.as_object_mut() {
                map.remove("user");
            }
            Ok(value)
        })
        .collect::<Result<Vec<_>, &'static str>>()?;
    Ok(
        json!({"revision":data.revision,"server_now":now,"notification_transport":actor.notifications_available,"scheduler":health,"run":cooking::view::run(data.run(id)?,actor,now)?,"batch":meta,"recipe":recipe,"notices":notices,"measurements":readings(data,id,None)?}),
    )
}
