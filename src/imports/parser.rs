//! Extract data only. HTML scripts and JSON-LD contexts are never executed or fetched.
use super::model::{Candidate, SourceStep};

use serde_json::Value;
fn plain(raw: &str) -> String {
    super::html::scan(raw, true)
        .text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
fn string(value: &Value) -> Option<String> {
    let raw = value.as_str()?;
    let text = plain(raw);
    (!text.is_empty()).then_some(text)
}
fn has_type(value: &Value, expected: &str) -> bool {
    let is = |s: &str| {
        s == expected
            || s == format!("https://schema.org/{expected}")
            || s == format!("http://schema.org/{expected}")
    };
    value["@type"].as_str().is_some_and(is)
        || value["@type"]
            .as_array()
            .is_some_and(|types| types.iter().filter_map(Value::as_str).any(is))
}
fn blocked(value: &Value, depth: usize) -> bool {
    if depth > 32 {
        return true;
    }
    match value {
        Value::Object(fields) => {
            fields
                .get("isAccessibleForFree")
                .is_some_and(|v| v == false || v.as_str() == Some("false"))
                || fields.values().any(|v| blocked(v, depth + 1))
        }
        Value::Array(values) => values.iter().any(|v| blocked(v, depth + 1)),
        _ => false,
    }
}
fn steps(
    value: &Value,
    section: Option<&str>,
    output: &mut Vec<SourceStep>,
    depth: usize,
) -> Result<(), &'static str> {
    if depth > 12 || output.len() > 100 {
        return Err("import_content_large");
    }
    match value {
        Value::String(raw) => {
            let text = plain(raw);
            if !text.is_empty() {
                output.push(SourceStep {
                    text,
                    section: section.map(str::to_owned),
                });
            }
        }
        Value::Array(values) => {
            for v in values {
                steps(v, section, output, depth + 1)?;
            }
        }
        Value::Object(_) => {
            if has_type(value, "HowToSection") || value.get("itemListElement").is_some() {
                let heading = string(&value["name"]);
                steps(
                    &value["itemListElement"],
                    heading.as_deref().or(section),
                    output,
                    depth + 1,
                )?;
            } else if let Some(text) = string(&value["text"]).or_else(|| string(&value["name"])) {
                output.push(SourceStep {
                    text,
                    section: section.map(str::to_owned),
                });
            }
        }
        _ => {}
    }
    Ok(())
}
fn ingredients(value: &Value, output: &mut Vec<String>, depth: usize) -> Result<(), &'static str> {
    if depth > 12 || output.len() > 100 {
        return Err("import_content_large");
    }
    match value {
        Value::String(raw) => {
            for line in raw.lines() {
                let text = plain(line);
                if !text.is_empty() {
                    output.push(text);
                }
            }
        }
        Value::Array(values) => {
            for v in values {
                ingredients(v, output, depth + 1)?;
            }
        }
        Value::Object(_) => {
            if value.get("itemListElement").is_some() {
                ingredients(&value["itemListElement"], output, depth + 1)?;
            } else if let Some(text) = string(&value["text"]).or_else(|| string(&value["name"])) {
                output.push(text);
            }
        }
        _ => {}
    }
    Ok(())
}
fn scalar(value: &Value) -> Option<String> {
    if let Some(values) = value.as_array() {
        let parts = values.iter().filter_map(scalar).collect::<Vec<_>>();
        return (!parts.is_empty()).then(|| parts.join(" / "));
    }
    string(value)
        .or_else(|| value.as_u64().map(|v| v.to_string()))
        .or_else(|| string(&value["name"]))
}
fn candidate(value: &Value, language: Option<&str>) -> Result<Candidate, &'static str> {
    let mut ingredients_list = Vec::new();
    ingredients(&value["recipeIngredient"], &mut ingredients_list, 0)?;
    let mut instructions = Vec::new();
    steps(&value["recipeInstructions"], None, &mut instructions, 0)?;
    let candidate = Candidate {
        title: string(&value["name"]),
        author: scalar(&value["author"]),
        language: scalar(&value["inLanguage"]).or_else(|| language.map(str::to_owned)),
        yield_text: scalar(&value["recipeYield"]),
        ingredients: ingredients_list,
        steps: instructions,
        prep_time: scalar(&value["prepTime"]),
        cook_time: scalar(&value["cookTime"]),
        total_time: scalar(&value["totalTime"]),
    };
    if !candidate.valid() {
        return Err("import_content_large");
    }
    Ok(candidate)
}
fn collect(
    value: &Value,
    language: Option<&str>,
    output: &mut Vec<Candidate>,
    depth: usize,
    nodes: &mut usize,
) -> Result<(), &'static str> {
    *nodes += 1;
    if depth > 32 || *nodes > 4096 || output.len() > 16 {
        return Err("import_content_large");
    }
    match value {
        Value::Object(fields) => {
            if has_type(value, "Recipe") {
                output.push(candidate(value, language)?);
            } else {
                for child in fields.values() {
                    if child.is_object() || child.is_array() {
                        collect(child, language, output, depth + 1, nodes)?;
                    }
                }
            }
        }
        Value::Array(values) => {
            for child in values {
                collect(child, language, output, depth + 1, nodes)?;
            }
        }
        _ => {}
    }
    Ok(())
}
pub(super) fn extract(document: &str) -> Result<Vec<Candidate>, &'static str> {
    let mut values = Vec::new();
    let mut language = None;
    if let Ok(value) = serde_json::from_str::<Value>(document.trim_start_matches('\u{feff}')) {
        values.push(value);
    } else {
        let data = super::html::scan(document, false);
        if data.overflow {
            return Err("import_content_large");
        }
        language = data.language;
        for raw in data.scripts {
            if let Ok(value) = serde_json::from_str::<Value>(raw.trim_start_matches('\u{feff}')) {
                values.push(value);
            }
        }
    }
    let mut output = Vec::new();
    let mut nodes = 0;
    for value in values {
        if blocked(&value, 0) {
            return Err("import_source_blocked");
        }
        collect(&value, language.as_deref(), &mut output, 0, &mut nodes)?;
    }
    if output.is_empty() {
        return Err("import_no_recipe");
    }
    if output.len() > 16
        || serde_json::to_vec(&output)
            .map_err(|_| "import_parser_failed")?
            .len()
            > 98_304
    {
        return Err("import_content_large");
    }
    Ok(output)
}
