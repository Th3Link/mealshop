# MealShop — Architecture and product decision log

## Proposed defaults (not yet formally accepted)
- Shared household meal plan as the default; preserve flexibility for individual meal plans later.
- Server generates meal plans; mobile client edits and displays them.
- Five primary mobile tabs: Today, Plan, Shopping, Recipes, More.
- Diet categories are presets layered over explicit food rules.
- Shopping list remains useful without meal planning.

## Questions to resolve
1. Which mobile stack and repository folder is authoritative (Flutter planned or already implemented)?
2. What exact API contract is shared by Nextcloud and standalone modes?
3. Household role/permission policy, especially for children's profiles and sensitive health data?
4. Should per-person independent plans be a first-release feature or only a model extension?
5. Pantry tracking MVP scope: none, manual quantities, or full lots?
6. Source of product/barcode/nutrition data and licensing?
7. Which OCR/AI deployment options are acceptable for privacy and cost?
8. What offline synchronization semantics and notification delivery infrastructure exist?
9. What level of nutrition calculations is acceptable when source data is incomplete?

Record each final decision with date, rationale, alternatives, consequences and owner.
