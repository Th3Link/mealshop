# Guided meal plan with typed and spoken commands

Status: In progress
Priority: Must
Milestone: M2
Origin: Customer request, 2026-11-28.

## Objective
Planning meals in the Nextcloud client is quick and guided: the cook says or types what should go
on the plan and confirms the usual review. The week is calm to read and the planner stays in view.
F10's hard constraints and revision-checked apply are unchanged; slice 2 changes when the
acknowledgement is asked and how a missing duration is treated.

Customer wording (German, summarized): the meal plan "is not yet easy to use" and should be
reworked; "I would like voice control most of all". AI steps should work with small, low-cost
models, larger ones only where needed.

## Acceptance
Slice 1 (this delivery):
1. A command field above the week accepts one typed or spoken sentence in German or English,
   whatever the display language. It is read deterministically, without a model: action (plan,
   remove, suggest, show), day (today, tomorrow, the day after, weekdays, "next" weekday, ISO and
   `d.m.` dates), meal (breakfast, lunch, snack, dinner and their German words; dinner when none
   is named), servings ("for 4", "für zwei Personen") and the recipe by title.
2. A command never changes the plan. Planning and removing open the existing planner with the
   server preview already loaded; the cook acknowledges and applies as before. Suggesting opens
   the existing suggestions for that slot; showing only moves the week.
3. What was understood is repeated in words with the resolved date. A recipe is taken only when
   exactly one title fits; several are offered as choices, none is said plainly with the words
   heard. One wrong letter in a longer word is tolerated; nothing else is guessed.
4. A sentence that cannot be carried out is answered, not failed: nothing planned to remove,
   several meals on that day, no people in the household yet.
5. Speech uses the browser's speech recognition where it exists, with the same disclosure as
   recipe dictation; the typed field is the fallback and the tested path.
6. The planner is a panel that stays in view instead of a form below the week. The recipe is
   picked from a searchable list. Review times are shown as durations, not seconds.
7. The week marks today, shows planned meals as cards and keeps empty slots to one quiet line.
8. German/English, keyboard, axe (WCAG 2.2 AA tags), 390 px and 1440 px are verified in the
   embedded browser suite.

Slice 2 (customer decisions of 2026-11-28):
9. A review with nothing to flag is applied or removed with one button and no acknowledgement
   (F40-REVIEW: A). "Nothing to flag" means: feasible, no conflict or warning, and for the people
   eating no allergy, intolerance, allergen group, allergen match, dietary preset or exception,
   exclusion, dislike or unidentified ingredient. The two general notices (incomplete allergen
   coverage, cross-contamination not modelled) stay visible. Anything else keeps the
   acknowledgement.
10. A recipe without steps can be planned (F40-NOSTEPS: A). The preview then reports no timing,
    lists `preparation_time_unknown` under `warnings`, does not check the time budgets, and the
    review says the duration is unknown and requires the acknowledgement.

Later slices (not implemented):
11. Drag or move meals between slots, copy a week, voice confirmation, and free-form requests
    ("something quick with rice") through a small model via Nextcloud Task Processing.

## Out of scope
Changes to F10's server rules, automatic planning, autonomous purchases, the standalone and
Flutter clients, a wake word or always-on listening.

## Links
Requirement: [F40](../../docs/requirements.md). Delivery: [plan.md](plan.md). Decisions:
[adrs.md](adrs.md). UX: [ux-notes.md](ux-notes.md). Evidence: [validation.md](validation.md).
