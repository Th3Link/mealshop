# ADR 0014: Recipe-owned parts and optional constituents

Status: Accepted for the customer-authorized F26 slice

## Context

Recipes need preparation branches and explicit ingredient-to-step associations. Optional food
constituents must accumulate without being presented as universal nutrients. Existing calculations,
shopping provenance, immutable snapshots, household permissions and exact quantities must remain intact.

## Alternatives

Nested or independently versioned recipe references would need cross-record snapshot resolution,
cycle handling and yield/conversion semantics not specified in this request. Reusing ingredient
composition as a recipe structure would confuse membership with measured recipe mass. Treating every
possible constituent as a mandatory nutrient would falsely imply common availability or known absence.

## Decision

Use recipe-owned named parts with stable IDs and an acyclic prepared-part dependency graph. Keep one
flat list of measured ingredient lines, each belonging to at most one part. Steps reference stable line
IDs and prepared parts. Nutrition, quantities, shopping and exclusions continue to consume each flat
line once. Legacy flat records load through defaults; references are validated before atomic publication.

Declare optional constituents in the existing own nutrient profile with stable IDs, explicit g/mg/µg
units, exact decimal amounts, names/translations and the profile's evidence. Inheritance resolves the
whole profile into the pinned ingredient snapshot. Sum by stable identity with matching units; unknown
contributions remain unknown and known subtotals remain explicit. Legacy beta-glucan/soluble-fibre
keys remain readable for existing API/demo clients; the native client separates the constituent view.
No physiological benefit or chemical conversion is inferred.

Sorbitol uses a separate intolerance registry and explicit household declaration. It is not appended
to the regulated fourteen-allergen identity mapping. The existing household constraint review resolves
both current and pinned declarations, including composed ingredients. The existing free-text intolerance
field stores its canonical ID when selected; no diagnosis or tolerance threshold is inferred.

## Consequences

No new crate, storage adapter or host-specific route is needed. Source records stay immutable. Recipe
parts do not promise a parallel timer schedule, independent reuse or cooking-loss corrections. New
fields require matching-version backups for downgrade, while old records remain supported on upgrade.
