# Recipe quantities without a known mass

Status: Accepted
Date: 2026-11-28
Scope: F02/F03/F04/F06/F10/F12/F39

## Context

A recipe line in millilitres, spoons or pieces could only be saved with conversion evidence, a
household-declared unit weight or an accepted rough estimate (ADR 0038). The customer still found
entry demanding ("I do not want to specify a hundred things") and asked for the nutrition gaps to
be closed; an unresolved quantity that keeps the recipe saveable was a planned F39 criterion. ADR
0038 listed lines without mass as a rejected alternative because every consumer assumed grams.
ADR 0002 forbids implicit volume-to-mass conversion; unknown data must stay unknown.

## Alternatives

- Keep requiring a weight for every non-mass line.
- Fill a default weight silently so that the line has grams.
- Store a placeholder mass of zero.
- Make the mass optional and carry "unknown" explicitly through every consumer.

## Decision

A recipe line with a volume, spoon or piece quantity and no conversion has no `grams`. A stated
mass next to such a quantity, a zero amount and a missing quantity are still rejected. Every
consumer handles the unknown mass explicitly:

- Nutrition calculates the weighed lines only. Each nutrient names the unweighed ingredients in
  `missing_ingredients`, reports no total, per-serving value or range, keeps `known_*` for the
  known part and is `partial` or `unknown`. `warning_codes` gains `unweighed_ingredients`.
- Every nutrition contribution and display line carries `mass_basis`: `weighed`,
  `source_portion`, `household_weight`, `rough_estimate` or `unknown`. `rough_mass_estimate` and
  `household_mass` warnings make accepted estimates and household weights visible in nutrition.
- Scaling keeps the original unit and scales its amount.
- Shopping lists an unweighed line in its own unit: millilitres or pieces, with spoons as
  millilitres of the metric 5/15 ml spoon. This is a volume, never a derived mass.
- Meal-plan and inventory mass arithmetic leaves unweighed lines out; they are not zero.
- Cooking shows the original quantity for the chosen serving count.

## Consequences

Entry no longer blocks on weights; the price is visibly incomplete nutrition until the cook adds
one. Inventory coverage and plan mass deltas do not see unweighed lines. Shopping may show the
same ingredient in grams and in millilitres or pieces. Clients must treat `grams` as optional; the
Nextcloud and Flutter clients do. Saved revisions are unchanged: every earlier line has a mass.
Source-backed piece weights and a cooking yield/retention model remain open and need source data.
