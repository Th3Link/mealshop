# ADR 0015: shopping provenance, shop order and product boundaries

Status: Accepted for F06/F07/F08

## Context
The customer requests all three shopping features together, backend and native frontend in ordered
usable slices. Existing meal-plan shopping totals are read-only aggregates, without purchases or extras.

## Alternatives
Mutating aggregate quantities would lose origin and purchase history. Inferring ingredient equivalence
from names or packages would compromise source identity. Unconsented global learning/lookup exposes
household behaviour. Additional persistence crates are unnecessary for the accepted file-backed slice.

## Decision
Persist a revisioned, atomically replaced household shopping file under the existing exclusive household
lock. Separate current source requirements from immutable purchased receipts. Aggregate only identical
ingredient IDs/base dimensions; retain original input and pinned recipe/package provenance. Rescaling
or removing a source never reverses a purchase or alters another source's/manual contributions.

Maintain household-only shops, preferences, assignments and explicit manual order. Opt-in completed
sessions provide explainable integer position averages with a three-session minimum; corrections do not
train and manual order wins. Disabling/resetting learning preserves list requirements and purchases.

Keep barcode products separate from the nutrition catalogue. Validate GTIN check digits, preserve leading
zeros, and require explicit package dimensions and ingredient-match confirmation. Native browser scanning
is optional. Open Food Facts requests require per-request consent; a fixed provider, bounded responses,
timeouts, no redirects and request throttling prevent arbitrary outbound requests. Retain source/licence/
timestamps and original package text; import no nutrition or allergy claims.

## Consequences
Private storage and optimistic revisions extend existing auth/CSRF/role boundaries. Unknown or incompatible
units remain separate. Shopping is not inventory accounting. Camera support varies; manual entry remains
complete. External lookup availability is not guaranteed. UI tests use recorded synthetic households and
archive their fixtures afterward, preserving all customer data.

## Sources
[Open Food Facts API contract](https://openfoodfacts.github.io/openfoodfacts-server/api/),
[native barcode support](https://developer.mozilla.org/en-US/docs/Web/API/Barcode_Detection_API).
