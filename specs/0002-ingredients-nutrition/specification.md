# Ingredient catalogue and nutrition

Status: Draft
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

## Out of scope
Invented nutrient values or medical treatment recommendations.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F02](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).
