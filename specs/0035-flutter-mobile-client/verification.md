# Planning PR verification

Scope: imported source documents, source-based capability audit, proposed delivery/UX/ADR and navigation
links. No Flutter application, new API behaviour, SDK setup, device build or mobile runtime acceptance
is claimed. Existing test-instance data and deployment configuration were not accessed or changed.

## Import and consistency checks

- The seven extracted files match the customer ZIP byte for byte; `manifest.json` records each length
  and SHA-256 plus the original archive hash. Only the agent reference filename changes.
- Root `AGENTS.md` matches its pre-import checksum. No nested active `AGENTS.md` is imported.
- Relative links in the changed planning/index/reference documents resolve locally.
- Current-state claims use actual Rust routes/models and current Nextcloud/MCP client transport code.
  Required mobile capabilities are explicitly proposed rather than labelled implemented.
- F35 and ADR 0026 remain Draft/Proposed. Existing F18/F29/F30 requirements retain their status and
  unchecked work. The client folder does not exist; all implementation tasks are still unchecked.
- The only executable-source change is generated rustdoc metadata. No Flutter/domain implementation,
  pipeline change or test-instance mutation is included.

## Mandatory security-gate maintenance

The initial `cargo audit` refreshed the advisory database and found three issues in the pre-existing
lockfile: `RUSTSEC-2026-0523` and `RUSTSEC-2026-0524` in `quinn-proto 0.11.18`, and
`RUSTSEC-2026-0538` in `quinn-udp 0.5.15`. The official `RustSec` records specify patched versions
`quinn-proto >=0.11.19` and `quinn-udp >=0.5.16,<0.6.0` (or the separate newer minor branch).

Use a separate maintenance commit for the minimal generated lockfile updates to **0.11.19 / 0.5.16**.
Cargo also refreshes existing Windows-target dependency edges; no direct manifest, feature selection,
new crate, advisory ignore or licence exception is added. `cargo tree --locked -i quinn-proto` reports
no active reverse dependency for the current host/features; that does not waive the whole-lock audit.
No application or phone behaviour is inferred from a passing security gate.

Primary records were read from the refreshed official advisory checkout (origin
`https://github.com/RustSec/advisory-db.git`):

- [RUSTSEC-2026-0523](https://github.com/RustSec/advisory-db/blob/main/crates/quinn-proto/RUSTSEC-2026-0523.md)
- [RUSTSEC-2026-0524](https://github.com/RustSec/advisory-db/blob/main/crates/quinn-proto/RUSTSEC-2026-0524.md)
- [RUSTSEC-2026-0538](https://github.com/RustSec/advisory-db/blob/main/crates/quinn-udp/RUSTSEC-2026-0538.md)

## Publication checks

Run the complete required repository gate after the patch updates, plus generated-doc synchronization,
strict rustdoc and the import/link/status checks above. Record actual command/results in the PR; final
head CI must pass before handoff. A passing Rust/docs pipeline does not establish Flutter functionality.
Physical-phone build, installation, two-client offline and OS notification proof belong to P1–P5.
