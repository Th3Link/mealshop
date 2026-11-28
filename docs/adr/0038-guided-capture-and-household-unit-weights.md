# Guided capture and household-declared unit weights

Status: Accepted; amended by customer decisions of 2026-11-28
Date: 2026-11-27
Scope: F03/F04/F11/F16/F39

## Context

Recipe entry required a catalogue identity, an amount and, for every spoon, millilitre or piece,
conversion evidence with source URL, version and uncertainty. Only 23 of 313 bundled ingredients
have reference portions, so ordinary recipes demanded evidence forms per line. The customer asked
for guided entry without "a hundred things", ratings on recipes and clearer nutrition, and asked
which AI inputs are feasible. ADR 0002 forbids implicit volume-to-mass conversion and requires
provenance; ADR 0023 keeps imports reviewed.

## Alternatives

- Bundle generic default weights (e.g. one egg, one tablespoon) and apply them silently.
- Allow lines without mass and carry unknown grams through nutrition, shopping and inventory.
- Keep the evidence form and only restyle it.
- Let the cook state what one unit weighs as an explicit household declaration.

## Decision

Conversion evidence may omit its source URL only when it names its input unit: that is a
household declaration of the weight of one such unit. The client records the declaration date as
version and a fixed "not a source record" uncertainty, and labels the line. Sourced evidence still
requires HTTPS. A bundled reference portion is attached automatically only when exactly one fits
the exact ingredient record and unit. Nothing is inferred from density or a similar food.

Text, speech and pictures are captured into the existing editor by a deterministic client-side
reader; identities are matched only on one exact name or alias. Ratings remain history facts and
are summarized per recipe without learned state. The nutrition overview rounds for reading only.

Amendment 2026-11-28 (customer decisions): the cook may accept one general estimate, water
density for volume units and metric spoons, which is offered explicitly, stored with the version
prefix `household estimate` and labelled as a rough estimate. This is an explicit, visible
conversion and does not relax ADR 0002's ban on implicit ones. Assistants and customer-operated
tools hand recipes in as unreviewed suggestions (`recipe_suggest`, per-household inbox) that
become recipes only through the existing review; a suggestion naming a source URL is an import.
AI structuring inside Mealshop will go through Nextcloud Task Processing and is not implemented.

## Consequences

Recipes with spoons and pieces can be saved with one extra number per such line. Nutrition and
shopping masses derived from a household weight are as good as that number; this is visible on
the line but not yet propagated as a nutrition warning. Lines still need mass, so truly unknown
quantities remain unsaveable until a later slice. The MCP recipe schema accepts the same
declaration. Browser dictation may send audio to the browser vendor; the UI says so.
