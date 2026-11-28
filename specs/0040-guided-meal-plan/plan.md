# Delivery plan

## Slice 1 — command field, visible planner, calmer week
- [x] Read F10, the planner, the calendar and the browser specs that depend on them.
- [x] Deterministic sentence reader with unit tests (AC 1, 3).
- [x] Command field with speech input, answers and recipe choices (AC 2–5).
- [x] Planner as a fixed panel, searchable recipe picker, readable review times (AC 6).
- [x] Week styling: today, planned cards, quiet empty slots (AC 7).
- [x] Embedded browser verification (AC 8); see validation.
- [ ] Real-microphone check in Chromium and Firefox — not testable headless.
- [ ] Customer review on the customer's instance.

## Slice 2 — customer decisions of 2026-11-28
- [x] One-button apply/remove when the review has nothing to flag (AC 9).
- [x] Recipes without steps plannable with an unknown-duration warning (AC 10).
- [ ] Show the unknown-duration warning in the Flutter client.

## Later
- [ ] Move/copy meals, week templates.
- [ ] Free-form requests through a small model via Nextcloud Task Processing, after the task
  types on the customer's instance are known.

Merge gate: remote CI must pass on the exact PR head. This branch is stacked on
`story/guided-recipe-capture` (F39).
