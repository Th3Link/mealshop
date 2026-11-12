# F16 delivery plan

1. Safe public retrieval and structured extraction: limits/DNS/robots/redirect tests before UI wiring.
2. Actor-bound previews and immutable reviewed import receipts: canonical publication, provenance,
   idempotency/body conflicts, current household roles and source-only downstream projection tests.
3. Native source entry/manual recovery and existing editor review; keyed contracts and draft protection.
4. Real public-source/manual/SSRF browser checks, rootless transports, installed restart and own-fixture
   cleanup; required gates and one PR with exact-head CI.

- [x] Read requirements, F16 and existing recipe/quantity/provenance/authentication boundaries.
- [x] Record threat model, contract, native UX and dependency rationale in ADR 0023.
- [x] Implement/test bounded fetching and extraction (AC 1, 4, 5).
- [x] Implement/test canonical reviewed publication and persistent source restrictions (AC 2, 3).
- [x] Deliver/test the native complete review and manual fallback (AC 1, 2, 5).
- [x] Verify real installation/restart, all gates and final CI; record evidence and cleanup (AC 1–5).

See [validation evidence](validation.md) for acceptance mapping, the installed image, all 31 native
browser scenarios, restart/cleanup and exact current-head PR checks.
