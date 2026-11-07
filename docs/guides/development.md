# Development and delivery

Start with AGENTS.md and the affected spec. Use stable Rust; Cargo.toml declares the tested
MSRV. Cargo.lock is committed; CI uses --locked. Unit preferences are presentation settings,
not a reason to erase source quantities. Keep core calculations independent of HTTP/storage.

## Local gate
```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo build --release --locked
cargo deny check licenses
cargo audit
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo msrv verify -- cargo check --all-targets --locked
```
Install cargo-nextest, cargo-deny, cargo-audit and cargo-msrv if absent. CI additionally
uses cargo-llvm-cov and cargo-sbom. Tests may unwrap assertion setup; production may not.

## Workflows
- CI: fmt/clippy, nextest/coverage, MSRV, debug/release artifacts, SPDX SBOM and size metrics.
- Security audit: `RustSec` advisory checks and dependency licence policy; runs daily as well.
- Docs: strict rustdoc with requirements/specs/ADRs embedded, uploaded and deployed to Pages.
- Renovate: daily self-hosted Renovate using `RENOVATE_TOKEN` (existing repository secret).

All workflows support `workflow_dispatch`. GitHub Pages must use the GitHub Actions build
source. Workflow permissions are scoped to the jobs that publish or create audit issues.
No upstream advisory ignores, ESP targets or machine-specific paths are retained.
To repeat a run: `gh workflow run <file>.yml --ref main --repo Th3Link/mealshop`.
Inspect `gh run list` and failed job logs; a dispatched workflow is not evidence of success.

## Documentation
`src/docs.rs` embeds the Markdown sources as rustdoc modules. Run `cargo doc --no-deps` and
open `target/doc/mealshop_server/docs/index.html`. The Docs workflow adds a root landing
page. After adding a spec or ADR, update its index; docs generation discovers Markdown files.
