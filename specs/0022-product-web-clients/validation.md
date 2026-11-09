# Nextcloud client validation

Environment: local Nextcloud 35.0.2, `AppAPI` 35.0.0, Chromium, native embedded route. Only fictional browser-test households and profiles were used. Host data was not reset.

## Completed checks

- Catalogue import check and offline verifier: 313 records; source identity and unknown composition preserved.
- `node --test tests/i18n.test.mjs`: passed.
- `cargo fmt --all --check`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed after correcting generated documentation formatting.
- `cargo nextest run --workspace --locked`: 43 passed, zero skipped.
- `cargo deny check licenses`: passed with explicit version-scoped dependency exceptions.
- `cargo audit`: passed, 140 locked dependencies checked.
- Client `npm run format:check`: passed.
- Client `npm test`: four tests passed.
- Client `npm run build`: strict type checking and production build passed; 620.70 kB JavaScript, 80.26 kB CSS before compression.
- Client `npm audit --audit-level=moderate`: zero vulnerabilities.
- Final `docker build -t mealshop.local/mealshop:0.1.0 .` passed (image da02292bdfa1). `python3 scripts/test_exapp_container.py` passed for rootless TCP and Unix sockets, lifecycle authentication, assets, custom CSRF header, JSON content type and private persistence.

## Browser evidence and limits

The Playwright suite drives household creation, addressed invitations, reader permissions, revocation, stale revision recovery, exact quantity display/scaling, original quantities, curated portions, dietary constraints, profile editing, planning and shopping-list application. It checks actual host light/dark themes, reduced motion, keyboard navigation, responsive bounds at 320–1440 pixels and automated accessibility rules. Screenshots and failure traces are private ignored local artifacts; accessibility automation does not establish complete WCAG conformance.

Final sequential run: `playwright test --config clients/nextcloud/playwright.config.ts` passed all four browser tests in 52.8 seconds. An earlier run during a simultaneous frontend rebuild timed out; the final run used stable assets.

## Verified test-container deployment

On 2026-11-09, the customer explicitly approved the internal `nextcloud` trusted domain and container switch. The hostname was added at index 3 without replacing existing entries. The manual process was gracefully stopped, its storage lock released, and the prepared container started with the same secret, port and persistent directory. The container healthcheck passed.

The complete four-test Playwright suite passed against the container in 58.5 seconds. A subsequent graceful container restart preserved byte-equivalent canonical JSON responses for the authenticated owner's household list, memberships/roles, recipes, profiles and plans across all 14 test households. Startup restored enabled state and authenticated access without a manual enable hook. No private payload was printed. A further complete browser run after restart passed all four tests in 58.3 seconds. Visual checks now await loaded fonts, native navigation state and completed finite transitions before measuring themes or capturing screenshots; PNG width and document bounds are asserted at each target width. No host animation or style override was introduced.

All ten checks for the initial PR commit passed in GitHub CI, including image build, native client, Rust quality/coverage, MSRV, both build profiles, licences, advisories, SBOM and binary metrics. The container-rollout commit (27324ea) also passed all ten CI checks, including the newly added isolated transport tests.

The standalone shell, full production `HaRP` tunnel deployment and Nextcloud account deletion/recreation reconciliation are outside this completed native-client implementation and must not be presented as implemented. See the deployment guide and ADR 0012.
