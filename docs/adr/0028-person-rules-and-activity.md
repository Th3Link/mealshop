# ADR 0028: Effective dietary rules and person-scoped activity access

Status: Accepted for F35 P2
Date: 2026-11-13

## Context and customer decisions

P2 delivers the person tour, recipe/ingredient library, collections/sharing and optional activity in
one Android/backend PR. Household people and Nextcloud accounts are different identities. Existing
household dietary constraints remain shared under the existing roles.

The customer explicitly selected person-specific activity permissions, including care without a child
account. Household owners administer those permissions but receive **no automatic activity read or
write access**. An owner must explicitly grant their own account access. No new diagnosis, weight or
medical-context fields are approved. Existing allergy/intolerance declarations remain hard constraints.

## Alternatives

- Household-wide activities: rejected by the customer.
- Private activities per login account: rejected in favour of person-specific rights.
- A second independent profile/activity file: complicates atomic deletion and revocation.
- A server-owned private section in the existing atomic profile document, omitted from public profile
  responses and planning snapshots.

## Decision

Keep the public profile response compatible and extend each member with optional versioned dietary
rules. Missing rules remain legacy behaviour. Explicit presets are unrestricted, vegetarian and vegan;
exceptions are exact ingredient IDs with an optional explicit household-produced origin condition.
An exception relaxes only its preset, never an allergy, excluded ingredient/tag or intolerance. Unknown
classification/provenance remains review-required. Mixed preset/exception profiles are labelled with
their exceptions rather than unqualified vegan claims. Use current and frozen ingredient evidence
conservatively. Canonical evaluation and every existing plan preview/apply/suggestion use these rules.
Ingredient production origin is an explicit household declaration, separate from nutrient sources.

Optional kitchen settings provide maximum active preparation time and recurring weekday/meal-slot
availability; they are enforced by existing plan review. Full-week generation, physical mealprep and
native plan editing remain P3. No clinical goal or invented energy target is stored.

Store activity/access state in a private, bounded section of the same profile document. Public GET/PUT
profiles and profile exports contain no activity entries or grants. Atomic replacement preserves that
private section; removing a person removes their entries and grants in the same publication. Household
editors cannot remove a person with protected activity state; the owner must perform that removal.
Legacy clients cannot erase saved new rules by omitting their optional field.

Expose explicitly new household-person activity and access routes. Access management requires current
household ownership; read/edit require an explicit current per-person grant, including for an owner.
An edit grant includes read/delete/export. A household reader may receive an activity edit grant
without obtaining permission to change dietary profiles, recipes or other household data. The domain
proxy exception is restricted to those activity routes and is checked again by the activity handler.
Membership removal atomically increments a retained per-account membership epoch. Activity grants
bind that epoch, so a later invitation does not resurrect access. No cross-file revoke transaction is
needed; public membership DTOs remain unchanged. Current membership is required on every read/write. Stable person generations
and expected revisions reject stale forms after person deletion/recreation or access changes.

Activity records contain an exact calendar date, kind, integer duration and optional self-reported
intensity/notes. No weight, calories or medical inference. Entries remain until authorized deletion
or person deletion; the API supports single deletion, full deletion and protected export. Backups and
previously exported copies remain outside live deletion. Use bounded record counts/bytes and no history
of deleted records. Non-shopping writes require online authority checks; no new replay queue is assumed.

Android tour drafts contain only the edited person, public revision and tour step, with explicit comparison after a stale revision. The visible generic Person N label permits skipping personal name capture while retaining a required display label. They use the existing encrypted record, scoped by server/account/household/person with
bounded retention. Canonical activity and shared/imported content are not persisted as offline caches.
Shared imports render the server's source-link-only projection, including collections and images.
Use existing raster upload contracts through the official [`image_picker` 1.2.4](https://pub.dev/packages/image_picker) from the Flutter publisher; gallery cancellation
preserves drafts. Recipe composition stays always open with parts, line/step links and explicit homemade
outputs; no independently nested saved-recipe contract is invented.

## Consequences

Private activity does not leak through profile export, planning hashes or reader roles. Owners can
change access administratively, including granting themselves access; the UI makes that explicit.
Existing full-document profile writes must preserve private state and new hard rules. An older binary
cannot read a document once private data/new rules have been persisted; recovery requires a matching
complete backup or a reviewed forward migration, not stripping private data.

Use the existing Rust file-lock/fsync/rename mechanisms and existing Flutter state/theme/network/vault.
No new Rust crate, generic synchronization framework or medical subsystem. Optional `expected_count` on recipe creation provides an atomic online compare-and-save guard against duplicate creation after a lost response; it is not an offline replay receipt. Existing unguarded callers remain compatible. Android instrumentation uses a separate verification application ID because the Flutter driver uninstalls its target; the installed development app and its data must survive testing. P2 adds Recipes to active
Shopping/More navigation, with people, ingredients, collections, sharing and activities reachable by
clear labelled entries. Actual test/upgrade/privacy evidence belongs in the F35 verification document.
