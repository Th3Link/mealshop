# Precise quantities and configurable units

Status: In progress
Priority: Must
Origin: Customer
Milestone: M1

## User value
Enter recipes naturally while viewing quantities in personally useful units.

## Acceptance criteria
1. Given a recipe input in g, kg, ml, l, tsp, tbsp or pieces, the original value and unit are retained with a typed dimension and exact decimal representation.
2. Given a user allowing only g and kg, recipe quantities display in those units when conversion is known; unavailable conversion is visible and requests ingredient-specific data.
3. Given a volume-to-mass conversion, it uses a sourced ingredient/state-specific density; piece conversions use a declared piece mass and spoon measures use a named regional standard.
4. Given scaling by servings, calculations preserve precision until presentation and record estimated conversions; mass never silently equals volume.
5. Given users with different unit preferences, one user’s display configuration does not rewrite another user’s recipe input.

## First slice
A typed quantity conversion endpoint retains the submitted decimal string and unit, identifies
mass/volume/count, and converts exactly between g/kg and ml/l. Piece identity is retained.
Cross-dimension conversion requires ingredient data; spoon conversions require an explicit
standard. Results beyond six fractional digits or supported bounds are rejected, never rounded.
Recipe lines now accept quantity plus optional conversion evidence, persist original input and normalized grams, and verify both on reload. Legacy gram lines remain supported. The demo supports per-line units and local g/kg display preferences. Evidence is user-declared with ingredient/state, HTTPS reference, version, uncertainty, and density or piece mass; spoon input additionally requires a named standard and declared spoon volume. Values are not independently verified provider data.

## Out of scope
A universal density or undocumented spoon/piece constants.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F03](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).
