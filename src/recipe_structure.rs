//! Validates recipe-owned parts and explicit preparation references.
use crate::recipes::Recipe;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Part {
    pub(crate) id: String,
    pub(crate) name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) uses_parts: Vec<String>,
}
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 100
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
pub(crate) fn validate(recipe: &Recipe) -> Result<(), String> {
    let parts: BTreeSet<_> = recipe.parts.iter().map(|p| p.id.as_str()).collect();
    let ids: BTreeSet<_> = recipe
        .lines
        .iter()
        .filter_map(|l| l.id.as_deref())
        .collect();
    if recipe.parts.len() > 30
        || parts.len() != recipe.parts.len()
        || ids.len() != recipe.lines.iter().filter(|l| l.id.is_some()).count()
        || ids.iter().any(|id| !valid_id(id))
        || recipe.parts.iter().any(|p| {
            !valid_id(&p.id)
                || p.name.trim().is_empty()
                || p.name.len() > 200
                || p.name.chars().any(char::is_control)
        })
        || recipe.lines.iter().any(|l| {
            l.part_id
                .as_ref()
                .is_some_and(|id| !parts.contains(id.as_str()))
        })
    {
        return Err(
            "Use unique recipe line/part IDs, existing parts and nonempty part names".to_owned(),
        );
    }
    let mut edges: BTreeMap<&str, Vec<&str>> = recipe
        .parts
        .iter()
        .map(|p| {
            (
                p.id.as_str(),
                p.uses_parts.iter().map(String::as_str).collect(),
            )
        })
        .collect();
    for step in &recipe.steps {
        if step
            .part_id
            .as_ref()
            .is_some_and(|id| !parts.contains(id.as_str()))
            || step.ingredient_line_ids.len() > 100
            || step
                .ingredient_line_ids
                .iter()
                .collect::<BTreeSet<_>>()
                .len()
                != step.ingredient_line_ids.len()
            || step
                .ingredient_line_ids
                .iter()
                .any(|id| !ids.contains(id.as_str()))
            || step.prepared_parts.len() > 30
            || step.prepared_parts.iter().collect::<BTreeSet<_>>().len()
                != step.prepared_parts.len()
            || step
                .prepared_parts
                .iter()
                .any(|id| !parts.contains(id.as_str()))
        {
            return Err(
                "Preparation steps must reference existing ingredient lines and recipe parts"
                    .to_owned(),
            );
        }
        if let Some(id) = &step.part_id {
            edges
                .entry(id)
                .or_default()
                .extend(step.prepared_parts.iter().map(String::as_str));
        }
    }
    for part in &recipe.parts {
        if part.uses_parts.len() > 30
            || part.uses_parts.iter().collect::<BTreeSet<_>>().len() != part.uses_parts.len()
            || part
                .uses_parts
                .iter()
                .any(|id| !parts.contains(id.as_str()))
        {
            return Err("Recipe part dependencies must be unique existing parts".to_owned());
        }
    }
    let mut complete = BTreeSet::new();
    for id in parts {
        if !visit(id, &edges, &mut BTreeSet::new(), &mut complete) {
            return Err("Recipe parts cannot depend on themselves or form a cycle".to_owned());
        }
    }
    Ok(())
}

fn visit<'a>(
    id: &'a str,
    edges: &BTreeMap<&'a str, Vec<&'a str>>,
    visiting: &mut BTreeSet<&'a str>,
    complete: &mut BTreeSet<&'a str>,
) -> bool {
    if complete.contains(id) {
        return true;
    }
    if !visiting.insert(id) {
        return false;
    }
    if edges.get(id).is_some_and(|deps| {
        deps.iter()
            .any(|next| !visit(next, edges, visiting, complete))
    }) {
        return false;
    }
    visiting.remove(id);
    complete.insert(id);
    true
}
