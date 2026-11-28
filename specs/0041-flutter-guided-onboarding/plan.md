# Implementation and verification plan

Branch `story/flutter-guided-onboarding`. One usable slice per PR.

## Slice 1 — invisible household (AC1–3)

- [x] Create and select the internal household automatically for an empty on-device dataset.
- [x] Remove household row, switcher and creation from on-device Home/More; keep the switcher
      only for legacy datasets with several households. Reword on-device texts.
- [x] Widget/controller tests for automatic creation, hidden UI and the legacy case.
- [ ] Emulator check of a fresh install and of an upgrade with an existing named household.

## Slice 2 — start assistant and people (AC4–7)

- [ ] Assistant shell with resumable protected draft, skip, reopen from More.
- [ ] Rework the person tour into one-topic screens with selectable options; mark the main user
      (compatible profile field, Rust core + Vue contract in the same PR).
- [ ] Tests: interruption/relaunch, skip paths, name-only person, DE/EN, large text.

## Slice 3 — goals and calibration (AC8–10)

- [ ] Health-data ADR, private goal/weight record with grants in the core, Nextcloud contract.
- [ ] Versioned declared-preference extension in the core (ADR), planner prior, explanations.
- [ ] Starter recipe client against the recipe server API (blocked until that API exists).
- [ ] Calibration UI, repeat from the person page, offline evaluation against the F11 baseline.

## Slice 4 — consent, privacy, dictation hook (AC11–12, AC7)

- [ ] Consent records, privacy summary, revoke/export/erase paths, backup round trip.
- [ ] Dictation button wired to the on-device recogniser delivered by F42.

## Customer decisions

Answered 2026-11-28: FL-STARTER — real starter recipes from the online recipe server, not bundled
(adrs D5); FL-GOALS — C (adrs D4). Calibration is blocked until the recipe server API exists.

## Verification

Slice 1, 2026-11-28, pinned Flutter 3.50.0: `flutter analyze` without findings, `flutter test`
88 passed (6 new in `test/device_home_test.dart`: automatic creation exactly once, reuse of an
existing household, legacy switcher, hidden row/name in More, retry instead of household setup).
Repository gate (catalogue import/verify, i18n, fmt, Clippy, 212 nextest tests, licences, audit,
generated docs check) passed. The emulator integration test `app_only_test.dart` was adapted but
not run; fresh-install and upgrade evidence on the emulator is open and is not inferred.
