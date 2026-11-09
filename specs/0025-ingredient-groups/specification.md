# Ingredient groups, declared nutrition and classification

Status: In progress
Origin: Customer, after PR #18 merge

1. Household ingredients form explicit groups with one level of subtypes. Existing wheat, rye and spelt flour records are organised without changing identities or copying nutrient profiles.
2. Editors can enter exact decimal nutrient values per 100 g, with named units, source/evidence notes and explicit unknowns. Readers cannot edit. Existing primary records and historical recipes retain their sources.
3. Subtypes explicitly choose reference, own or inherited nutrition. Inherited data retains its group identity and declaration provenance; missing values remain unknown. Recipe revisions pin the effective values and group so subsequent changes cannot rewrite old calculations.
4. Dietary class (unknown/vegan/vegetarian/meat or fish) and origin (unknown/plant/animal/mixed) are separate, explicit declarations. They do not replace allergens or establish safety.
5. Grouped browsing, subtype search and recipe selection remain native, responsive and keyboard accessible. Existing annotations, favourites and archive actions preserve group/nutrition data.
6. Validate references, prevent hierarchy cycles, retain quantities and enforce existing revision/permission checks. Verify persistence, recipe calculations and real Nextcloud flows.
