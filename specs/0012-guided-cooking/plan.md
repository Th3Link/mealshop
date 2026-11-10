# Combined F12/F13 implementation plan

- [x] Read existing acceptance contracts and recipe/part/auth boundaries; record ADR 0016 and UX notes.
- [x] Implement exact UTC/civil-time resolution, dependency/window planning and immutable source references.
- [x] Implement protected durable instances, independent timers, revision guards and reviewed apply.
- [x] Deliver native cooking focus/list views and recipe entry points with exact scaled ingredients.
- [x] Add multi-day stage/cycle/equipment planning, target/delay previews and explicit uncertainty.
- [x] Add opt-in Nextcloud notification worker, persisted attempts, acknowledgements and recovery states.
- [x] Verify API failures/roles/restarts, DST transitions, notification ambiguity, native browser UX and cleanup.
- [x] Run the repository gate and rootless image checks; record AC evidence. PR CI remains a required merge check.

Each criterion retains its original scope. Customer data is never a test write target; synthetic household
IDs are recorded and archived after verification. No standalone/Flutter completion is claimed.
