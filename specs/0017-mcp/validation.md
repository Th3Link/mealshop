# F17 validation evidence

Date: 2026-11-12. Implementation: `d6788cc`; final-head checks are attached to
[PR #32](https://github.com/Th3Link/mealshop/pull/32). F17 is complete for the documented Nextcloud
contract; no OAuth provider or independently delegated per-client credential is claimed.

## Acceptance trace

| Criterion | Evidence |
| --- | --- |
| AC 1: authenticated scoped discovery/read/preview | `tests/mcp.rs`, official SDK 2.4.0 modern direct HTTP and auto/legacy/pinned-modern stdio, and real Nextcloud native connection. |
| AC 2: canonical validation, provenance and audit | `src/recipe_creation_tests.rs`, canonical quantity/image/ingredient/composition validation, source-only retroactive historical reads, private first-revision actor/time and native pinned activity links. |
| AC 3: role/scope and household boundaries | `src/exapp/mcp_tests.rs`, disabled/read/create grants, current role intersection/removal, reader create denial, foreign-account/home denial and immediate MCP grant revocation. |
| AC 4: idempotency and structured quantity errors | concurrent HTTP insert/restart/key-conflict tests, official SDK retry identity and an actual installed container restart preserving one recipe and receipt. |

The 16 new Rust regressions comprise nine HTTP/protocol tests, two trusted Nextcloud adapter tests,
three canonical receipt tests and two settings/preview-lifetime tests. They cover malformed/oversized
RPC, metadata/header disagreements, Origin rejection, legacy missing params, exact quantity-only
input, no-write preview/denied confirmation, changed ingredients/grants, monotonic expiry, corrupt or
duplicate receipts and private 0600 settings/recipe files. A reproduced failed scope-revocation write
previously left stale capabilities usable; its regression now verifies that all tools and scope reads
return unavailable until recovery, with no recipe inserted.

## Local and independent-client gates

Passed:

- Catalogue import/check and verifier; nutrition-estimate import/check and verifier; conversion and
  allergen verification; demo syntax and keyed i18n tests.
- `cargo fmt --all --check`, Clippy with warnings denied, **142/142** nextest tests, licence policy,
  advisory audit and strict rustdoc. No runtime crate or advisory exception was added.
- Native Prettier, strict Vue/TypeScript and Vite build; **30/30** Vitest contracts; npm audit with
  zero vulnerabilities; generated MCP JSON schema checked against committed recipe contracts.
- **5/5** Python standard-library bridge tests: protected configuration, mirrored protocol headers,
  bounded input, redirect rejection and sanitized/unmatched upstream responses.
- Official SDK **2.4.0**: modern HTTP and auto/legacy/pinned-modern stdio interoperate with the actual
  Rust binary; exact quantities, preview/confirmation, repeated creation, private audit, historical
  source-only projection and scope reduction are asserted using isolated synthetic state.
- Final Rust-1.85 `ExApp` image passes the isolated rootless **TCP and Unix-socket** smoke suite,
  including existing shopping/cooking/sharing/history flows and MCP preview/create/retry/redaction.

CI includes an independent official-client/stdio job against its compiled debug artifact. The final
handoff requires all eleven checks to pass on the current PR head. The PR Checks tab is the
authoritative current-head record.

## Real Nextcloud and browser checks

Environment: existing Nextcloud **35.0.2**, `AppAPI` **35.0.0**, Chromium, native embedded routing and the
real authenticated proxy. The official SDK/stdio browser flow authenticates as the real owner and
accepted reader, creates from a quantity-only preview, retries after client restart, follows the
pinned audit link and proves that a subsequent imported restriction hides revision one's contents.
Owner reduction/revocation applies on subsequent calls. A second browser scenario verifies two-tab
conflict retention and explicit navigation/reload discard decisions.

EN/DE, system light/dark, reduced motion, keyboard and **1440/390/320 px** are covered. Axe WCAG
2/2.1/2.2 A/AA checks report zero violations in both owner and reader views at these sizes. Header,
connection and activity screenshots were inspected; no horizontal page overflow or page errors occurred.
This is automated/visual evidence, not a claim of formal accessibility certification.

All **28 distinct embedded browser scenarios** have passed across batches. The first complete
invocation passed 26 and hit the existing account household limit in unified preparation and visual
shopping: two additional manual app-password fixture households were created concurrently. After
archiving only owned fixtures, the permission-guard image passed **4/4** targeted cases: both MCP
scenarios plus both capacity-blocked scenarios. After adding exact source-portion discovery, the final
image passed both MCP scenarios again; the native application assets are unchanged. No product capacity was relaxed and no failing behavior was skipped.
Future full runs must start with fixture capacity available and avoid concurrent manual fixture creation.

A separate short-lived, uniquely named **actual Nextcloud app password** authenticated stdio discovery
through the real proxy. Only that test token was deleted. Its next calls were denied after the installed
Nextcloud authentication cache expired (**11.3 s observed**; installed provider TTL is 10 s). This
upstream cache is distinct from Mealshop's immediate per-call MCP grant/role checks. Credentials stayed
in process memory/environment and were never copied to a file or tool output.

## Installed restart and cleanup

Final installed image:
`sha256:ff307fe41d3d0ffc2ee6bbdcbbf305d9c32b3ff629701f44cbe3f218c6edbcb1`
(`mealshop.local/mealshop:mcp-recipe-tools`). It is healthy. Public JavaScript (**1,490,474 bytes**) and
CSS (**168,690 bytes**) match the image's asset hashes; the development SDK is outside the browser graph.
The image contains the verified implementation; subsequent changes are documentation and screenshot
coverage only.

An actual container restart in a newly ledger-recorded fixture retained its account grant and exact
first-revision creation receipt/audit. A retry returned that same revision, stayed source-only after a
later imported edit, and left exactly one recipe. The uncommitted preview disappeared on restart;
reducing scope then denied even a completed-write retry.

Exact source-portion discovery is additionally tested through the real proxy; custom food records
do not receive a neighbouring reference conversion.

All created browser/manual/restart households are privately ledger-recorded. With the sole writer
stopped, cleanup validates only those synthetic directories and archives them. All non-fixture
membership/library entries and allocation counters are preserved; retained household directories are
not inspected, hashed or copied. No test app password remains. The final fixture check reports zero active ledger households, and the cleaned installed container
is healthy.
AGENTS.md is unchanged.
