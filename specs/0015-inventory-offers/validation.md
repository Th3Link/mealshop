# F15 validation evidence

Date: 2026-11-12. Installed acceptance is complete; the current-head PR Checks tab is authoritative
for the mandatory eleven-check CI handoff gate. No check is waived.
The delivery covers exact stock and manually entered offers, not a provider integration or scraping.

## Acceptance trace

| Criterion | Evidence |
| --- | --- |
| AC 1: exact lots and corrections | `src/inventory/{lots,ledger,validation}.rs`, foundation/API tests and native add/count-to-zero/archive flow. Original quantity/unit, frozen identity, location, batch, date and append-only reasoned corrections survive reload. |
| AC 2: visible reservations | `src/inventory/allocations.rs`, shopping projection/source reconciliation tests and native reserved/covered-by-stock flow. Exact dimensions, expiry/use date, source changes and exact product identity are checked. Plans and check-off never debit physical stock. |
| AC 3: explicit physical bookings | `src/inventory/{purchases,cooking,operations}.rs`, concurrent/restarted retry and publication-failure tests; native purchase/cooking/reversal/ambiguous-response flow. Partial consumption remains unbooked; another source's holds cannot be used. |
| AC 4: manual offers | `src/inventory/offers.rs`, current-profile exclusion and frozen source tests; native localized price, package, validity, source and reviewed add flow. A profile change after preview blocks confirmation; old shopping facts survive editing/archive. |

Ten isolated API tests plus two foundation tests cover exact normalization, old/default-empty state,
private publication, concurrency, restart, changed-key conflicts, corruption and ambiguous publication,
dimensions, passed dates, product GTIN matching, source changes, incomplete cooking, reverse/rebook,
current dietary constraints and actual movement pagination. A failed/uncertain store refuses reads,
new writes and replays until recovery; no stale pre-publication balance is returned as current.

## Local gates and review

The complete 23-command local gate passed: catalogue/nutrition/conversion/allergen checks; demo syntax
and i18n; fmt, Clippy with warnings denied, **166/166** nextest tests, unchanged licence policy and
advisory audit; native Prettier, **33/33** Vitest tests, strict Vue/TypeScript/Vite and zero-vulnerability
npm audit; generated docs/MCP schema, **5/5** Python bridge tests, official MCP SDK modern HTTP and
modern/legacy stdio interoperability, locked debug build and strict rustdoc. The last layout refinement
was rechecked with Prettier, all 33 client tests and the production build.

`cargo +1.85.0 check --workspace --all-targets --locked` passes. The actual compiler in the unchanged
pinned image is verified as **rustc 1.99.1**; the old CI display label mentioning 1.85 is not evidence of
that image's compiler. No new runtime dependency, advisory exception or loosened lint is introduced.
The final handoff requires all eleven checks to pass on the current PR head.

The five-axis code review found a missing movement offset/limit and late-response/locale review
invalidation risks. Regression coverage now enforces exact history pages, unchanged quantities across
language changes, immutable ambiguous retries and no unsolicited navigation after a delayed response.
An intentionally inverted stock-versus-purchase classification fails the new regression; the original
code was restored before all client checks. Fmt/Clippy findings were fixed, not suppressed.

## Native browser acceptance

Environment: existing Nextcloud **35.0.2**, `AppAPI` **35.0.0**, Chromium, actual authenticated embedded
routes/proxy and native assets. Five new scenarios pass against the implementation in this environment:

1. Exact stock, shopping reservation, completed cooking, physical consumption and reversal.
2. Lost successful response, frozen values, identical retry, one movement and delayed navigation guard.
3. DE/EN price input, offer review/add, exact package multiplication, purchase booking and reversal.
4. An actually invited/accepted reader sees stock but no mutation actions; direct writes are denied.
5. A real second-tab conflict retains the draft, latest-state reload/review, zero correction and archive.

Stock, reservation review, offer review and movement history are checked in EN/DE, system light/dark,
reduced motion and **1440/390/320 px**. Axe WCAG 2/2.1/2.2 A/AA reports zero violations; no horizontal
workspace overflow is accepted. Screenshots under ignored `test-results/inventory-*.png` were visually
inspected for alignment, labels, exact quantities, native contrast and density. This is automated and
visual evidence, not formal accessibility certification. Early harness iterations were corrected to
await confirmed membership/purchase changes and retain the existing ungrouped exact-quantity format;
the completed suite passes without relaxing its functional or accessibility assertions.

## Installed verification and cleanup

The installed implementation image is
`sha256:15b52166d8cd7e4bf1b604b48295e9ee249f58018da6c4d07d114e92d91ec20e`.
Public JS/CSS served by the `ExApp` match the container assets byte-for-byte and health is verified.
The final image passes both isolated **rootless TCP and Unix-socket** smoke transports, including
preview-without-write, exact stock booking, identical retry, denied foreign-account writes and 0600
storage alongside existing authentication, CSRF, shopping, products, cooking, sharing, history, MCP
and recipe import. The image contains the final runtime implementation; evidence-only documentation
updates do not claim a second runtime change.

All **36/36 distinct installed browser scenarios** passed in final batches of **16/16**, **12/12** and
**8/8**, covering the 31 existing scenarios and five new F15 scenarios. No final-batch failure was
waived. The five new scenarios also passed earlier against the local implementation through the
real Nextcloud proxy. Tests run without competing image/Cargo builds.

Main advanced during acceptance with the already customer-merged Node build-image and Vue-Vite-plugin
updates (PRs 33/34, main `6709ccf`). An ordinary merge incorporated them; clean npm installation,
33 client tests, strict production build and both rootless transports passed again. The rebuilt
image retained the exact same ID; independent hashes of its public server binary, JS and CSS were
byte-identical to those used for the earlier installed batches. No retained data was hashed.

An additional ledger-owned synthetic household verifies actual installed restart: original exact
`0.100001 kg`, one opening movement, a dated source allocation and reduced proposed purchase survive.
Replaying both original requests returns the same operations without duplication, even with their
old expected revisions. Purchase quantity remains zero. The private snapshot has mode 0600. Normal
Docker health and asset identity are rechecked after restart; the harness waits for the regular health
probe to leave its initial starting state rather than treating an immediate probe race as a failure.

Final cleanup reports **zero active ledger-owned test households** and the expected image is restarted.
Only ledger-owned synthetic households are used. Cleanup stops the sole writer and validates fixture
identity/content before archiving its folders and removing its membership/library entries. Every
non-ledger household directory remains uninspected, uncopied and unhashed; all retained membership/
library records and allocation counters are preserved. No reset or old private backup is used.
