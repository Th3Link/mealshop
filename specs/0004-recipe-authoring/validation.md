# Recipe slice validation

Date: 2026-11-07. F04 Done after customer-authorized completion; F18 remains In progress.

## Automated evidence
- `python3 scripts/generate_docs.py --check` passed.
- `python3 scripts/import_catalogue.py --check` and `python3 scripts/verify_catalogue.py` passed:
  unchanged 313 ingredients, 284 profiles, 29 identity-only records and 3,778 numeric values.
- `node --check src/web/demo.mjs` and `node tests/i18n.test.mjs`: six locale tests passed.
- `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --locked -- -D warnings` passed.
- `cargo nextest run --workspace --locked`: 27 tests passed, none skipped, including seven new
  durable-recipe acceptance tests and the loopback-only startup check. Startup socket checks
  require an environment that permits loopback binding; an initial sandboxed run rejected
  that operation, and the full run with the needed permission passed.
- `cargo build --release --locked`, `cargo deny check licenses` and `cargo audit` passed.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked` passed.
- `cargo msrv verify -- cargo check --all-targets --locked`: Rust 1.85.0 compatible.

`tests/recipes.rs` verifies create/list/load with exact original gram strings and step durations,
restart round trips, historical/latest retrieval, concurrent stale-write rejection, source
snapshot stability after catalogue changes, new-revision recalculation, invalid inputs,
unknown composition, exclusive locks, corrupt/interrupted-store startup rejection and file
publication failure without overwrite or subsequent writes. `tests/startup.rs` verifies the
executable rejects non-loopback exposure before authentication exists.

## Browser evidence
Using temporary storage and the local server, create a German Hafer-Chia-Frühstück recipe
with 100,500000 g oats, 25 g chia, two servings and one step (60 active / 300 passive seconds).
Saving produced revision 1 with 256,22 kcal / 10,55 g protein per serving. Reloading the page
and loading the saved recipe preserved title, exact gram string, instruction and both durations.
Changing the same batch to four servings and saving produced revision 2 with 128,11 kcal;
the ingredients were not proportionally scaled. Invalid German `1.000` was rejected while
keeping input and loaded revision. Switching English preserved `100.500000`, authored German
text and durations; reloading restored the saved revision and nutrition. German saved-list,
load/success/error labels were exercised without product UI design work.

## Review limits
This is a local, single-household file slice with self-reported attribution, not authenticated
ownership, Git-backed storage, SQL adapters, timers, product UI or proportional scaling.
Filesystem failure tests simulate a target collision and interrupted/corrupt files; they do not
claim power-loss validation on every filesystem. Linux/local hard-link and fsync support is
required; POST retries after network timeouts need list/history inspection. Operator recovery
and bounds are documented in the recipe guide. GitHub CI/security evidence belongs in the PR.

## Completion evidence
AC coverage: recipe authoring/revisions/restart/conflicts and unknown data are covered by
`tests/recipes.rs`; original units, exact conversion and unavailable context by quantities
and recipe tests. All 43 curated portions are verified against frozen sources and round-trip
through saved recipes. Local display preferences do not alter stored input. Scaling tests
cover non-terminating ratios, repeated target changes, invalid counts, mass bounds, unknown
nutrition, original-yield retention and restart. Nutrition uses the original rational ratio;
gram strings alone are rounded for display, never fed back into nutrient computation.
No authenticated or cross-device preference claim is made. Existing de/en presentation is
retained; conversion hints identify required ingredient evidence and spoon standards.
Completion commands: import/check verifiers, locale tests, fmt, clippy, nextest, licenses,
audit, strict rustdoc and MSRV. Final outcomes and CI links are recorded in the completion PR.

Final local gate passed: 33 nextest tests, six locale tests, catalogue and conversion verifiers,
fmt, clippy, licenses, audit, strict rustdoc and MSRV 1.85.0.

Browser completion check passed: missing ml evidence produces an actionable hint; g/kg
automatic display shows 1500 g as 1.500000000 kg and 25 g as 25.000000 g without changing
stored input. The batch-serving editor and proportional preview are explicitly distinguished.
