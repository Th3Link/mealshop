use super::{
    model::{Equipment, EquipmentInput, Receipt, Recipe, Record},
    validation,
};
use crate::cooking::model::Data as Processes;
use serde::{Deserialize, Serialize};
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Command {
    CreateBatch {
        draft: super::model::BatchDraft,
    },
    Replan {
        run_id: String,
        change: crate::cooking::operations::PlanChange,
    },
    Timer {
        run_id: String,
        step_id: String,
        action: crate::cooking::operations::TimerAction,
    },
    Cancel {
        run_id: String,
    },
    ArchiveBatch {
        run_id: String,
        archived: bool,
    },
    Subscribe {
        run_id: String,
        enabled: bool,
        language: String,
        permission: crate::cooking::model::Permission,
    },
    Acknowledge {
        notice_id: String,
    },
    Retry {
        notice_id: String,
        duplicate_risk_accepted: bool,
    },
    AddReading {
        run_id: String,
        reading: super::model::ReadingInput,
        corrects: Option<String>,
        reason: Option<String>,
    },
    SaveRecipe {
        id: Option<String>,
        expected_recipe_revision: Option<u64>,
        recipe: Recipe,
    },
    ArchiveRecipe {
        id: String,
        archived: bool,
    },
    SaveEquipment {
        id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected_equipment: Option<EquipmentInput>,
        equipment: EquipmentInput,
    },
    ArchiveEquipment {
        id: String,
        archived: bool,
    },
}
pub(super) fn replay(
    data: &Processes,
    actor: &str,
    key: &str,
    expected: u64,
    command: &Command,
) -> Result<Option<String>, &'static str> {
    let Some(r) = data
        .brewing
        .operations
        .iter()
        .find(|r| r.actor == actor && r.key == key)
    else {
        return Ok(None);
    };
    if r.expected_revision != expected
        || r.command != serde_json::to_value(command).map_err(|_| "brewing_storage_failed")?
    {
        return Err("brewing_request_conflict");
    }
    Ok(Some(r.id.clone()))
}
pub(super) fn result(data: &Processes, operation: &str) -> Result<Option<String>, &'static str> {
    data.brewing
        .operations
        .iter()
        .find(|r| r.id == operation)
        .map(|r| r.result_id.clone())
        .ok_or("brewing_data_invalid")
}
pub(super) fn execute(
    data: &mut Processes,
    identity: &crate::cooking::Actor,
    key: &str,
    expected: u64,
    command: Command,
    now: i64,
) -> Result<String, &'static str> {
    let actor = &identity.user;
    let id = data.allocate("brew-operation")?;
    let result_id = match &command {
        Command::CreateBatch { draft } => Some(super::batches::create(data, draft, actor, now)?),
        Command::AddReading {
            run_id,
            reading,
            corrects,
            reason,
        } => Some(super::readings::append(
            data,
            run_id,
            reading,
            corrects.as_deref(),
            reason.as_deref(),
            actor,
            now,
        )?),
        Command::SaveRecipe {
            id,
            expected_recipe_revision,
            recipe,
        } => Some(save_recipe(
            data,
            actor,
            now,
            id.as_deref(),
            *expected_recipe_revision,
            recipe,
        )?),
        Command::ArchiveRecipe { id, archived } => {
            if !data.brewing.recipes.iter().any(|r| r.id == *id) {
                return Err("brewing_recipe_not_found");
            }
            data.brewing.archived_recipes.retain(|v| v != id);
            if *archived {
                data.brewing.archived_recipes.push(id.clone());
            }
            Some(id.clone())
        }
        Command::SaveEquipment {
            id,
            expected_equipment,
            equipment,
        } => {
            validation::equipment(equipment)?;
            let id = if let Some(id) = id {
                let e = data
                    .brewing
                    .equipment
                    .iter_mut()
                    .find(|e| e.id == *id)
                    .ok_or("brewing_equipment_not_found")?;
                if expected_equipment.as_ref() != Some(&e.details) {
                    return Err("brewing_equipment_conflict");
                }
                e.details = equipment.clone();
                id.clone()
            } else {
                if expected_equipment.is_some() {
                    return Err("brewing_equipment_conflict");
                }
                let id = data.allocate("brew-equipment")?;
                data.brewing.equipment.push(Equipment {
                    id: id.clone(),
                    details: equipment.clone(),
                    archived: false,
                });
                id
            };
            Some(id)
        }
        Command::ArchiveEquipment { id, archived } => {
            let e = data
                .brewing
                .equipment
                .iter_mut()
                .find(|e| e.id == *id)
                .ok_or("brewing_equipment_not_found")?;
            e.archived = *archived;
            Some(id.clone())
        }
        other => Some(super::controls::execute(data, other, identity, now)?),
    };
    data.brewing.operations.push(Receipt {
        id: id.clone(),
        actor: actor.clone(),
        key: key.to_owned(),
        expected_revision: expected,
        command: serde_json::to_value(command).map_err(|_| "brewing_storage_failed")?,
        recorded_at: now,
        result_id,
    });
    Ok(id)
}

fn save_recipe(
    data: &mut Processes,
    actor: &str,
    now: i64,
    id: Option<&str>,
    expected_recipe_revision: Option<u64>,
    recipe: &Recipe,
) -> Result<String, &'static str> {
    validation::recipe(recipe, true)?;
    let head = id.and_then(|id| {
        data.brewing
            .recipes
            .iter()
            .filter(|r| r.id == id)
            .map(|r| r.revision)
            .max()
    });
    if id.is_some() && head.is_none() {
        return Err("brewing_recipe_not_found");
    }
    if head != expected_recipe_revision {
        return Err("brewing_recipe_conflict");
    }
    let id = match id {
        Some(id) => id.to_owned(),
        None => data.allocate("brew-recipe")?,
    };
    data.brewing.recipes.push(Record {
        id: id.clone(),
        revision: head.unwrap_or(0).checked_add(1).ok_or("brewing_capacity")?,
        recipe: recipe.clone(),
        actor: actor.to_owned(),
        recorded_at: now,
    });
    Ok(id)
}
