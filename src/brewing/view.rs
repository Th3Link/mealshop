use crate::cooking::model::Data;
use serde_json::{Value, json};
pub(super) fn view(data: &Data, now: i64) -> Value {
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
    json!({"revision":data.revision,"server_now":now,"recipes":recipes,"equipment":data.brewing.equipment,"batches":[],"readings":[]})
}
