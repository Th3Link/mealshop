# F16 validation evidence

Date: 2026-11-12. The implementation and current-head CI are attached to
[PR #35](https://github.com/Th3Link/mealshop/pull/35). The delivery contract covers public structured
extraction and explicit private review, with manual entry for unavailable or unsupported sources.
It does not claim universal website support, login bypass or publication rights.

## Acceptance trace

| Criterion | Evidence |
| --- | --- |
| AC 1: URL preview or actionable unsupported/blocked result | `src/imports/parser_tests.rs`, `src/imports/network_tests.rs`, real public HTTPS fixture retrieval and native multi-candidate/manual controls. HTML, JSON-LD arrays/graphs and nested instruction sections are supported. |
| AC 2: explicit ingredient/quantity/step/provenance review; unknown originals | `tests/recipe_import.rs`, `src/imports/receipt_tests.rs`, Vue quantity/draft contracts and native browser review. No identity guessing, translation, implicit volume conversion or source nutrition/diet adoption. Missing required canonical values prevent save. |
| AC 3: persistent source restrictions | First-revision source receipt reload/edit/historical tests; `src/exapp/import_tests.rs` checks recipient source-only projection. Existing sharing/collection/MCP regressions remain required. |
| AC 4: hostile network/content boundary | DNS mixture/rebinding, private/special address families, alternate IP encodings, unsafe redirects, robots, size/type limits and non-executing parser checks. The browser fixture contains script/image sentinels that must never load. |
| AC 5: access restrictions and manual fallback | Robots and explicit paywall checks fail closed. Reader and foreign-account restrictions are covered at HTTP and trusted Nextcloud boundaries; manual entry makes no network request. |

The 12 new Rust regressions comprise three network-policy tests, three parser tests, two source
receipt tests, three HTTP import tests and one trusted Nextcloud adapter test. Concurrent identical
requests publish one recipe; changed payload/key reuse conflicts. The receipt and selected source
fields are atomically published with revision one, survive actual storage reopen and retain private
0600 permissions. Receipt keys and preview IDs do not appear in public recipe projections.

## Local gates and container compatibility

The complete local gate passed:

- Catalogue import/check and verifier; nutrition-estimate import/check and verifier; conversion and
  allergen verification; demo syntax and keyed i18n tests.
- `cargo fmt --all --check`, Clippy with warnings denied, **154/154** nextest tests, unchanged licence
  policy, advisory audit and strict rustdoc. Existing dependency versions remain pinned; the new
  HTML5 tokenizer and robots matcher add no browser runtime or advisory exception.
- Native Prettier, strict Vue/TypeScript and Vite build; **32/32** Vitest contracts; npm audit with
  zero vulnerabilities; generated MCP schema verification; **5/5** Python bridge tests and official
  SDK modern HTTP and auto/legacy/pinned-modern stdio interoperability.
- `cargo +1.85.0 check --workspace --all-targets --locked` passes; the pinned container compiler
  is verified directly as **1.99.1**. The final rootless image passes isolated **TCP and Unix-socket**
  smoke tests, including manual import, unsafe-target rejection, explicit review, identical retry,
  reader denial and private first-revision storage alongside existing domain and MCP flows.

The final handoff requires all eleven CI checks to pass on the current PR head. The PR Checks tab
is the authoritative current-head record; no failing check is waived.

## Native browser evidence

Environment: existing Nextcloud **35.0.2**, `AppAPI` **35.0.0**, Chromium, authenticated embedded routes
and the actual proxy. Three new import scenarios passed against the implementation using its native
assets; all three also passed against the installed image. A real public HTTPS request retrieves
the committed synthetic fixture; transport is not mocked. Before merge, `MEALSHOP_IMPORT_FIXTURE_URL` points to the immutable implementation commit;
the default points to the versioned fixture on main after merge.

The import scenarios cover consent, preserved exact `100.500000 g` input, explicit ingredient IDs,
original versus revised source fields, source-only sharing notice, rejected unsafe URLs, manual
fallback, required missing values, lost successful-response retry without duplication, explicit
discard and an actual accepted reader's hidden/disabled/denied import operations. Source script,
image and health-claim sentinels are never executed, loaded or promoted into recipe nutrition.

EN/DE, system light/dark, reduced motion, keyboard and **1440/390/320 px** review views pass axe
WCAG 2/2.1/2.2 A/AA with zero reported violations and no horizontal page overflow or page errors.
Screenshots were visually inspected. This is automated/visual evidence, not formal certification.
The saved source text remains separate from reviewed ingredient mappings and quantities.

A deliberately delayed retry response reproduced an unsolicited redirect after the user confirmed
leaving the import. The import now captures its view generation/destination and ignores responses
after navigation or unmount. The retry regression requires the recipe list to remain open with
exactly one imported recipe after the held response arrives.

All **31/31 distinct embedded browser scenarios** have passed against the final installed image
across batches. The first implementation image also passed the complete **28/28** existing suite in
one invocation. The final navigation-guard image passed the three import scenarios **3/3**.

Its repeated baseline invocation passed 24 scenarios, failed two and did not run two serial dependants:
a native navigation click stalled after scrolling, before reaching the meal-plan view. That setup had
not created the meal required by a later usability scenario. The trace showed successful fixture API
responses and no application console errors; the failed action could not capture a final screenshot.
The underlying cause of that one click stall was not established. After synthetic cleanup, the entire
Nextcloud/usability block passed **6/6**, including every initially failed/skipped scenario, with fresh
fixtures, no parallel local builds and unchanged assertions/timeouts. No persistent product failure
was reproduced or failing scenario waived.

Synthetic cleanup between batches respects the existing 20-household account limit. No capacity,
assertion or timeout was relaxed, and no concurrent manual fixture was created.

## Installation, restart and cleanup

Installed implementation image:
`sha256:4f51e81eba963f7b078aa3682339d0230cb004b66140fdf5cc095eada192464a`
(`mealshop.local/mealshop:reviewed-recipe-import`). Public JavaScript (**1,510,135 bytes**) and CSS
(**169,350 bytes**) match the installed container asset hashes. The implementation image is healthy;
the image contains the final behavior, including the navigation guard. Subsequent changes update
documentation only.

An actual container restart in a newly recorded synthetic household retained the fetched source
snapshot and atomic import receipt. An identical confirmed retry returned the same first revision,
including after a normal edit; there was exactly one recipe. Uncommitted previews disappeared on
restart, later edits retained original source fields and a pinned MCP read remained source-link-only.

With the sole writer stopped, cleanup validated and privately archived only newly ledger-recorded
synthetic households. Every non-fixture membership/library entry and allocation counter was
preserved; retained household directories were never inspected, hashed or copied. The final ledger
check reports **zero active synthetic households** and the cleaned installed image is healthy.
No reset or retained recipe inspection was performed. `AGENTS.md` is unchanged.
