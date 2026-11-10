# Implementation and validation plan

- [x] Record approved contract and archive approved general/multipart screens (AC6).
- [x] Persist typed ingredient temperature extensions with exact value/provenance and legacy defaults; API negative/round-trip tests (AC4).
- [x] Expose extensions in native authoring/read/cooking and preserve edits (AC4).
- [ ] Group native ingredient/step editing by recipe part, preserve references and drafts; client build/tests and browser check (AC3).
- [x] Add production/allocation persistence and atomic validation, production-once shopping and compatibility tests (AC1/2).
- [ ] Display shared preparation/meal calendar and native detail form; verify mealprep and bread in browser (AC1/2).
- [ ] Connect calendar anchors to durable cooking/reminder flow, preserve parallel starts and action-time timers; integration/browser checks (AC5).
- [ ] Run full gate, accessibility/responsive checks, review diff and prepare reviewable PR; publish honest validation evidence (AC6).

Status is intentionally In progress until all acceptance evidence exists. Device integration is outside this slice.

Native implementation is written, but the three browser-dependent tasks remain unchecked until the host can be exercised. See validation.md for exact evidence and blockers.

Local deployment: the matching final image passed isolated TCP/Unix smoke checks, then replaced the local test container after a complete private backup. Container health is green. Draft PR #27 is open; native browser acceptance remains the merge blocker.
