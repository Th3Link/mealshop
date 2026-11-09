# Ingredient organization and nutrient declarations

Status: Accepted

## Context

The customer needs ingredient groups, subtypes, editable nutrient values and dietary classification.
Existing catalogue records have exact source identities, while recipe revisions pin their effective inputs.
The household overlay already provides private storage, writer checks and optimistic revisions.

## Alternatives

- Rewrite the curated catalogue or reuse a nearby flour's nutrient values: breaks source identity.
- Treat groups as composition: implies food membership instead of organizational hierarchy.
- Automatically inherit every value and dietary label: conceals uncertainty and explicit user choices.
- Add a separate persistence adapter or framework: unnecessary for this slice.

## Decision

Extend the existing household overlay with one-level organization and explicit reference/own/inherit
nutrition modes. Preserve primary sources separately when applying household overrides. Pin both the
parent and effective values in recipe revisions. No new crates or API routes are required.

Dietary class and origin are separate explicit enums: vegetarian food may still have animal origin.
These declarations do not clear allergies or override hard dietary constraints. Only nutrient values
inherit; classifications remain explicit per ingredient. Unknown values stay unknown, including the
retrieval date of newly declared nutrient profiles. Salt and sodium remain independent quantities.

## Consequences

Primary catalogue files remain untouched, existing flour identities remain searchable, and historical
recipes do not change after group edits. Old overlays load through defaults; writes retain their current
revision and private atomic-storage protocol. Older application versions require a matching pre-upgrade
backup to read data written with organization fields. Arbitrarily nested organizational trees are outside
this accepted one-level slice.
