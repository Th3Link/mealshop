# F17 delivery plan

1. Canonical creation preview and atomic recipe receipt: model → file publication → retry/restart tests.
2. Disabled-by-default account scopes and bounded MCP protocol/tools: current membership → canonical
   recipe/ingredient adapters → source-only projection → permission/protocol tests.
3. Stdio connection and native setup/activity: credential handling → explicit grants → user flows.
4. Real Nextcloud/client/container verification, synthetic cleanup, full gates and one reviewable PR.

## Tasks
- [x] Read requirements/F17 and current source, quantity, recipe and authorization boundaries.
- [x] Record delivery contract, trust boundaries, native UX and primary protocol sources before code.
- [x] Implement canonical nonpersistent validation and atomic audited/idempotent creation.
- [x] Implement owner-managed scopes, current-role enforcement and bounded dual-era MCP tools.
- [x] Ship stdio bridge and native setup/activity with safe credentials and explicit confirmation.
- [x] Verify all acceptance criteria, browser/client/container/restart and cleanup; record evidence.
- [x] Run required gates and prepare one PR; verify exact-head remote CI before handoff.

AC 1: authenticated discovery/search/read/preview and real client flow.
AC 2: canonical validation parity, provenance restrictions and immutable creation audit.
AC 3: current role, explicit scope, revocation and household isolation tests.
AC 4: identical/concurrent/restarted retries, conflicting reuse and structured quantity errors.

## Delivery evidence

See [validation.md](validation.md) for the 142-test backend gate, real official-client/Nextcloud
connection, all 28 distinct browser scenarios across batches, installed restart and synthetic-only
cleanup. [PR #32](https://github.com/Th3Link/mealshop/pull/32) carries the final-head CI checks.
