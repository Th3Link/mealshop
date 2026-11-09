# Acceptance evidence

Environment: existing embedded Nextcloud 35.0.2 / `AppAPI` 35.0.0 test instance. No reset, new
endpoint, dependency, source-data rewrite, permission relaxation or AGENTS.md change.

| View / workflow | Review and acceptance evidence |
| --- | --- |
| Household list, creation and invitations | Live overview/switcher review; existing `nextcloud.spec.ts` covers creation, reader invitation/acceptance, role revocation and isolation. Last-household behaviour remains covered by `daily-workflows.spec.ts`. |
| Recipe list and details | Live native review; `usability-review.spec.ts` searches titles, clears empty results, retains the query on return and verifies heading focus after asynchronous detail loading. Existing tests cover exact portion previews and g/kg display. |
| Recipe creation/editing | Live editor review; existing authoring/daily/quantity tests cover direct ingredient creation, parts, steps, images, provenance, source quantities, dirty navigation, conflicts and reload. Native fields are disabled while saving. Cancel is available at the sticky action area as well as the top. |
| Ingredient list/details | Live grouped browsing/search/detail review; usability regression resets all filters and verifies localized nutrient labels. Existing catalogue/group tests cover subtype discovery, exact identities, inheritance, favourites, archive/restore and recipe reverse lookup. |
| Ingredient dialog | Existing authoring/group/daily tests exercise creation, optional values/constituents, dietary classification, composition, sorbitol and explicit stale-version recovery. Shared draft protection covers route/browser exit; successful save clears only the dialog's own dirty state. |
| Dietary profiles | Live native review; new tests reject household-switch/discard confirmations and preserve drafts, retain inputs after simulated HTTP 503, require explicit reload confirmation and announce successful save. Optional notes preserve raw multiline input while emitting normalized API entries; canonical sorbitol selection survives note edits. Test declarations are restored after the assertion. |
| Access and person picker | Live account-visibility-respecting search/options review; persistent external labels and namespaced portalled option styles. Existing browser tests cover role-specific actions and stale access. |
| Calendar/editor/review/shopping | Live calendar, existing-meal and return-focus review; new tests verify edited cancel protection, existing/new headings, review focus and renewed acknowledgement on another preview. Existing daily/planning tests cover skips, constraints, explicit review/apply/removal and shopping quantities. |

Three targeted regressions failed against the unmodified deployed frontend before the fixes. The
updated three workflows and both affected daily workflows pass. A deliberately inverted discard
decision is caught by the household-switch regression; original source and built assets restored
before repeating the tests.

Profile screenshots inspected at 1440/390 px in native light/dark modes; bounds also checked at
768/320 px. The final screenshots disable animations to avoid capturing a navigation transition.
Scoped axe checks use WCAG 2/2.1/2.2 A/AA tags. No formal certification or exhaustive assistive-
technology claim. Manual browser review recorded no console warnings/errors in the native views.

Current checks: 58 Rust tests, seven client unit tests, strict client build and formatting,
catalogue import/verification, i18n, clippy, Rust formatting, licences and advisories pass.
Final immutable-image rollout, all twelve workflows together, restart integrity and exact-head CI
are pending and will be recorded in the PR before review readiness.

Implementation references: [Vue Router guards](https://router.vuejs.org/guide/advanced/composition-api)
and [Vue composable lifecycle cleanup](https://vuejs.org/guide/reusability/composables).
