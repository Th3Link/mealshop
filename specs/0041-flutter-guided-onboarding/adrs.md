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

## D4 — Goal and weight data (decided: FL-GOALS C, 2026-11-28)

F35 kept weight and medical fields out pending an access and retention decision. Health data is a
special category under GDPR Art. 9 and needs explicit consent, minimization, export and erasure.
Decision: coarse goal plus optional weight and target weight in on-device and connected mode.
Consequence: the server stores health data, so the slice needs a cross-cutting ADR before code:
a private per-person record beside activities with the same explicit grants (no automatic owner
access), consent record, export, separate erasure, backup inclusion, exact decimal weight with
unit and date, and no derived calorie or health claim. Never part of sharing or federation.

## D5 — Starter recipes come from the recipe server (decided: FL-STARTER, 2026-11-28)

Calibration fetches a small starter set from the customer's online recipe server; the app bundles
none. The client contract (endpoint, authentication, recipe format, licence and attribution,
language, filtering by hard rules) is defined by that server's API and is a dependency of the
calibration slice. Fetching is a visible, consented network request; hard rules are applied on
the device, so no dietary profile is sent to the server.

## D6 — The main user is remembered per device (accepted with slice 2)

Context: the customer's main user is "whoever is typing on this device". Household profiles are
shared; in connected mode every phone has a different main user, and the profile schema rejects
unknown fields for older clients.

Alternatives: a `main_user` flag on the household member (wrong for several devices, needs a
schema change and a Vue round trip); a device-local reference to the person ID.

Decision: store the reference in the app's protected local state per household, together with the
assistant's started/done state. No core or schema change. It is cleared with the local session and
is not part of a backup; "This is me" in a person's summary sets it again. A person is still not
a login account and the marker grants nothing.

