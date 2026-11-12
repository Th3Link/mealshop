# Product requirements

Origin: customer vision, 2026-11-07. Mealshop is a working name.
The [feature index](../specs/README.md) maps F01–F32 one-to-one to acceptance contracts.

## Cross-cutting requirements
- N01: Rust/Axum/Tokio backend and Flutter client; standalone and free Nextcloud `ExApp` deployment.
- N02: Selectable Git/file, PostgreSQL, SQLite or MariaDB persistence, with portable versioned data.
- N03: Precise quantities, configurable presentation, reproducible nutrition and explicit unknowns.
- N04: Household isolation, dietary exclusions above recommendations, consented calendar/learning use.
- N05: AGPL-3.0-only complete software; paid official app, self-build allowed, free/paid hosted tiers.
- N06: Imported recipes share their source URL only, including via collections, MCP and federation.
- N07: Multi-day processes and reminders survive restart and explain delivery limitations.
- N08: Spec-driven development, ADRs and CI with Renovate, builds, docs/Pages, audit, SBOM, metrics and MSRV.

- N09: English is the coded/canonical base with stable language-independent IDs and separate translations. German/English i18n from the outset: UI, content names/search, exact locale-aware quantity input and number/date/plural formatting; preserve source language when translations are absent.
- N10: Vue 3 product web clients for Nextcloud and standalone hosting. The existing framework-free web surface is a pragmatic demo only; significant UX/UI work belongs in the product workflow.

## Scope and unresolved decisions
M0 delivered the bootstrap. F02 adds a read-only, versioned nutrition starter catalogue and
stateless demo accepted in PR #4. Its USDA snapshots are CC0; long-term provider coverage/refresh, identity/authentication,
calendar provider, database migration/sync details, payment provider, attestation feasibility,
notification transport, federation protocol and concrete paid tiers remain open. No customer
price, health guarantee or provider commitment is inferred from the vision.

## Traceability
N01 → F01/F18; N02 → F18; N03 → F02/F03/F04; N04 → F09/F10/F11;
N05 → F19; N06 → F05/F16/F17/F20; N07 → F12/F13/F14; N08 → F01.
Each feature plan must link its tests to its numbered acceptance criteria before Done.

N09 → F21 (all client features); N10 → F22/F18. F02 is Done following customer merge of PR #4.

F23 expands the kitchen reference catalogue from the customer ingredient list with exact BLS/USDA
source mappings; unresolved product profiles remain explicitly unknown.

F23 is accepted through PR #5. F04/F18 now starts local recipe authoring and immutable file
revisions with pinned nutrition inputs; authenticated ownership, scaling and other adapters
remain future work. See [recipe guide](guides/recipes.md).

F25 adds household ingredient groups/subtypes, exact declared nutrients with explicit inheritance,
and separate dietary/origin declarations; see its acceptance contract. N03 → F25, N04 → F25.

F26 refines native authoring, recipe-owned preparation parts, optional constituents and explicit
sorbitol intolerance declarations. N03/N04/N09/N10 → F26. Full catalogue grouping follows separately.

F27 organizes every bundled ingredient through a complete exact-ID mapping, after F26 verification.
Groups provide discovery and optional explicit household declarations, without shared source nutrition.

F28 reviews all shipped native views and improves draft protection, profile editing, discovery,
error recovery and review focus. N03/N04/N09/N10 → F28.

F06/F07/F08 deliver protected shopping provenance, optional household shop ordering and manual/native
barcode product capture. N03/N04/N09/N10 apply. F21/F22 close only the delivered Nextcloud scope
by explicit customer decision. F29/F30 retain the complete original cross-client acceptance contracts
unchanged; standalone/Flutter localization, versioned content translations and standalone shell remain open.
N09 → F21/F29; N10 → F22/F30/F18.

F12/F13 deliver native Nextcloud guided cooking and reviewed multi-day baking with durable UTC deadlines,
IANA/DST resolution and personal `AppAPI` reminders. N03/N04/N07/N09/N10 apply; accepted delivery is not
guaranteed device delivery. See [cooking guide](guides/cooking.md) and ADR 0016.

F31 records the customer-approved unified Nextcloud meal/preparation calendar, mealprep allocations, multipart authoring and typed ingredient-line extensions for future device consumption. N03/N07/N09/N10 apply; see specs/0031-unified-preparation. Implementation status is tracked there.

F32 refines household shopping discovery, item tiles, exact quantity entry and check-off/undo within
the native Nextcloud shell, following the supplied Bring interaction reference. N03/N04/N09/N10 apply.

F33 refines recipe reading/editing against the approved screens: compact always-open composition,
optional plus actions and explicit homemade outputs with measured base inputs. N03/N04/N07/N09/N10
apply. See [F33](../specs/0033-compact-recipe-preparation/specification.md) and ADR 0018.

### F34 — Source-backed nutrition ranges
The customer authorizes comparative estimates for missing composition, separate from original
source facts. Freeze selected reference observations and matching rationale, distinguish reported
supplements from estimated/group/declared ranges, aggregate exact bounds and show native warning
indicators with complete/partial totals and provenance. Do not infer constituents, allergy safety,
density, nutrient definitions or population confidence. The customer explicitly authorized private
household ingredient review in memory only, without household audit copies or record logging.
See [F34](../specs/0034-evidence-nutrition-ranges/specification.md) and ADR 0019.


## F05 Nextcloud delivery boundary
Recipes remain household-owned. Private collections preserve ordered recipe/revision references;
source owners manage named-account grants, addressed invitations to household owners and direct
grants to another household they own. Current recipient membership gates every shared read.
Imported recipes share only their retained original HTTPS source URL, including historical revisions
and collection entries. Revocation prevents later server access, without claiming to remove copies.
See [F05](../specs/0005-sharing-collections/specification.md) and
[the sharing guide](guides/sharing.md). Payments, public links, federation and other clients remain
outside this Nextcloud delivery.

## F11 Nextcloud delivery boundary
Explicit manual or verified completed-session history, household-reported ratings, owner-managed
local learning epochs and explained whole-pool suggestions reuse existing household authorization
and dietary/time/quantity checks. Reset removes learned state without silently replaying retained
ratings. Model changes have a versioned synthetic baseline comparison; real-world effectiveness
is not inferred from offline proxies. See [F11](../specs/0011-recommendation-feedback/specification.md).

F17 delivers disabled-by-default per-account MCP grants within current Nextcloud household roles,
source-only imported projections, canonical nonpersistent recipe previews and atomically audited,
idempotent confirmed creation. The supplied stdio bridge uses existing Nextcloud app passwords;
hosted OAuth and per-client delegated credentials are outside this authentication slice.
N03/N04/N06/N09/N10 apply. See [MCP guide](guides/mcp.md) and ADR 0022.
