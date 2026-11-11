# ADR 0019: Explicit supplemental nutrition and comparison ranges

Status: Accepted for F34 implementation; review with the PR.

## Context
The customer requests useful approximate composition for existing foods, yellow recipe indicators,
ranges and comparison sources. F02/F23 previously left unmatched profiles entirely unknown. The
primary catalogue is already close to its 1 MiB serialized-input limit. Recipes freeze primary inputs.

## Alternatives
Copying a nearby food into the original nutrient map loses uncertainty and identity. A universal
percentage lacks evidence. Dropping missing inputs undercounts totals. Rewriting old recipe files
breaks the append-only history. Provider calls at runtime add availability/privacy dependencies.

## Decision
Keep original readings intact. Add optional self-contained supplemental evidence to an ingredient,
with reviewed reference facts and bounds. Direct same-source supplements remain reported facts;
comparative/group envelopes and user-declared bounds remain estimates. Calculate bounds exactly,
round outward and distinguish complete ranges from covered subtotals. Never transfer dietary safety.
Use a frozen versioned offline registry, with immutable v1 fallback for legacy snapshots matched to
original stable identities/source state. New revisions pin the full effective evidence. Do not write
old persisted histories while reading them. Expand the bounded catalogue envelope only as measured
source payloads require; preserve the separate 2 MiB recipe-publication/restart validation boundary.

## Consequences
Comparison min/max reflects selected observed records, not population coverage or confidence.
Unsupported individual nutrients remain explicitly incomplete. Original source values, carbohydrate
semantics, quantities, household roles and homemade input accounting remain authoritative. Complete
private backups precede installation; old binaries may reject new optional persisted fields.

Method references: [FAO/INFOODS matching guidelines](https://www.fao.org/fileadmin/templates/food_composition/documents/Nutrition_assessment/INFOODSGuidelinesforFoodMatching_version_1_2.pdf),
[FAO missing composition data](https://www.fao.org/4/y4705e/y4705e14.htm),
[USDA data documentation](https://fdc.nal.usda.gov/data-documentation/).
