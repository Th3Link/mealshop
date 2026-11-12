# MCP recipe tools

Status: In progress
Priority: Should
Origin: Customer
Milestone: M3

## User value
Manage recipes through authorized assistant tooling.

## Acceptance criteria
1. Given an authenticated MCP client, scoped tools search/read authorized recipes and preview insertion of a structured recipe.
2. Given recipe creation, validation and imported-content sharing restrictions match the regular API and the action is auditable.
3. Given a household boundary or read-only scope, tools cannot expose another household or mutate data.
4. Given repeated insertion with an idempotency key, the server avoids duplicate recipes and returns a structured error for invalid quantities.

## Out of scope
Unauthenticated access or treating imported recipe text as tool instructions.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F17](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).

## Nextcloud delivery contract
The customer requests F17 after merging F11. Under the standing autonomous-work instruction this
contract moves through Ready into In progress before implementation. No cloud AI provider, OAuth
server, account system or mandatory SDK/runtime dependency is introduced.

- Reuse the authenticated Nextcloud `AppAPI` proxy and current household membership on every request.
  A household owner explicitly grants each existing account Disabled, Read or Read/create MCP scope;
  default is Disabled. The effective capability is the intersection of that grant and current role.
  Readers never create recipes. Changing/removing access invalidates subsequent calls and previews.
  A grant conveys no access to another household, profiles, plans, shopping or learning data.
- Offer a bounded stateless JSON-RPC MCP endpoint under the existing household domain. Support current
  MCP 2026-07-28 metadata/discovery and legacy 2025-11-25 initialization for existing clients. Use JSON
  responses; no SSE subscriptions, sampling, remote fetches or notifications are advertised. Reject
  browser Origin headers on the RPC transport; native setup uses ordinary protected REST instead.
  Preserve the existing write marker, JSON limits, enabled-state checks and no-store responses.
- Supply a stdio bridge using Python's standard library and environment-provided Nextcloud username,
  app password and exact MCP URL. Credentials never enter JSON-RPC results, command arguments, logs
  or browser storage. Require HTTPS except explicitly local loopback development; reject redirects.
  App passwords retain their underlying Nextcloud account authority; they are not household-limited
  OAuth tokens. For an untrusted client use a dedicated least-privilege account. Hosted OAuth and
  independently delegated per-client credentials remain outside this existing Nextcloud slice.
- Discover/search ingredients by stable IDs and localized aliases; search/read owned recipes in the
  selected household with bounded pages and pinned revisions. Imported recipes, including older
  revisions subsequently marked imported, expose only their retained original HTTPS source URL.
  Do not search imported title/body/nutrition or export their text, ingredients, steps or images.
  Content is untrusted data and never changes the tool allowlist, identity or operation semantics.
- Preview a structured recipe through canonical recipe, ingredient, exact quantity, composition,
  provenance and image validation. Preview is nonpersistent and bound to actor, current scope and
  ingredient revision, expires after ten minutes, and cannot create a recipe. Imported preview
  results remain source-only. No allergy safety, density or nutritional value is inferred.
- Creation requires the actor's still-valid preview, explicit client confirmation and an opaque
  idempotency key. Confirmation is a client attestation, not proof of physical human review; hosts
  must allow users to deny writes. Invalid quantities return structured tool errors. An old preview
  after changed ingredients requires a new preview. Unknown fields cannot select another household.
- Persist creation actor/time, preview identity and idempotency receipt atomically inside the first
  immutable recipe revision. The canonical append-only recipe transaction is the sole publication
  point. Same actor/key/preview returns the original reference across retries, concurrency and restart;
  the same key with another preview is a conflict. Do not use a separate journal that can drift from
  recipe publication. Later edits retain the receipt in revision one and current imported restrictions.
- Native setup shows the current scope, exact proxy URL, credential-free connection example and
  household-visible creation audit with pinned recipe links. Only the owner edits grants; readers
  see their scope and activity. Explain export/confirmation and unknowns before connection.

## Verification
Trace the four acceptance criteria to protocol, canonical validation, current-role/scope, source-only
historical reads, concurrent/restarted idempotency and malformed input regressions. Exercise a real MCP
client plus the stdio bridge, real Nextcloud account authentication, native EN/DE themes/responsiveness,
all required repository/client/container gates and exact-head PR CI. Live checks use only new ledger-
recorded synthetic households; do not inspect or copy retained household directories.
