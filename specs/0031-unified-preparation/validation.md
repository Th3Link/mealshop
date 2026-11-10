# F31 acceptance evidence

Status: Done — accepted native Nextcloud scope.

Target: actual Nextcloud 35.0.2 / `AppAPI` 35.0.0, native Vue 3 client and protected Rust backend.
Approved PNGs in screens/ remain design references, separate from actual browser screenshots.

| AC | Observable evidence |
|---|---|
| 1 | `e2e/unified-preparation.spec.ts` creates preparation and eating allocations in one calendar and opens the linked cooking schedule. Existing daily/calendar tests cover flat meals and skipped attendees. |
| 2 | Four portions are produced once, allocated two per eating day, then moved without duplicating the synchronized `100.000002 g`. Planning API tests cover restart, empty allocations, over-allocation, duplicate/occupied slots, revisions and immutable shopping provenance. |
| 3 | Native authoring tests create ingredient/step groups with stable prepared-part references, retain the parent draft during inline ingredient creation, then reopen the saved revision. |
| 4 | API tests preserve exact original Celsius spelling through save/reopen, scaling and restart, rejecting malformed/duplicate/unsupported extensions. The browser saves, edits and reads `60.000001 °C` in cooking without changing mass or identity. |
| 5 | API tests anchor planned work at 07:00 but start a 45-minute wait from actual 07:08 completion, ending at 07:53. The browser starts a one-second wait after work with a 120-second estimate, then advances the dependent part without a readiness assessment. Independent timers, offline/reload recovery and multi-day scheduling remain covered. |
| 6 | Nine approved concept PNGs are retained. Actual embedded workflows, API tests, client checks and the repository gate provide implementation evidence. |

## Checks and native review
Catalogue import/verifier, conversion/allergen verifiers and Node i18n passed. Rust formatting,
workspace Clippy with warnings denied, 88 nextest tests (zero skipped), licence policy, `RustSec`
audit and strict rustdoc passed. Locked Node 24.15 / npm 12.2.2 client formatting, strict Vue/`TypeScript`
build, 16 unit tests and npm audit (zero vulnerabilities) passed. The full installed-container embedded suite
passed all 19 tests. The final shopping-only label refinement separately passed the four existing
shopping contracts and its visual/localization test. Rootless container smoke tests
passed for TCP and Unix sockets: lifecycle, assets, auth/CSRF, private shopping/products and cooking.

The sidebar has one native title, distinct production/allocation labels and focus after its opening
transition. A completed preview receives focus. Native dialogs use their close lifecycle, draft
protection and connected invoker focus restoration. The timer regression explicitly holds an old
background response, proving pause refresh before that response returns rather than using a speed
threshold. Reviewed precision/provenance, legacy defaults, household authorization/revisions,
bounded allocations, source stability, dependency cycles, persistence and production-once shopping.
No dependency or crate was added.

## Operations and limits
An actual container restart preserved authenticated API snapshots, cooking deadlines/timer states and
35 persisted files (11 memberships, one referenced image). Verified fixture archival then retained both
customer memberships and their eight files byte-for-byte against the initial PR #27 backup. The final repeat passed; only ledger/content-verified synthetic
households and their identified reminders are archived. Both customer households remain active, with
monotonic IDs preserved. The final documentation-only image rebuild retains verified frontend bytes
and is checked before handoff. Complete private backups and the original functional image remain available.
Never run host/container writers concurrently or use customer households as acceptance fixtures.

One shared attendee list conservatively applies to all eating allocations. Calendar anchors are
planned reminders; author temperatures and elapsed waits establish neither device measurements nor
food readiness. Physical scale/heater integration, guaranteed device alerts and standalone/Flutter
clients remain outside this slice. Remote CI must pass on the exact PR head before handoff.

Actual [calendar/cooking screenshots](verification/README.md) were reviewed; an additional read-only
calendar check passed four widths and the WCAG-tagged axe audit.
