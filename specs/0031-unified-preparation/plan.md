# Implementation and validation plan

- [x] Record approved contract and archive approved general/multipart screens (AC6).
- [x] Persist typed ingredient temperature extensions with exact value/provenance and legacy defaults; API negative/round-trip tests (AC4).
- [x] Expose extensions in native authoring/read/cooking and preserve edits (AC4).
- [x] Group native ingredient/step editing by recipe part, preserve references and drafts; client build/tests and browser check (AC3).
- [x] Add production/allocation persistence and atomic validation, production-once shopping and compatibility tests (AC1/2).
- [x] Display shared preparation/meal calendar and native detail form; verify mealprep and bread in browser (AC1/2).
- [x] Connect calendar anchors to durable cooking/reminder flow, preserve parallel starts and action-time timers; integration/browser checks (AC5).
- [x] Run full gate, accessibility/responsive checks, review diff and prepare reviewable PR; publish honest validation evidence (AC6).

Accepted Nextcloud scope is complete. See validation.md for API/browser acceptance, operational
checks and explicit limits. Remote CI remains a required merge check.
