# Bootstrap verification

Date: 2026-11-07. Bootstrap source commit: `2a2ad812fd33992fba0e48c66fdd1e4291e3dfd3`.
The follow-up commit only records completion/evidence and regenerates the embedded docs.

## Local acceptance evidence
- `cargo fmt --all --check`: PASS.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: PASS.
- `cargo nextest run --workspace --locked`: 4 passed, 0 skipped. Health returns 200 and
  exactly `ok\n`; unknown recipes route returns 404; invalid address and occupied port fail startup.
- `cargo build --release --locked`: PASS.
- `cargo deny check licenses`: PASS; no upstream licence exceptions copied.
- `cargo audit`: PASS, 1,422 `RustSec` advisories loaded; no advisory exceptions.
- `cargo msrv verify -- cargo check --all-targets --locked`: PASS on Rust 1.85.0.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`: PASS.
- `python3 scripts/generate_docs.py --check`: PASS.
- SPDX JSON generation: 54 packages including the server and dependency graph.
- Real TCP smoke check with a custom listen address: HTTP 200/body verified; Ctrl-C exits 0.
- All 75 generated rustdoc project pages checked for missing local file links: none.
- Published Pages landing page fetched successfully over HTTPS.

## Manually dispatched GitHub workflows
All four completed with conclusion **success** against the bootstrap source commit:

| Workflow | Evidence | Coverage |
|---|---|---|
| CI | [Run 42367613245](https://github.com/Th3Link/mealshop/actions/runs/42367613245) | fmt, clippy, nextest/coverage, MSRV, debug/release builds, SPDX SBOM, binary metrics |
| Security audit | [Run 42367615043](https://github.com/Th3Link/mealshop/actions/runs/42367615043) | `RustSec` advisories and licence policy |
| Docs | [Run 42367617179](https://github.com/Th3Link/mealshop/actions/runs/42367617179) | strict rustdoc, Pages artifact and deployment |
| Renovate | [Run 42367619146](https://github.com/Th3Link/mealshop/actions/runs/42367619146) | existing repository token, scan and update PR creation |

The first push-triggered CI was superseded by the manual CI through the configured concurrency
policy. The manual run completed all six jobs successfully. Artifacts include debug/release
Linux `x86_64` executables, coverage, SPDX SBOM, per-profile metrics and the combined metrics report.
Metrics currently measure binary bytes and are stored as per-run artifacts; firmware memory
budgets/history from the ESP project are not applicable to this server bootstrap.

Renovate created update PRs for `actions/configure-pages` and `actions/deploy-pages`.
Those are separate dependency changes, not silently merged into the bootstrap.

## Delivered scope
20 feature contracts, six ADRs, README, contributor guidance, roadmap, full AGPL-3.0 text,
Tokio/Axum stub and four workflows. Only F01 is Done; F02–F20 remain Draft.
This verifies a delivery foundation, not production readiness of the planned product.
