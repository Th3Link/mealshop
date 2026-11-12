# F11 acceptance evidence

## Contract and automated checks
`cargo nextest run --workspace --locked`: **126 passed, 0 skipped**.
`npm --prefix clients/nextcloud test`: **28 passed**.

| Acceptance | Observable evidence |
|---|---|
| 1: actual completed cooks | Explicit manual confirmation, actual attendees/date/timezone, optional exact reported effort and 1–5 ratings; active/cancelled sessions rejected; completed-session recipe/servings pinned; duplicate/stale writes rejected; original/correcting actors retained; historical recipe reading and correction after session removal |
| 2: incremental, explained suggestions | Contribution subtraction/replacement/deletion, selected profile scopes, persisted counts/sums/dates/revisions, current-epoch reconstruction on restart; coherent whole-pool ordering and 20-preview pages; reasons, support counts and earlier-revision evidence in native planning |
| 3: cold start and discovery | Rule fallback, explicit absent/limited/repeated history, declared-dislike ordering, early discovery within equal-dislike groups, 25-candidate pagination regression; canonical feasibility precedes ranking |
| 4: model comparison | Versioned synthetic holdout compares hard violations, held-out acceptance, distinct first choices and rating utility against `rules-v1`; see [evaluation](evaluation.md); these are offline proxies, not measured human satisfaction or causal gains |
| 5: reset and opt-out | Off by default, owner-only scope/reset/bulk-clear, new epochs without replay, same-scope saves preserve contributions, separate retained-history deletion, readers cannot write, recipe grantees gain no history access, removed members denied, strongest positive ratings cannot override exclusions before/after opt-out |

Focused tests are in `tests/feedback.rs`, `src/feedback/`, `src/exapp/feedback_tests.rs` and
`clients/nextcloud/tests/feedback-api.test.ts`. Fixtures are synthetic and contain no household audit data.
The fixture comparison gives 0 hard violations in both modes, acceptance hits 0/4 versus 4/4,
distinct first choices 1 versus 2 and rating points 4/4 versus 20/4. These intentionally learnable
examples do not establish real-world satisfaction, fairness or generalization.

## Repository and client gates
All 17 required catalogue, nutrition evidence, conversions, allergens, localization, formatting,
Clippy, Rust test, licence/advisory, strict Vue build, contract and npm audit gates passed. Strict
rustdoc and demo syntax checks also pass; generated documentation is checked after status updates.

```sh
python3 scripts/generate_docs.py --check
python3 scripts/import_catalogue.py --check
python3 scripts/verify_catalogue.py
python3 scripts/import_nutrition_estimates.py --check
python3 scripts/verify_nutrition_estimates.py
python3 scripts/verify_conversions.py
python3 scripts/verify_allergens.py
node --check src/web/demo.mjs
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

The production client transforms 873 modules and produces approximately 168 kB CSS (24 kB gzip)
and 1.48 MB JS (451 kB gzip). These are build output sizes, not load-time measurements.
No crate/runtime dependency, remote training service, global CSS reset, font or theme palette was added.
Existing upstream Zod annotation notices do not affect strict type checking.

## Native browser evidence
`clients/nextcloud/e2e/feedback.spec.ts` exercises completed manual/session recording, exact effort,
future-date rejection, corrections, retained conflicting drafts, explicit reload/discard, pinned recipe
revisions, owner opt-in/reset, no automatic plans-as-cooks, two-account reader restrictions, discovery,
positive/negative ratings, hard exclusions and the normal acknowledged plan review.

Form, history/detail, learning and suggestion states are checked in EN/DE, system light/dark,
1440/390/320 px with no page overflow and scoped WCAG 2.2 AA axe checks. Native checkbox actions use
the keyboard. Screenshot inspection refined native field labels, compact definition-list spacing and
learning controls; results receive focus after loading. These are scoped automated checks, not blanket
accessibility certification. Screenshots remain local ignored artifacts of owned synthetic data.

## Installed container verification
The complete installed Playwright sweep passed **26/26 (9.4 minutes)**. After the final scoped
alignment changes, all three F11 scenarios passed again on the final image (**3/3, 1.4 minutes**).
That image is `mealshop.local/mealshop:cooking-history-feedback`,
`sha256:16877aa292e05b95ecd2110320f1c0dbedf73daff73d0099d9d2cb8b742f4165`, built from
implementation commit `f728682`. Later acceptance/status documentation changes do not change the
functional server/client implementation. The final image passes
`python3 scripts/test_exapp_container.py --image mealshop.local/mealshop:cooking-history-feedback`
in both isolated rootless TCP and Unix-socket modes, including F11 history/learning/constrained
suggestions and existing lifecycle, authentication, CSRF, sharing, shopping and cooking checks.

It is installed in the existing Nextcloud 35.0.2 test deployment with its existing `AppAPI`
registration, secret, internal URL and data volume. Only one application writer runs. Public JS/CSS
responses match the running container's assets, and the expected image reports healthy.

A separately ledger-recorded synthetic fixture verifies an actual installed container restart:
history list/detail, exact 125-second effort, actor provenance, policy/epoch/counts and explained
suggestion responses remain identical. Opt-out erases aggregates while retaining raw history;
re-enabling leaves contribution counts at zero instead of replaying that history.

After testing, only ledger-recorded synthetic households are validated and archived with the single
writer stopped. Retained household directories are neither read nor copied by cleanup. Retained
membership/library entries and monotonic household identities are preserved; no active ledger-owned
fixture remains. No broad original-file checksum audit is claimed. `AGENTS.md` is unchanged.
The final container is restarted and healthy after cleanup. Exact-head remote CI is tracked in the PR.
