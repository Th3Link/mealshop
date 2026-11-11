# Source-backed nutrition ranges

Status: In progress
Priority: Must
Milestone: M2
Origin: Customer request, following F33, 2026-11-11.

## Objective
Every existing food contributes meaningful available nutrition. Missing product-specific data may
use explicitly labelled comparative estimates with source-backed ranges, visible in yellow in recipes.
The customer explicitly authorizes comparative estimates; this extends F02/F23's earlier exact-only
policy without replacing the original primary records or inferring allergy safety.

## Acceptance
1. Audit all 313 bundled food identities and household ingredients. Provide source-backed energy,
   protein, fat and a correctly defined carbohydrate value for every bundled food; fill further
   supported missing nutrients using retained primary records/declarations. Keep unavailable individual
   nutrients explicit, never invent values or treat missing data as zero. Unbounded custom inputs
   receive a separate incomplete-nutrition warning; their private source audit awaits permission.
2. Separate original reported values, same-source supplemental values, comparative ranges and household
   declared ranges. Retain food/state, per-100-g basis, original decimal facts, publisher, record/version,
   retrieval, URL, licence status, selection rationale and method. Existing source facts win.
3. Comparison envelopes use the observed minimum/maximum of reviewed compatible records; they are
   comparison spans, not guaranteed bounds or statistical confidence intervals. No arbitrary percentage,
   raw/cooked substitution, implicit density, carbohydrate-definition conversion or constituent inference.
4. Calculate complete recipe and portion ranges with checked exact integers, preserving unrounded
   inputs through serving changes and outward rounding at display. A wholly unbounded input makes a
   nutrient incomplete; show a labelled covered subtotal instead of claiming a complete range.
5. Mark affected recipe ingredients, ingredient details and affected nutrient rows in native warning
   yellow with text/icons. Show the ranges and accessible, inspectable comparison sources in English
   and German, system light/dark and narrow layouts. Preserve F33's compact always-open recipe sections.
6. Household editors can explicitly enter nutrient ranges with evidence, including inherited group
   values; readers cannot write. Organizational group ranges use their explicit members only. New
   recipe revisions pin effective evidence. Legacy recipes gain a versioned supplemental analysis while
   their persisted originals/history stay byte-identical; legacy fallback v1 remains immutable.
7. Dietary constraints, ingredient identity, shopping masses and homemade input-only accounting remain
   unchanged. Verify old/new records, scaling, malformed bounds/provenance, stale/reader writes,
   restart and real embedded Nextcloud behavior. Audit coverage reproducibly offline, retain backups,
   archive only owned fixtures, run all repository/client/container gates and check CI on the PR head.

## Implementation boundaries
Rust/Axum remains the calculation authority; Vue 3/strict TypeScript uses native Nextcloud controls
and keyed messages. No new dependency, provider call during app use or invented endpoint. Source
facts and estimation data are separate files. Sources without an explicit reuse licence retain that
unknown status; record factual nutrition declarations only, not whole copyrighted pages.

Build/check: `cargo nextest run --workspace --locked`; `cargo clippy --workspace --all-targets --locked -- -D warnings`;
`npm --prefix clients/nextcloud run build`; `npm --prefix clients/nextcloud test`; the full AGENTS.md gate.
Use a new offline estimation verifier plus existing import/catalogue checks. See plan.md and ux-notes.md.
