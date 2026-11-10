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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) starts_after: Vec<String>,
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
                p.uses_parts
                    .iter()
                    .chain(&p.starts_after)
                    .map(String::as_str)
                    .collect(),
            )
        })
        .collect();
    let mut outputs = BTreeSet::new();
    for line in &recipe.lines {
        if let Some(source) = &line.prepared_part {
            if !parts.contains(source.as_str()) || !outputs.insert(source.as_str()) {
                return Err("Homemade ingredients require an existing, distinct preparation part per output".to_owned());
            }
            if let Some(owner) = &line.part_id {
                edges.entry(owner).or_default().push(source);
            }
        }
    }
    validate_step_references(recipe, &parts, &ids)?;
    for step in &recipe.steps {
        if let Some(id) = &step.part_id {
            edges
                .entry(id)
                .or_default()
                .extend(step.prepared_parts.iter().map(String::as_str));
        }
    }
    for part in &recipe.parts {
        if part.starts_after.len() > 30
            || part.starts_after.iter().collect::<BTreeSet<_>>().len() != part.starts_after.len()
            || part
                .starts_after
                .iter()
                .any(|id| !parts.contains(id.as_str()))
            || part.uses_parts.len() > 30
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
    for source in outputs {
        if !recipe
            .steps
            .iter()
            .any(|step| step.part_id.as_deref() == Some(source))
            || !has_inputs(recipe, source, &mut BTreeSet::new())
        {
            return Err("Homemade preparation parts require measured base ingredients and preparation steps".to_owned());
        }
    }
    Ok(())
}

fn validate_step_references(
    recipe: &Recipe,
    parts: &BTreeSet<&str>,
    ids: &BTreeSet<&str>,
) -> Result<(), String> {
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
    }
    Ok(())
}

fn has_inputs(recipe: &Recipe, id: &str, visited: &mut BTreeSet<String>) -> bool {
    if !visited.insert(id.to_owned()) {
        return false;
    }
    recipe
        .lines
        .iter()
        .filter(|line| line.part_id.as_deref() == Some(id))
        .any(|line| {
            line.prepared_part
                .as_ref()
                .is_none_or(|source| has_inputs(recipe, source, visited))
        })
        || recipe
            .parts
            .iter()
            .filter(|part| part.id == id)
            .flat_map(|part| &part.uses_parts)
            .chain(
                recipe
                    .steps
                    .iter()
                    .filter(|step| step.part_id.as_deref() == Some(id))
                    .flat_map(|step| &step.prepared_parts),
            )
            .any(|source| has_inputs(recipe, source, visited))
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
