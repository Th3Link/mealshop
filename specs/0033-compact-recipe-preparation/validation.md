# F33 acceptance evidence

Target: native embedded Nextcloud 35.0.2 / `AppAPI` 35.0.0, Vue 3 and `@nextcloud/vue` 9.14.1.
The customer supersedes the older screens' collapsed parts: every recipe section remains open.

| AC | Observable verification |
|---|---|
| 1–2 | Real `compact-recipes.spec.ts` and updated authoring/preparation workflows exercise open parts, shared headers, accessible hidden row labels, inline ingredient creation, native plus menus, temperature editing and row trash. Desktop ingredients/instructions share a row; narrow layouts stack. |
| 3–4 | `homemade_parts_keep_declared_outputs_but_calculate_only_base_inputs` preserves original output quantity/source identity through revisions, scale and reopen, while nutrients/constituents use measured inputs. Stateless unresolved calculations fail. Display names come from the frozen catalogue, including outputs omitted from nutrient contributions. |
| 4–5 | `homemade_ingredients_purchase_only_inputs_and_keep_dietary_and_cooking_guards` checks exact direct shopping and idempotent plan sync, both finished/input hard exclusions, restart and main-before-producer instruction ordering. Real `AppAPI` role tests reject reader recipe writes; existing stale-write/isolation contracts remain exercised. |
| 5–6 | Invalid unknown/empty/self/cross-cycle/multiple/zero-mass/unknown-food preparations fail. Browser checks shared-part removal guards, disabled self-dependent step moves, cancelled conversion and confirmed conversion without losing the finished amount/main instruction. New preparation quantities start blank. |
| 7 | Embedded Chromium checks English/German, native system light/dark, reduced motion, keyboard focus, 320/390/768/1440 layouts and WCAG 2/2.1/2.2 AA axe tags. This supports an accessibility review, not a universal conformance claim. |

## Visual review
The approved reference composition informs density and hierarchy, retaining native Nextcloud colors,
fonts, controls and icons. Screenshots exposed native selector minimum-width overlap, clipped exact
amounts, offset trash buttons and a sticky title colliding with the navigation toggle. Scoped layout
rules correct those issues; full amounts and the added temperature's Celsius unit remain visible.
Draft tracking is limited to editing; fragment-only navigation keeps the draft and focuses the
preparation region. The read toolbar groups portion preview, display unit and cooking. Metadata and source evidence remain
open below the core recipe; no global reset, palette, typeface or UI framework was introduced.

## Checks
Repository/client gate: catalogue import/check, catalogue/conversion/allergen/group verification,
keyed-message tests, Rust fmt/clippy/nextest, licences/audit, native format/types/unit/build/audit and
strict rustdoc. The full Rust suite contains 90 tests; the client suite contains 16 tests. Browser
coverage contains 20 real embedded workflows, including shopping, cooking, invitations, conflicts,
image persistence, exact localized quantities and mealprep.

## Measured asset cost
The independent locked build contains 1,377,783 JavaScript bytes versus 1,367,492 in the previous
PR #27 test image. Python gzip level 9 with zero mtime gives 422,702 versus 419,997 bytes
(+2,705 compressed bytes). This measures asset size, not page-load speed. No dependency or crate
was added; preparation graphs and ingredient lines retain their existing bounded sizes.

## Operational verification
The installed `mealshop.local/mealshop:compact-recipes` image passes all 20 browser workflows
serially in the real Nextcloud shell. The final image passes isolated rootless TCP/Unix lifecycle,
authentication/CSRF, assets, private storage, shopping and cooking checks. Its JavaScript and CSS
are byte-identical to the independent locked build. A real restart preserves authenticated API
snapshots, cooking states/deadlines and all 39 persisted files (12 memberships and one referenced
image checked), including the saved homemade recipe revision.

Complete private backups precede installation and cleanup. Only verified ledger-owned synthetic
households and their own cooking notices were archived. Both pre-existing household membership rows
and all eight customer files match the initial snapshot; the cleaned instance starts authenticated
with the two original households retained. Four final [observed screens](screens/README.md) were inspected.

The repository/client gate passes with 90 Rust tests and 16 client tests. The independent strict
rustdoc and ingredient-group verification also pass. Remote CI is a merge gate on the exact PR head;
the PR records the observed check links without changing executable code to record that result.

No finished-food availability, recipe, raw/output ratio, nutrient retention, cooking loss or yield is
inferred. Independent saved subrecipes and multi-output yield models remain outside this explicit
recipe-owned preparation contract. Old binaries require restoring the matching complete snapshot.
