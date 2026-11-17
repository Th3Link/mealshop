# Development and delivery

Start with AGENTS.md and the affected spec. Use stable Rust; Cargo.toml declares the tested
MSRV. Cargo.lock is committed; CI uses --locked. Unit preferences are presentation settings,
not a reason to erase source quantities. Keep core calculations independent of HTTP/storage.

## Local gate
```sh
python3 scripts/generate_docs.py --check
python3 scripts/import_catalogue.py --check
python3 scripts/verify_catalogue.py
node --check src/web/demo.mjs
node --test tests/i18n.test.mjs
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
- Release: pushed stable `vX.Y.Z` tags reuse CI/security/Android gates, verify both native container
  architectures, and publish the multi-platform image plus OCI Helm chart to GHCR. See
  [releasing and installing](releases-and-helm.md). No customer cluster is deployed by CI.

All non-release workflows support `workflow_dispatch`; release publication requires a tag.
GitHub Pages must use the GitHub Actions build
source. Workflow permissions are scoped to the jobs that publish or create audit issues.
No upstream advisory ignores, ESP targets or machine-specific paths are retained.
To repeat a run: `gh workflow run <file>.yml --ref main --repo Th3Link/mealshop`.
Inspect `gh run list` and failed job logs; a dispatched workflow is not evidence of success.

## Renovate licence inventories

The self-hosted workflow renders its runner policy with
`python3 scripts/renovate_licences.py --config`. Cargo/Pub updates run the guarded refresh once per
branch before Renovate commits. Tool constraints come from `clients/flutter/toolchain.json`;
Flutter's exact revision and Rust's version are checked again by the generator. Containerbase
supplies the tools without an Android SDK, emulator or signing key. SDK package detection works
with both the project-local Flutter checkout and Containerbase's versioned installation paths.

Only `python3 scripts/renovate_licences.py` is authorized as a post-upgrade command. Only
`clients/flutter/dependencies.json` and `clients/flutter/rust-dependencies.json` are added as
post-upgrade artifacts. The repository's ordinary `renovate.json` remains auto-discovered and the
existing `RENOVATE_TOKEN` remains in use. This policy activates on the next scheduled or manually
dispatched Renovate run **after the PR is merged**.

Version changes and removed dependencies are refreshed automatically when all remaining package
identities, licence declarations and text hashes match reviewed records. Both candidates are
validated before either inventory is changed. New packages, changed material/repository identities,
supplements and changed Rust runtime notices stop for review. The existing committed-inventory
checks, source hash checks and licence/advisory gates remain enabled.

For a review stop, inspect the exact locked upstream records and licence text. With the pinned
tools prepared, resolve `flutter pub get --enforce-lockfile` in `clients/flutter` and
`cargo fetch --locked` in the repository root. Only after reviewing the reported change, use the
explicit approval commands from the root:

```sh
python3 scripts/verify_mobile_packages.py --write --audit
python3 scripts/native_notices.py --write
```

Review the resulting diff and rerun the repository gates. Do not request a Renovate rebase on a
manually repaired branch; an explicit rebase discards manual edits. Ordinary version-only updates
need no manual repair after this policy is active. Regression tests run in CI and locally with
`python3 -m unittest discover -s tests -p 'test_licence_inventories.py'`.

## Documentation
`src/docs.rs` embeds the Markdown sources as rustdoc modules. Run `cargo doc --no-deps` and
open `target/doc/mealshop_server/docs/index.html`. The Docs workflow adds a root landing
page. After adding a spec or ADR, update its index; docs generation discovers Markdown files.

## MCP interoperability

After installing the pinned Nextcloud development dependencies, check the committed tool schema with
`node scripts/generate_mcp_schema.mjs --check`, run
`python3 -m unittest discover -s tests -p 'test_mcp_stdio.py'`, then
`node scripts/test_mcp_client.mjs target/debug/mealshop-server` after `cargo build --locked`.
The latter starts isolated synthetic HTTP/stdio fixtures and uses the official MCP client. CI also
runs these checks against its compiled debug artifact; the SDK never enters shipped browser assets.
The `ExApp` container smoke test covers both rootless TCP and Unix-socket MCP adapters.
