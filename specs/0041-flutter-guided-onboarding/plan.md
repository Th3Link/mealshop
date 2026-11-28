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

## Slice 3 — goals and calibration (AC8–10; needs FL-GOALS, FL-STARTER)

- [ ] Versioned declared-preference extension in the core (ADR), planner prior, explanations.
- [ ] Calibration UI, repeat from the person page, offline evaluation against the F11 baseline.

## Slice 4 — consent, privacy, dictation hook (AC11–12, AC7)

- [ ] Consent records, privacy summary, revoke/export/erase paths, backup round trip.
- [ ] Dictation button wired to the on-device recogniser delivered by F42.

## Open customer decisions

Frage 1 (FL-STARTER): source of the dishes shown for calibration.
A: a small bundled starter set written for Mealshop (own content, exact quantities, usable as real
recipes afterwards). B: dish cards only (title, tags, main ingredients; no recipe) until the public
exchange can deliver recipes. C: only the user's own recipes, so calibration appears later.
Recommendation: B for the first calibration slice, A as a later content task.
Default if unanswered: B.

Frage 2 (FL-GOALS): weight and target weight.
A: coarse goal only, no numbers. B: coarse goal plus optional weight/target weight, on-device only,
never synced or shared, separately erasable. C: also in connected Nextcloud mode.
Recommendation: B. Default if unanswered: A.

## Verification

Slice 1, 2026-11-28, pinned Flutter 3.50.0: `flutter analyze` without findings, `flutter test`
88 passed (6 new in `test/device_home_test.dart`: automatic creation exactly once, reuse of an
existing household, legacy switcher, hidden row/name in More, retry instead of household setup).
Repository gate (catalogue import/verify, i18n, fmt, Clippy, 212 nextest tests, licences, audit,
generated docs check) passed. The emulator integration test `app_only_test.dart` was adapted but
not run; fresh-install and upgrade evidence on the emulator is open and is not inferred.
