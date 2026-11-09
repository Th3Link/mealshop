# Native Nextcloud daily workflows

Status: Done
Origin: Customer, 2026-11-09, after PR #15 merge

## Acceptance contract

1. A persistent native household switcher changes context without visiting household management. Opening the app restores the last accessible household for the current account; explicit household management and deep links remain available. Revoked households never become an authority through saved preferences.
2. Recipe creation has a plus icon. Author attribution is optional. Exact quantities omit redundant trailing fractional zeros for display, without changing stored input or precision. Recipe images are optional, private to the household and persist across restart.
3. Recipe ingredient selection supports multilingual search, quick keyboard selection and inline creation without losing the recipe draft. A labelled trash icon ends each ingredient row.
4. The weekly meal calendar has breakfast, lunch, snack and dinner slots. People may attend or skip each meal independently; serving estimates and shopping changes remain explicit and reviewed. Existing plans remain editable and no calendar provider is implied.
5. Invitations use native searchable account selection with Nextcloud's actual sharing visibility rules, retaining addressed invitation acceptance and household roles.
6. Ingredient search retains focus while typing. Household editors can create and archive ingredients; readers cannot mutate shared catalogue data. Bundled primary records are not destroyed. Favourites persist and reverse lookup lists matching saved recipes.
7. Ingredients support household tags and explicit composition. Ingredient IDs remain language independent. Composition references are validated and cannot cycle. Declared allergen information and nested ingredients participate in conservative hard exclusions; ordinary tags cannot assert allergy safety. Missing composition/nutrition remains unknown.
8. Existing source snapshots, recipe revisions, health-data isolation, role revocation, exact quantities, host theme and accessible mobile operation remain intact. Backend changes have restart/conflict/permission tests and all new UI paths are exercised in the real test instance.

## Safe implementation decisions

Household catalogue additions and annotations are shared within that household only. Removing a referenced ingredient archives it for new selection; existing recipe snapshots remain readable. User declarations are labelled as declarations, never primary nutrient records. Composition membership does not imply a known mass ratio or nutrient composition. No automatic nutritional estimate is made from a composite's name or nearby food.

## Links
F02/F04/F09/F10/F22 remain the domain contracts. See ux-notes.md, adrs.md and plan.md for this customer-authorized extension.
