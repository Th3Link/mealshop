//! Shape and bounds of uploaded recipes and ingredients.
//!
//! Intake validates only what must hold for a submission to be storable and reviewable. It
//! never matches ingredients, converts units or judges tags; those are later check stages.
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

/// A rejected upload: a stable code for clients and an English explanation.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Invalid {
    pub(crate) code: &'static str,
    pub(crate) message: String,
}

fn invalid<T>(code: &'static str, message: impl Into<String>) -> Result<T, Invalid> {
    Err(Invalid {
        code,
        message: message.into(),
    })
}

fn text(value: &str, max: usize) -> bool {
    !value.trim().is_empty()
        && value.len() <= max
        && !value
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
}

fn optional(value: Option<&String>, max: usize) -> bool {
    value.is_none_or(|v| text(v, max))
}

fn slug(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// A language tag such as `de` or `pt-br`; the source language is kept, not restricted.
fn language(value: &str) -> bool {
    (2..=16).contains(&value.len())
        && value
            .split('-')
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_alphanumeric()))
}

/// An exact non-negative decimal with at most six fractional digits, never a float.
pub(crate) fn decimal(value: &str) -> bool {
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    let digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    !whole.is_empty()
        && whole.len() <= 12
        && digits(whole)
        && (whole == "0" || !whole.starts_with('0'))
        && fraction.len() <= 6
        && digits(fraction)
        && (fraction.is_empty() != value.contains('.'))
}

fn https(value: &str) -> Option<url::Url> {
    if value.len() > 2048 || value.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return None;
    }
    url::Url::parse(value).ok().filter(|u| {
        u.scheme() == "https"
            && u.host_str().is_some()
            && u.username().is_empty()
            && u.password().is_none()
    })
}

fn tags(values: &[String]) -> Result<(), Invalid> {
    let unique: BTreeSet<_> = values.iter().collect();
    if values.len() > 50 || unique.len() != values.len() || values.iter().any(|t| !slug(t, 64)) {
        return invalid(
            "invalid_tags",
            "Use at most 50 distinct tag ids of lowercase letters, digits and hyphens, for example \"lactose-free\"",
        );
    }
    Ok(())
}

/// The CDN origin image links must point to, for example `https://cdn.example`.
#[derive(Clone, Debug)]
pub(crate) struct CdnOrigin(url::Origin);

