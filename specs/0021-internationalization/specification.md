# Nextcloud internationalization and localized presentation

Status: Done
Priority: Must
Origin: Customer-approved Nextcloud scope separation, 2026-11-10
Milestone: M1

## User value
Use the shipped Nextcloud application in German or English without changing stored quantities or source facts.

## Acceptance criteria
1. Keyed de/en messages, explicit language choice, regional-tag normalization and English fallback work in the Nextcloud client.
2. Existing reviewed ingredient translations and aliases support discovery; unavailable recipe/source translations remain visibly original. No generated translation replaces source content.
3. Exact localized decimal input/output preserves six digits and valid values during locale switches; invalid inputs are never reinterpreted silently.
4. Dates, integer counts and serving plurals use the active locale independently of units. API identities, units and canonical decimal strings remain stable.
5. Error and warning presentation uses stable codes and focused UI/API checks; calculations and source provenance remain on the server.

## Scope decision
The customer explicitly approved closing the Nextcloud delivery and retaining all unfinished work in separate specifications.
[F29](../0029-cross-client-localization/specification.md) preserves the original six acceptance criteria verbatim, including
versioned content translations, API negotiation, complete parameterized backend errors and standalone/Flutter coverage.
Those requirements remain open; this Done status does not claim their implementation.

## Evidence
See [plan.md](plan.md), [F22 validation](../0022-product-web-clients/validation.md),
and the combined [shopping delivery validation](../0006-shopping-list/validation.md).

## UX
[ux-notes.md](ux-notes.md). Native host conventions take precedence.
