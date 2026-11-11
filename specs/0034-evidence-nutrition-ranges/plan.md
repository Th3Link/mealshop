# Delivery plan

- [x] Audit current source/model boundaries and original customer instructions.
- [x] Research and freeze comparative records; record matching rationale and coverage.
- [x] Add validated reported/supplemental/range modeling and exact aggregation regressions.
- [x] Integrate frozen/legacy recipe evidence, own/group ranges and recipe/ingredient rendering.
- [x] Verify real Nextcloud, languages/themes, accessibility, responsive layouts and restart.
- [x] Complete source verification, full gates and fixture cleanup.
- [ ] Review private household ingredients after the requested explicit read permission.

PR dependency: F33 / PR #28 remains open; this slice branches from its verified UI commit.

Evidence so far: 95 Rust tests (including exact outward rounding, public-core coverage, malformed
source/range rejection, explicit group inheritance, frozen legacy reads/new revision pinning),
19 Vue unit tests, strict build, reproducible 313-food/73-group coverage and original archive
comparison of 1,341 facts. Runtime/CI evidence is recorded after final installation below.

Privacy boundary: an automatic review rejected a proposed private-ingredient read plus full audit
copy. It was not executed or retried. The customer's separate explicit question for a read-only,
in-memory comparison is pending. No private household source review, audit export or guessed
name-based automatic mapping has been performed. Existing authorized private deployment backups
remain in place; fixture tests are limited to owned ledger-recorded households.

## Installed acceptance evidence

- The full 21-test Playwright suite passed **against the installed final container**, including
  native recipe authoring, household roles/invitations, dietary blocks, group inheritance, conflicts
  and draft retention, shopping, timers, barcode/camera fallbacks and responsive/theme checks.
- F34 specifically verifies 580–597 kcal total / 290–298.5 kcal per serving for the synthetic
  100 g oats + 100 g psyllium fixture; independent per-100-g references, explicit own-range
  validation/persistence, English/German, light/dark and 1440/390/320 px. Scoped WCAG 2.2 AA axe
  scans found no violations. Screenshots were visually inspected. This is automated evidence,
  not a blanket accessibility certification.
- Rootless TCP and Unix-socket container lifecycle/auth/CSRF/private-storage checks passed.
- Installed image: `sha256:caecd0263b91e499fc7182d8a8a92d0ae80f6a249b0ba2f3b98e822506ae41da`.
  Public JS/CSS SHA-256 hashes match the local strict production build.
- A real container restart preserved the owned-fixture authenticated API snapshots, one referenced
  image and all 41 persisted files exactly. API snapshots excluded both original households;
  protected customer files were checked only for integrity.
- Ledger-owned synthetic households/notices were archived with full private deployment backups.
  Both initial households (`home-1`, `home-72`) and their eight files remain byte-identical.
- All 17 repository/client gates, strict rustdoc and direct original-archive verification passed;
  final pre-push rerun and PR-head CI are recorded in the pull request.

The technical public-catalogue slice is complete and installed. F34 remains **In progress** because
acceptance item 1 also names private household source review. Automatic approval rejected the planned
read/export; the requested read-only/in-memory permission remains unanswered. The rejected audit
copy does not exist. No private source audit, silent name-based mapping or invented private values
has been substituted. Unsupported core contributions have explicit incomplete-nutrition warnings.
See [public coverage and comparisons](coverage.md). PR publication is a separate dependent change
on top of F33 / PR #28; merge/review and CI state remain visible on GitHub.

The importer merges complementary nutrient fields of identical reference records before deriving
group extrema. The independent verifier also reconstructs each group bound from all effective
members, rather than merely checking the retained extremal references. Browser checks explicitly
cover sweetener sugars 0–100 g and vinegar salt 0.045–0.208 g per 100 g.
