# Flutter client decisions

[ADR 0026](../../docs/adr/0026-flutter-phone-delivery.md) accepts five complete device-testable PRs,
Nextcloud-first login/transport and a bounded shopping offline/replay boundary. Status: Accepted.
The customer selected Android, requested an emulator and explicitly gave implementation Go.
[ADR 0027](../../docs/adr/0027-mobile-shopping-receipts.md) specifies P1 credentials, dependencies,
protected cache, bounded request receipts and upgrade/downgrade semantics.

Existing decisions remain authoritative: stack (0001), quantities (0002), persistence boundaries (0003),
licensing/imported sharing (0004), privacy (0005), clients/localization (0007), household authority (0012),
durable civil-time cooking (0016), preparation (0017), nutrition ranges (0019), sharing (0020), feedback
(0021), scoped MCP (0022), reviewed import (0023), inventory (0024) and brewing (0025).

P1 uses focused `ChangeNotifier` state, official Material controls and a bounded protected record;
OCR, native reminders and broader database choices remain decisions of their later slices. F18's unimplemented storage adapters are not silently replaced by a phone cache.

## Source decision reconciliation

| Original `DECISIONS.md` question | Current finding / proposed action |
| --- | --- |
| Stack/folder | Flutter was already selected; propose new `clients/flutter/` (P1). No current mobile source. |
| Nextcloud/standalone API | Shared Rust domain routes exist; mobile Nextcloud login/proxy writes need proof in P1. Standalone identity remains F18/F30 scope. |
| Roles/children/health | Household owner/editor/reader exists. Persons are not accounts; new sensitive/child-profile projections and retention require approval before P2 collection. |
| Separate personal plans | Propose shared household plan with explicit per-person participation/alternatives; retain independent-plan extension. |
| Pantry scope | F15 lots/transactions already exist. P3 ports them and adds explicit physical prepared-batch allocation; do not start a conflicting generic pantry. |
| Product/nutrition sources | Existing frozen licensed catalogue and consented product lookup are reused. Exact identity, uncertainty and source-only import rules remain. |
| OCR/AI | No provider selected; propose on-device candidates in P5. External upload needs explicit separate authorization. |
| Offline/notifications | New mobile offline protocol required. P1 shopping receipts; P4 cooking reconciliation/local notification integration. No assumed mobile push provider. |
| Incomplete nutrition | F34 already supports explicit bounded comparisons/unknowns; port its exact evidence semantics, never replace missing data with zero. |
