# Combined F06/F07/F08 evidence

Environment: Nextcloud 35.0.2, `AppAPI` 35.0.0, Vue 3, @nextcloud/vue 9.14.1; real embedded local test
installation. Synthetic households are recorded/archived; customer data is never a test write target.

## Acceptance mapping
| Contract | Observable verification |
|---|---|
| F06 AC1–3 | tests/shopping.rs: exact units, aggregation, selected plan revisions, immutable purchases, rescale/remove, extras and restart |
| F06 AC4 | e2e/shopping.spec.ts: shopping mode, 1440/768/390/320 bounds, axe WCAG tags and native touch targets |
| F07 AC1/4 | Protected API and embedded preferences/fallback/manual-order flow; unchanged source quantities/provenance |
| F07 AC2/3/5 | Three-session threshold, manual precedence, correction exclusion, opt-in, inspect/reset/disable and restart |
| F08 AC1–3 | GTIN/check digits, package snapshots, explicit match confirmation, denied-camera/failed-provider manual continuation |
| F08 AC4 | Bounded provider records, no nutrition/match inference, forged-candidate rejection; consent denial has no file/network mutation |
| F21/F22 | Native locale tests and embedded suite; original cross-client criteria retained verbatim in F29/F30 |

## Automated results
- Required catalogue import/verifier and demo i18n checks pass.
- `cargo nextest run --workspace --locked`: 67 tests pass; strict workspace Clippy, formatting, licences and audit pass.
- `npm --prefix clients/nextcloud run test`: 10 tests pass; strict Vue build and whole-client formatting pass.
- Full npm audit: zero vulnerabilities; no dependency or crate additions.
- Isolated rootless container tests pass for TCP and Unix socket, including shopping/products and mode-0600 storage.
- Mutation check: removing the shopping revision guard fails the stale-write acceptance test; original source restored.
- Complete embedded suite on the deployed container: 16/16 pass, including roles/revocation, previous authoring/planning/ingredient views, light/dark/reduced motion, responsive bounds and axe checks.
- Scan-adapter tests use a local synthetic `MediaStream` and native API stub; denied permission and failed lookup retain full manual entry.
- F29/F30 acceptance criteria compare byte-for-byte with the original F21/F22 criterion sections.
- Final image presentation checks: 4/4 shopping browser contracts pass, including product-dialog responsive bounds at 390/320/1440 and axe WCAG checks. Screenshots were inspected locally.
- Upgrade and restart: all authenticated API snapshots and 24 persistent files remain identical; one referenced image verified.
- Final cleanup: only the retained customer household remains; own recipe access works, all retained files equal the initial pre-test backup, 19 household slots are free, monotonic IDs are preserved.
- Final deployed local image ID: d976a819339f; no production deployment or external publication. Physical mobile camera
hardware and global provider coverage are outside this environment. Simulated denied-camera and
synthetic provider records establish fallback/data handling, not universal optical recognition.

## Data cleanup
The previous 36 confirmed browser households were archived with a complete private backup. Customer
files were retained byte-identically. New fixtures have a private ID ledger plus content checks;
unrecognized/customer content is excluded. No credentials/private household content enter the PR.

## Native tile refinement (F32)
The existing source/quantity/permission contracts remain intact. Searchable native tiles, exact inline
entry, check/undo, item details and their real embedded acceptance are recorded in
[F32 validation](../0032-visual-shopping/validation.md). No endpoint or persisted model changed.
