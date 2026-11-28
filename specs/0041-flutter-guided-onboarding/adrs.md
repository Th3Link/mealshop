# Decisions

## D1 — One implicit household in on-device mode (accepted, customer requirement 2026-11-28)

Context: the shared Rust core is household-scoped; backups, grants and every API path carry a
household ID. The customer wants no household concept in standalone use.

Alternatives: remove households from the core for app-only (large, breaks backup format and the
shared contract); keep the visible household flow (rejected by the customer); create one household
automatically and hide it.

Decision: keep the core and backup format unchanged. The app creates one household with a fixed
internal English name when an on-device dataset has none, selects it and never shows the name.
The name is not localized and not user data. Legacy datasets with several households keep a
switcher so that no saved data becomes unreachable.

Consequences: no migration, no new crate, no core change. Connected mode is untouched. A later
local-to-server migration can still map the internal household to a real one.

## D2 — Taste calibration is declared preference, not learned history (proposed)

Context: F11 learns integer preference sums from completed cooking only, owner opt-in, in epochs
(ADR 0021). A new install has no history; the customer wants a ranked start.

Proposal: store the calibration result per person as an explicit, versioned "declared at setup"
preference set (liked/disliked dish attributes with small bounded integer weights and the ranked
items). Ranking uses it as a prior next to, never instead of, observed history; hard constraints
filter first. Recalibration replaces the set. No new learning framework. Needs a compatible
versioned profile extension in the Rust core and its own ADR in `docs/adr/` when accepted.

## D3 — Consent records (proposed)

Proposal: a device-local consent record per purpose (`public_recipe_sharing`, `health_data`)
with decision, timestamp, wording version and language; revocation is a new record, not an edit.
Included in backups. Transmitted nowhere. The exchange session defines what a later upload needs
(identity, licence, access control); this record alone never authorizes an upload of imported or
photographed content.

## D4 — Goal and weight data (open, FL-GOALS)

F35 kept weight and medical fields out pending an access and retention decision. Health data is a
special category under GDPR Art. 9 and needs explicit consent, minimization, export and erasure.
Proposal: coarse goal for everyone who wants it; weight and target weight only in on-device mode,
never synced or shared, erasable separately, included in the encrypted backup.
