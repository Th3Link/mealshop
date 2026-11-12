use super::parser::extract;
#[test]
fn graph_recipes_nested_steps_original_text_and_unknown_fields_are_preserved() {
    let document = r#"<html lang="de"><script>throw new Error('never execute')</script><script type="application/ld+json">{"@graph":[{"@type":"Recipe","name":"Brot &amp; Butter","author":{"@type":"Person","name":"Synthetic author"},"recipeYield":"4 Portionen","recipeIngredient":["100,500 g Haferkorn","Salz nach Geschmack"],"recipeInstructions":[{"@type":"HowToSection","name":"Dough","itemListElement":[{"@type":"HowToStep","text":"<p>Mix</p><p>carefully.</p>"}]}]}]}</script></html>"#;
    let parsed = extract(document).unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].title.as_deref(), Some("Brot & Butter"));
    assert_eq!(parsed[0].language.as_deref(), Some("de"));
    assert_eq!(parsed[0].ingredients[0], "100,500 g Haferkorn");
    assert_eq!(parsed[0].steps[0].text, "Mix carefully.");
    assert_eq!(parsed[0].steps[0].section.as_deref(), Some("Dough"));
    assert!(parsed[0].cook_time.is_none());
    let unknown = extract(r#"{"@type":["Recipe"],"name":"Synthetic"}"#).unwrap();
    assert!(unknown[0].yield_text.is_none() && unknown[0].ingredients.is_empty());
}
#[test]
fn unsupported_paywalls_oversized_or_malformed_sources_do_not_become_recipes() {
    assert!(extract("<html>Sign in to view this recipe</html>").is_err());
    assert!(extract(r#"<script type="application/ld+json">broken</script>"#).is_err());
    assert!(
        extract(r#"{"@type":"Recipe","isAccessibleForFree":false,"recipeInstructions":"private"}"#)
            .is_err()
    );
    let large = serde_json::json!({"@type":"Recipe","recipeIngredient":vec!["1 g oats";101]});
    assert!(extract(&large.to_string()).is_err());
}

#[test]
fn public_fixture_extracts_without_loading_assets_or_importing_its_health_claims() {
    let recipes = extract(include_str!("../../tests/fixtures/import-recipe.html")).unwrap();
    assert_eq!(recipes.len(), 1);
    assert_eq!(recipes[0].steps[0].text, "Mix the synthetic ingredients.");
    assert_eq!(recipes[0].total_time.as_deref(), Some("PT3M"));
    let data = serde_json::to_string(&recipes).unwrap();
    assert!(
        !data.contains("99999")
            && !data.contains("GlutenFreeDiet")
            && !data.contains("must-not-execute")
    );
}
