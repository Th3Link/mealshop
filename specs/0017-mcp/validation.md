# F17 validation evidence

Status: In progress. Final installed checks and remote CI are pending.

- `cargo nextest run --workspace --locked`: 140/140 passed (local test sockets allowed).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- Official MCP SDK 2.4.0: modern HTTP and auto/legacy/pinned-modern stdio passed against isolated
  synthetic state; exact quantities, confirmed creation/retry, audit, retroactive source-only reads
  and scope reduction are asserted.
- Python bridge unit suite: 5/5 passed; transport bounds, credential handling, redirect rejection,
  unusable upstream responses and invalid input are covered.
- Browser/installed/container evidence will be completed before Done. Only ledger-owned synthetic
  households are used. No retained household directory is inspected or copied.
