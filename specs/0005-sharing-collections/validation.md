# F05 acceptance evidence

## Automated contract checks
`cargo nextest run --workspace --locked`: **109 passed, 0 skipped**.
`npm --prefix clients/nextcloud test`: **23 passed**.

| Acceptance | Observable evidence |
|---|---|
| 1: private and explicit sharing | Private collection creation/editing; source-owner-only sharing; immediate account grants, addressed household acceptance, direct owned-household grants; typed native management/incoming views |
| 2: access and revocation | Unrelated/anonymous access denied; current membership/role checks; no source household access for recipients; owner-only editing of shared collections; removed entries and collection deletion revoke access; pinned images retain MIME/no-sniff/no-store; restart and failed storage/lookup checks |
| 3: imported source URL only | Sticky HTTPS provenance, unsafe URL rejection, strict client source-only schema, no cached content in shared summaries/details/scaling/images, and retroactive restriction of older shared revisions without rewriting history |
| 4: explicit revisions | Collection references and personal publication remain pinned after source edits; explicit revision publication; historical collection reading with Open current revision; exact existing serving scaling and frozen ingredient evidence; existing meal-plan revision regressions remain green |

Focused regressions live in `tests/recipes.rs`, `src/exapp/library_tests.rs`,
`src/exapp/sharing_tests.rs`, `src/exapp/sharing_collection_tests.rs` and
`clients/nextcloud/tests/library-api.test.ts`. Fixtures contain no private audit findings.

## Repository and client gates
All commands passed:

```sh
python3 scripts/generate_docs.py --check
python3 scripts/import_catalogue.py --check
python3 scripts/verify_catalogue.py
python3 scripts/import_nutrition_estimates.py --check
python3 scripts/verify_nutrition_estimates.py
python3 scripts/verify_conversions.py
python3 scripts/verify_allergens.py
node --test tests/i18n.test.mjs
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo deny check licenses
cargo audit
npm --prefix clients/nextcloud run format:check
npm --prefix clients/nextcloud test
npm --prefix clients/nextcloud run build
npm --prefix clients/nextcloud audit --audit-level=moderate
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked
```

The production build transforms 861 modules and produces about 167 kB CSS (24 kB gzip) and
1.44 MB JS (442 kB gzip). These are build output sizes, not measured load-time claims.
No runtime dependencies or crates were added. Existing Zod annotation build notices do not affect
strict type checking. No global design reset, fonts or theme palette were introduced.

## Container and installed checks
`python3 scripts/test_exapp_container.py --image mealshop.local/mealshop:sharing-collections`
passed both isolated rootless TCP and Unix-socket modes. Alongside lifecycle, authentication, CSRF,
assets, private files, shopping and cooking, the smoke test creates a collection, grants a canonical
account read access, denies source-household traversal, corrects the recipe's imported origin and
verifies source-only access to its older pinned revision before revoking the grant. `library.json`
remains mode 0600.

The F05 image is installed in the existing Nextcloud 35.0.2 test deployment using its existing `AppAPI`
registration, secret, internal URL and data volume. A single writer is maintained during replacement.
All 23 existing/new browser scenarios have been exercised on the installed image. The first complete
sweep passed 21, hit one Nextcloud SQLite `database is locked` error during a synthetic recipient login,
and skipped the final serial scenario in that file. The test instance used `DELETE` journal mode.
After switching its supported `sqlite.journal_mode` setting to `WAL` and verifying the actual mode with
`PRAGMA journal_mode`, all three scenarios in `nextcloud.spec.ts` passed (48.8 s). This is a test-environment
change, not a Mealshop database migration. The F05 scenarios passed in that installed sweep (24.8/35.2 s).

The closing full sweep, final-candidate F05 rerun, image identity and exact-head CI are recorded in the PR.
Synthetic households are selected from the fixture ledger, checked against synthetic recipe/collection
identities and archived with the single writer stopped. Retained household directories are neither read
nor copied by this cleanup. Their membership entries and monotonic household identity counter are kept.
No broad original-file checksum audit is claimed.

## Browser coverage
`clients/nextcloud/e2e/sharing.spec.ts` covers two real Nextcloud accounts: personal revision
publication/scaling/revocation, private collection editing/reordering and focused search, historical
reading, native contact selection, household acceptance/removal, imported-source-only collection
entries, revoked-content clearing and browser exceptions. It checks EN/DE, system light/dark,
1440/390/320 px, no page overflow and scoped WCAG 2.2 AA axe scans. Native source controls are exercised
by keyboard. Browser evidence is scoped automated testing, not a blanket accessibility certification.
Screenshots remain local ignored artifacts of owned synthetic data.