impl CdnOrigin {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match https(value) {
            Some(u) if u.path() == "/" && u.query().is_none() && u.fragment().is_none() => {
                Ok(Self(u.origin()))
            }
            _ => Err("Use an HTTPS origin without path, query or credentials".to_owned()),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ImageLink {
    url: String,
    alt: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Line {
    /// The line as the contributor wrote it.
    text: Option<String>,
    ingredient_id: Option<String>,
    amount: Option<String>,
    unit: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    text: String,
    section: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Recipe {
    title: String,
    language: Option<String>,
    author: Option<String>,
    source_url: Option<String>,
    servings: Option<u32>,
    yield_text: Option<String>,
    prep_time: Option<String>,
    cook_time: Option<String>,
    total_time: Option<String>,
    ingredients: Vec<Line>,
    #[serde(default)]
    steps: Vec<Step>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    image_links: Vec<ImageLink>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    name: String,
    url: String,
    record_id: String,
    version: String,
    retrieved_on: String,
    license: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reading {
    amount: String,
    unit: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Ingredient {
    name: String,
    language: Option<String>,
    #[serde(default)]
    aliases: Vec<String>,
    form: Option<String>,
    category: Option<String>,
    source: Option<Source>,
    /// Readings per 100 g edible portion, keyed by nutrient id.
    #[serde(default)]
    nutrients_per_100g: BTreeMap<String, Reading>,
    #[serde(default)]
    declared_allergens: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    note: Option<String>,
}

fn parse<'a, T: Deserialize<'a>>(body: &'a [u8]) -> Result<T, Invalid> {
    serde_json::from_slice(body).or_else(|error| {
        invalid(
            "invalid_json",
            format!("The body does not match the expected shape: {error}"),
        )
    })
}

fn unit(value: &str) -> bool {
    text(value, 32) && !value.contains(['\n', '\t'])
}

/// Checks an uploaded recipe; `cdn` is the configured image origin, if any.
pub(crate) fn recipe(body: &[u8], cdn: Option<&CdnOrigin>) -> Result<(), Invalid> {
    let r: Recipe = parse(body)?;
    if !text(&r.title, 200)
        || !optional(r.author.as_ref(), 200)
        || !optional(r.yield_text.as_ref(), 200)
        || !optional(r.prep_time.as_ref(), 200)
        || !optional(r.cook_time.as_ref(), 200)
        || !optional(r.total_time.as_ref(), 200)
        || r.language.as_ref().is_some_and(|l| !language(l))
        || r.servings.is_some_and(|s| s == 0 || s > 10_000)
    {
        return invalid(
            "invalid_recipe",
            "Use a title of 1–200 bytes, optional author, yield and time texts of at most 200 bytes, a language tag such as \"de\" and 1–10000 servings",
        );
    }
    if r.source_url.as_ref().is_some_and(|u| https(u).is_none()) {
        return invalid(
            "invalid_source_url",
            "Provide the original HTTPS recipe URL without embedded credentials",
        );
    }
    if r.ingredients.is_empty() || r.ingredients.len() > 100 {
        return invalid("invalid_ingredients", "Provide 1–100 ingredient lines");
    }
    for (index, line) in r.ingredients.iter().enumerate() {
        let n = index + 1;
        if line.text.is_none() && line.ingredient_id.is_none() {
            return invalid(
                "invalid_ingredients",
                format!("Ingredient line {n} needs its original text or an ingredient_id"),
            );
        }
        if !optional(line.text.as_ref(), 4000)
            || line.ingredient_id.as_ref().is_some_and(|i| !slug(i, 100))
            || line.unit.as_ref().is_some_and(|u| !unit(u))
        {
            return invalid(
                "invalid_ingredients",
                format!(
                    "Ingredient line {n}: use text of at most 4000 bytes, a lowercase ingredient_id and a unit of at most 32 bytes"
                ),
            );
        }
        if line.amount.as_ref().is_some_and(|a| !decimal(a)) {
            return invalid(
                "invalid_amount",
                format!(
                    "Ingredient line {n}: write the amount as an exact decimal string with at most six fractional digits, for example \"0.5\"; keep ranges and fractions in text"
                ),
            );
        }
    }
    if r.steps.len() > 100
        || r.steps
            .iter()
            .any(|s| !text(&s.text, 4000) || !optional(s.section.as_ref(), 200))
    {
        return invalid(
            "invalid_steps",
            "Use at most 100 nonempty steps of at most 4000 bytes",
        );
    }
    tags(&r.tags)?;
    image_links(&r.image_links, cdn)
}

fn image_links(links: &[ImageLink], cdn: Option<&CdnOrigin>) -> Result<(), Invalid> {
    if links.is_empty() {
        return Ok(());
    }
    let Some(cdn) = cdn else {
        return invalid(
            "image_links_unavailable",
            "Image links cannot be accepted yet because no CDN origin is configured; upload the recipe without image_links",
        );
    };
    if links.len() > 50 {
        return invalid("invalid_image_links", "Use at most 50 image links");
    }
    for (index, link) in links.iter().enumerate() {
        let on_cdn = https(&link.url).is_some_and(|u| u.origin() == cdn.0);
        if !on_cdn || !optional(link.alt.as_ref(), 500) {
            return invalid(
                "invalid_image_links",
                format!(
                    "Image link {}: use an HTTPS URL on the configured CDN origin and an alternative text of at most 500 bytes",
                    index + 1
                ),
            );
        }
    }
    Ok(())
}

/// Checks an uploaded ingredient.
pub(crate) fn ingredient(body: &[u8]) -> Result<(), Invalid> {
    let i: Ingredient = parse(body)?;
    if !text(&i.name, 200)
        || !optional(i.form.as_ref(), 200)
        || !optional(i.note.as_ref(), 4000)
        || i.language.as_ref().is_some_and(|l| !language(l))
        || i.category.as_ref().is_some_and(|c| !slug(c, 64))
        || i.aliases.len() > 50
        || i.aliases.iter().any(|a| !text(a, 200))
        || i.declared_allergens.len() > 50
        || i.declared_allergens.iter().any(|a| !slug(a, 64))
    {
        return invalid(
            "invalid_ingredient",
            "Use a name of 1–200 bytes, at most 50 aliases, lowercase category and allergen ids, a language tag such as \"de\" and a note of at most 4000 bytes",
        );
    }
    if let Some(s) = &i.source {
        let fields = [
            &s.name,
            &s.record_id,
            &s.version,
            &s.retrieved_on,
            &s.license,
        ];
        if fields.iter().any(|f| !text(f, 200)) || https(&s.url).is_none() {
            return invalid(
                "invalid_source",
                "A source needs name, HTTPS url, record_id, version, retrieved_on and license; omit the source when there is no primary record",
            );
        }
    }
    if !i.nutrients_per_100g.is_empty() && i.source.is_none() {
        return invalid(
            "nutrients_need_source",
            "Nutrient readings need the source record they come from; without one, upload the ingredient without nutrients",
        );
    }
    if i.nutrients_per_100g.len() > 200
        || i.nutrients_per_100g.iter().any(|(id, reading)| {
            !slug(&id.replace('_', "-"), 64) || !decimal(&reading.amount) || !unit(&reading.unit)
        })
    {
        return invalid(
            "invalid_nutrients",
            "Use at most 200 nutrient ids with exact decimal string amounts of at most six fractional digits and an explicit unit",
        );
    }
    tags(&i.tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimals_are_exact_strings() {
        for ok in ["0", "1", "250", "0.5", "12.125000"] {
            assert!(decimal(ok), "{ok}");
        }
        for bad in [
            "",
            ".5",
            "1.",
            "01",
            "1,5",
            "1/2",
            "-1",
            "1e3",
            "0.1234567",
            " 1",
        ] {
            assert!(!decimal(bad), "{bad}");
        }
    }

    #[test]
    fn cdn_origin_rejects_paths_and_plain_http() {
        assert!(CdnOrigin::parse("https://cdn.example").is_ok());
        assert!(CdnOrigin::parse("https://cdn.example/images").is_err());
        assert!(CdnOrigin::parse("http://cdn.example").is_err());
        assert!(CdnOrigin::parse("https://user@cdn.example").is_err());
    }

    #[test]
    fn slugs_are_lowercase_ids() {
        assert!(slug("lactose-free", 64));
        assert!(!slug("Lactose", 64));
        assert!(!slug("-x", 64));
        assert!(!slug("", 64));
    }
}
