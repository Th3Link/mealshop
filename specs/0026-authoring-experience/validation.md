# Acceptance evidence

Environment: existing Nextcloud 35.0.2 / `AppAPI` 35.0.0. Private pre-upgrade copy retained;
no reset, primary catalogue rewrite or AGENTS.md change. Browser iterations use a read-only built
asset mount on the real rootless application container; final packaged checks are recorded in the PR.

| AC | Evidence |
| --- | --- |
| 1–2 | `authoring-experience.spec.ts` finds Haferflocken as Oat flakes, creates a searched ingredient in place, retains recipe title/quantities, and checks the dialog with WCAG 2/2.1/2.2 A/AA tags. Screenshots cover the new dark mobile dialog and structured recipe. The subsequent editor check passes 320/390/768/1024/1440 px bounds. Native portalled menu content uses uniquely namespaced slot styles, not host-wide selectors. |
| 3 | `optional_constituents_aggregate_exactly_and_sorbitol_blocks_pinned_recipes` checks 6 g beta-glucan and 3.75 mg of a declared custom constituent, with exact portion rounding. `constituent_identity_units_unknowns_and_legacy_values_remain_consistent` checks unknown/partial subtotals, duplicate IDs, unit conflicts and compatibility values. The browser adds an optional field and sees its recipe subtotal after reload. |
| 4 | The same backend test checks sorbitol's pinned declaration still blocks after the current ingredient declaration is removed. Fourteen regulated allergen groups and their 106 exact positive mappings are unchanged. |
| 5 | `recipe_parts_preserve_associations_and_reject_cycles_and_unknown_references` checks line/part/step identity, dependency cycles including step references, exact scaling, missing/duplicate references and restart equality. The browser creates two named parts, assigns lines/steps and prepared parts, saves, reloads and edits them. |
| 6 | Full real browser suite: eight passed together in approximately 2.5 minutes, including reader/revocation/conflicts, German quantities, exact source conversions, images, calendar skips, themes and reduced motion. Backend suite: 55 passed; strict client build and seven unit tests passed. |

Offline catalogue import/verification, reference conversions and allergen verification pass. Format,
clippy, licence and advisory gates are retained. This evidence does not establish formal WCAG certification,
medical benefit, comprehensive intolerance coverage, or a parallel cooking/timer schedule. The broader
catalogue grouping is deliberately a subsequent slice after these checks.
