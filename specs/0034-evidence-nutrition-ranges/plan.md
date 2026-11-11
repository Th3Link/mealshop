# Delivery plan

- [x] Audit current source/model boundaries and original customer instructions.
- [x] Research and freeze comparative records; record matching rationale and coverage.
- [x] Add validated reported/supplemental/range modeling and exact aggregation regressions.
- [x] Integrate frozen/legacy recipe evidence, own/group ranges and recipe/ingredient rendering.
- [x] Verify real Nextcloud, languages/themes, accessibility, responsive layouts and restart.
- [x] Complete source verification, full gates and fixture cleanup.
- [x] Review private household ingredients with the customer's explicit in-memory read permission.

PR dependency: F33 / PR #28 is merged. PR #29 targets `main`; its merge commit is incorporated
without rewriting the shared branch history.

Evidence: 96 Rust tests (including exact outward rounding, public-core coverage, malformed
source/range rejection, explicit group inheritance, frozen legacy reads/new revision pinning),
19 Vue unit tests, strict build, reproducible 313-food/73-group coverage and original archive
comparison of 1,341 facts. Runtime/CI evidence is recorded after final installation below.

Privacy boundary: an automatic review rejected a proposed private-ingredient read plus full audit
copy. That operation was not executed. The customer subsequently authorized a read-only,
in-memory ingredient review, explicitly prohibiting audit copies and record logging. That review
is complete; its findings are excluded from published evidence. Public checks use the bundled
catalogue and synthetic fixtures. Unknown composition remains an explicit product state and must
not receive invented ranges. Existing authorized deployment backups remain in place; browser
fixtures use ledger-recorded synthetic households only.

## Installed acceptance evidence

- The full 21-test Playwright suite passed **against the installed container before the final inheritance correction**, including
  native recipe authoring, household roles/invitations, dietary blocks, group inheritance, conflicts
  and draft retention, shopping, timers, barcode/camera fallbacks and responsive/theme checks.
- F34 specifically verifies 580–597 kcal total / 290–298.5 kcal per serving for the synthetic
  100 g oats + 100 g psyllium fixture; independent per-100-g references, explicit own-range
  validation/persistence, English/German, light/dark and 1440/390/320 px. Scoped WCAG 2.2 AA axe
  scans found no violations. Screenshots were visually inspected. This is automated evidence,
  not a blanket accessibility certification.
- Rootless TCP and Unix-socket container lifecycle/auth/CSRF/private-storage checks passed.
- Initial installed image: `sha256:caecd0263b91e499fc7182d8a8a92d0ae80f6a249b0ba2f3b98e822506ae41da`.
  Public JS/CSS SHA-256 hashes match the local strict production build.
- A real container restart preserved the owned-fixture authenticated API snapshots, one referenced
  image and all 41 persisted files exactly. API snapshots excluded both original households;
  protected customer files were checked only for integrity.
- Ledger-owned synthetic households/notices were archived with full private deployment backups.
  Both initial households (`home-1`, `home-72`) and their eight files remain byte-identical.
- All 17 repository/client gates, strict rustdoc and direct original-archive verification passed;
  final pre-push rerun and PR-head CI are recorded in the pull request.

The implemented and installed slice and the authorized memory-only review are complete. F34 is
**Done** with explicit per-nutrient unknowns; Done does not claim universal nutrition completeness.
See [coverage and comparisons](coverage.md). PR #29 targets `main`; review and exact-head CI remain
visible on GitHub.

The importer merges complementary nutrient fields of identical reference records before deriving
group extrema. The independent verifier also reconstructs each group bound from all effective
members, rather than merely checking the retained extremal references. Browser checks explicitly
cover sweetener sugars 0–100 g and vinegar salt 0.045–0.208 g per 100 g.

Final review found that a subtype inheriting an explicitly replaced household group profile could
still borrow the group's original catalogue estimates for omitted nutrients. The focused regression
`inherited_household_profile_does_not_borrow_replaced_group_estimates` failed before the correction
on an unexpected fat range and passed afterward. Only reference-mode parents now pass comparison
profiles to subtypes; a household profile's omitted fields remain unknown. The regression verifies
recipe output, persisted inheritance and immutable recipe reads after restart. Final candidate image,
installed-browser rerun and exact-head CI evidence are recorded in PR #29.

## Acceptance trace

| Acceptance | Evidence |
|---|---|
| 1 | Offline 313-food/73-group core coverage, independent archive checks and the authorized aggregate-only private audit above |
| 2–3 | Frozen primary observations, importer/verifier checks, provenance/bound validation regressions and inspectable source facts |
| 4 | Checked exact aggregation and outward-rounding Rust regressions, serving-scale browser assertions |
| 5 | Installed EN/DE light/dark, 1440/390/320 px browser checks, scoped axe scans and inspected screenshots |
| 6 | Own-range/group inheritance tests, reader/stale-write checks, frozen revision and byte-identical legacy/restart checks |
| 7 | Full repository/client/container gates, 21 installed browser tests, cleanup/integrity proof and PR-head CI |
