# Ingredient catalogue and nutrition

Status: In progress
Priority: Must
Origin: Customer
Milestone: M1

## User value
Understand the ingredients used in recipes, with trustworthy nutritional context.

## Acceptance criteria
1. Given an ingredient, its identity, aliases, edible form (raw/cooked), nutrient basis, source, version and uncertainty are retained.
2. Given known composition and quantities, energy, macros and micronutrients are calculated per recipe and per serving with explicit units and rounding.
3. Given missing nutrient values or cooking/yield uncertainty, the UI distinguishes unknown values from zero and explains the calculation limits.
4. Given beta-glucan in oats or soluble fibre in psyllium husks/chia, educational notes reference evidence, applicable quantities and limitations without promising health outcomes.

## Delivery contract (2026-11-07)
The customer authorized F02 implementation. The first slice provides a read-only ingredient
catalogue, a stateless calculation API and a browser nutrition workbench at `/`.
A reviewed bundled catalogue uses frozen USDA SR Legacy oats/chia records and an explicitly
unknown psyllium-husk composition. A local JSON catalogue may replace it at startup; it is
not a household database or a storage adapter. No remote lookup occurs during app use.

5. `GET /api/ingredients` supports name/alias search; detail lookup retains ingredient ID,
   form, per-100-g edible-portion basis, source record/version/licence and uncertainty.
6. `POST /api/nutrition/calculate` accepts ingredient IDs, positive decimal-string grams,
   positive integer servings and preparation context. Decimal inputs have up to six fractional
   digits and a maximum of 1,000,000,000; at most 100 lines and 1,000,000 servings are accepted.
   Quantities never pass through binary floating point. Results round half away from zero to
   two decimal places only after aggregation and portion scaling, with units and policy stated.
7. Missing/null values produce null recipe/portion totals plus explicitly labelled known
   subtotals and missing ingredient IDs; a measured zero remains a complete zero. No nutrient
   is inferred from another (in particular no beta-glucan estimate from total fibre).
8. Input-source uncertainty and preparation context accompany every result. Unmodelled cooking
   displays a warning: no retention, water/yield or cooking-loss correction has been applied.
9. Invalid IDs, unsupported units/fields, zero/negative/nonfinite/exponent/numeric JSON amounts,
   excessive input and malformed catalogue files fail explicitly; calculations do not mutate data.
10. The browser allows ingredient selection, gram quantities and serving count; prominent
    per-serving macros, a nutrient table, unknown/partial states, source links and evidence notes
    remain usable on narrow screens and with keyboard navigation. Errors preserve user input.

## Out of scope
Invented nutrient values or medical treatment recommendations. General unit conversion/preferences (F03),
recipe storage (F04/F18), household/authentication (F09), provider auto-sync and Flutter (F18).

## UX notes
See [ux-notes.md](ux-notes.md); the browser workbench is an end-to-end review surface for this slice.

## Links
Requirement: [F02](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md). Validation: [validation.md](validation.md).
