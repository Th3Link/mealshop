# Acceptance evidence

Environment: existing Nextcloud 35.0.2 with `AppAPI` 35.0.0 at the customer-authorized local
address. Existing manual-install registration, secret and private data directory were retained.
The test image runs as UID/GID 1000 with a read-only root filesystem. No data reset or AGENTS.md edit.

| Acceptance | Evidence |
| --- | --- |
| AC 1: groups and exact identities | Existing wheat/rye/spelt exact IDs are grouped; `ingredient-groups.spec.ts` exercises creating and finding a group and subtype. Offline catalogue import/verifier still passes with 313 unchanged primary/identity records. |
| AC 2: exact declared nutrients and permissions | `ingredient_profiles_reject_invalid_inputs_and_preserve_primary_sources` checks reader denial, exact 1.250000, explicit zero, absent unknowns and retained original source. Negative amounts and unsupported source URLs fail. |
| AC 3: explicit inheritance and pinned revisions | `subtype_nutrition_is_explicit_pinned_and_persistent` calculates 21 g protein from an explicitly inherited 10.5 g/100 g profile and 200 g recipe; changing the group to 20 g changes the live subtype but leaves the saved recipe unchanged across restart. |
| AC 4: dietary/origin declarations | Backend enum validation and the browser workflow exercise vegan/plant declarations; the detail shows both independently. Existing hard-exclusion/composition/planning tests still pass. |
| AC 5: native browsing and editing | Browser workflow checks native editor controls, own/inherit modes, reload, mobile bounds and automated WCAG 2/2.1/2.2 A/AA tags. Existing focus, invitation, localized quantity, light/dark and reduced-motion workflows remain in the full suite. |
| AC 6: hierarchy, revisions and persistence | Invalid parents, self references and group/subtype cycles are rejected; duplicate nutrient keys fail. Existing concurrent ingredient/plan and stale-writer tests still pass. Legacy favourite updates preserve organization. Real authenticated restart/API/file checks are recorded below. |

## Executed checks

- `cargo clippy --workspace --all-targets --locked -- -D warnings` and
  `cargo nextest run --workspace --locked`: 52 tests passed. A deliberately inverted parent
  validation condition was detected by the subtype calculation/persistence test; original code restored.
- Client strict checking, production build, formatting and all five unit tests passed.
  Production output is approximately 358 kB gzip JavaScript and 18.3 kB gzip CSS, with no new packages.
- Full real embedded Nextcloud browser suite passed all seven workflows together in approximately
  1.8 minutes. The group form had no automated WCAG-tag violations; screenshots at 390 and 1440 px
  were inspected. No application console errors occurred in the group workflow.
- Rootless TCP and Unix-socket container smoke tests passed lifecycle, assets, auth, CSRF and private storage.
- Real container restart preserved canonical authenticated APIs and SHA-256 hashes of 141 persisted
  files across 23 memberships; eleven referenced images retained exact bytes and private response headers.
- Offline import/catalogue, reference-conversion and allergen verifiers, demo i18n, Rust format,
  licence and `RustSec` gates passed. Client audit reported zero vulnerabilities.

## Final review

The final archive-parent workflow passed separately in 18.8 seconds: archiving a group leaves
active subtypes visible and preserves their selected archived parent in the editor. Core implementation commit 723af6f passed all ten PR checks, including Rust 1.85 image
build/transport verification, coverage, both build profiles, client types/build/audit, MSRV, licences,
advisories, SBOM and binary metrics. Current-head checks are tracked in
[PR #19](https://github.com/Th3Link/mealshop/pull/19). Browser evidence is scoped to tested flows and sizes; it does not claim formal WCAG
certification. Classifications are household statements, not inferred safety, and no nutrient value
is inferred from a neighbouring ingredient or converted implicitly between sodium and salt.
