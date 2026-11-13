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
- [x] AC 1: exact lots, metadata/corrections/archive and durable source records.
- [x] AC 2: reviewed reservations and explicit remaining purchase projection.
- [x] AC 3: purchase/completed-cooking booking, reversal and idempotent recovery.
- [x] AC 4: manual offers, package/price/source/validity and current dietary constraints.
- [x] Native integration, local gates, installed transport/restart evidence and synthetic cleanup.

Current-head CI is an external handoff gate: all eleven PR checks must pass before the PR is marked
ready and handed back. Its authoritative record is the PR Checks tab.

Implementation uses no new dependencies and remains within the existing shopping transaction. The
review corrected movement pagination, preserved reviews across locale-only quantity changes and
guarded delayed booking/refresh responses. See [validation.md](validation.md) for exact evidence.
