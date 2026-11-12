# F15 delivery plan

1. Extend the existing atomic shopping state with exact lots, immutable movement/operation receipts,
   held/consumed/released source allocations and manually recorded offers. Test durable validation,
   dimensions, insufficient stock, atomic failures, reversal and idempotent restart first.
2. Wire authenticated read/preview/confirm routes, current catalogue/profiles/completed cooking and
   shopping receipts; derive purchase reduction and preserve source facts under edits/check-off.
3. Deliver native inventory and offer views plus shopping/cooking entry points, typed contracts,
   review/retry/discard and keyed EN/DE messages.
4. Verify every criterion in isolated backend and real Nextcloud owner/reader flows, screenshots,
   responsive/accessibility checks, installed restart/rootless transports; clean only owned synthetic
   households, run mandatory gates and complete one PR against main.

- [x] Read requirements/spec and existing shopping, quantity, planning, cooking and role boundaries.
- [x] Record the delivery contract, UX and atomicity decision.
- [ ] AC 1: exact lots, metadata/corrections/archive and durable source records.
- [ ] AC 2: reviewed reservations and explicit remaining purchase projection.
- [ ] AC 3: purchase/completed-cooking booking, reversal and idempotent recovery.
- [ ] AC 4: manual offers, package/price/source/validity and current dietary constraints.
- [ ] Native integration, all gates, installed evidence, synthetic cleanup and exact-head CI.
