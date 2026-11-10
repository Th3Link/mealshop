# F31 validation evidence

Status: In progress — native browser acceptance is not verified.

## Completed checks
- Import catalogue check and offline verifier: passed (313 ingredient IDs, 284 source profiles, 29 identity-only records).
- Node i18n test: passed.
- `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed; regenerate/check embedded documentation after evidence updates.
- `cargo nextest run --workspace --locked` with local test sockets allowed: 88 tests passed, zero skipped.
- Native Vue typecheck and production build: passed. Locked Node 24 container: `npm run format:check`, `npm test` (14 passed) and `npm audit --audit-level=moderate` (zero vulnerabilities) passed. The host Node 26 cannot run `npm ci` under the required engine; the container supplies the declared runtime without weakening engine checks.
- Licence gate: passed. `RustSec` audit: passed after allowing its cache lock/update.
- `docker build -t mealshop.local/mealshop:unified-preparation .` passed. `python3 scripts/test_exapp_container.py --image mealshop.local/mealshop:unified-preparation` passed for TCP and Unix sockets; isolated lifecycle, assets, auth/CSRF, private storage and cooking review. The native host still requires the final local image refresh and browser acceptance.

## Behavioral coverage
Recipe API tests preserve exact temperature spelling through revision persistence, scaling and restart, reject duplicates, and keep mass intact. Planning API tests preserve production-once requirements, day allocation movement, over-allocation/duplicate/slot-conflict rejection and restart. Cooking tests demonstrate a 07:00 calendar anchor followed by an actual 07:08 start and a 07:53 deadline; moving to the next action acknowledges only elapsed passive waits.

## Unresolved verification
The in-app browser failed to initialize, then reported a closed browser transport and no available browser. A fresh tab attempt also failed. No substitute browser automation was used. Therefore no new visual, responsive, accessibility, sidebar-focus or end-to-end Nextcloud acceptance is claimed. Approved drawings in screens/ are design references, not evidence of working UI. Do not mark this feature Done or merge before those checks pass.

GitHub authentication was confirmed outside the restricted sandbox. Browser acceptance remains an external blocker, not evidence of a successful launch.

## Code review before draft PR
Checked exact values and unit boundaries, legacy defaults, household authorization/revisions, bounded allocations, persistence, dependency cycles and shared shopping provenance. Corrected empty-allocation fallback, edits that could change production identity, review visibility inside the sidebar, stale review markers on allocated days, draft replacement and recipe-part collapse while typing. Recipe-derived completion starts passive time even when active work has a planning estimate. No dependency or crate was added. Browser verification remains a merge blocker.
