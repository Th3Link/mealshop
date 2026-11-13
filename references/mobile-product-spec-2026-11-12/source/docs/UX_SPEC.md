# MealShop — Mobile UX specification

Status: proposed. Target is a smartphone-first UI, not a shrunk desktop/Nextcloud screen.

## Navigation
Five bottom tabs:
1. **Today** — today's meals, prep reminders, cooking actions, outstanding shopping.
2. **Plan** — week/day calendar, participants, portions, meal swaps, generation and confirmation.
3. **Shopping** — shared fast-entry list, check-off, scan, categories and stores.
4. **Recipes** — browse, search, import, create, cook.
5. **More** — household, profiles, activity, settings, privacy.

Provide contextual `+` actions and quick scanner entry points. Avoid overcrowding the persistent navigation.

## First-run tour
Progressive and skippable. Save draft after every step; support adding additional people and later editing.

### Household setup
Name household; add first person; invite/add others (invitation mechanism to match existing auth); default cooking/shopping preferences. Do not require sensitive health information.

### Person profile steps
1. Name and optional avatar.
2. Dietary baseline: omnivore / vegetarian / vegan / pescetarian / flexitarian / custom.
3. Exceptions: allowed animal products and species; origin-specific rules such as eggs only from own hens. Detect and resolve conflicting selections.
4. Allergies, intolerances, absolute exclusions (hard); disliked and preferred ingredients/dishes (soft). Distinguish each visibly.
5. Goals: balanced, weight change, protein-focused, low-fat, low-carb, custom. Optional measurements; no mandatory calorie target.
6. Optional health conditions and clinician-specified rules. Do not infer exclusions automatically from diagnoses (e.g. Hashimoto).
7. Meal pattern: breakfast/lunch/dinner/snacks; recurring weekday attendance; time windows including 16:8, 18:6 or custom; per-date exceptions.
8. Cooking pattern: fresh / batch prep / mix, separately by meal; prep batch days (e.g. lunch for 2–4 days), leftovers, fridge/freezer constraints.
9. Summary of derived rules, editable before first plan generation.

### Onboarding behavior
- Show 1–2 questions per screen; clearly display progress.
- Provide 'skip for now' for optional questions.
- Show plain-language rule summary; never hide assumptions.
- Let users return to any step from their profile.
- Ask only essential questions before first usable plan; defer details contextually.

## Today screen
Top: household selector, notifications. Main: chronological meal cards with participants, status (planned/prepared/ready), cooking time and next action. Secondary: shopping count and upcoming prep tasks. Empty state offers create plan or add meal.

## Plan screen
Week strip and selected-day meal cards. Each card: dish, meal type, participants, portions, prep/batch status, time, relevant warnings. Actions: swap dish, edit participants, adjust servings, mark prepared, confirm. A plan can be partly confirmed. Show resulting shopping changes before applying disruptive edits.

## Shopping screen
Fast text add and autocomplete; large checkable item tiles/rows; quantity, unit, product preference, category, store; checked section; barcode shortcut; shared sync status and offline queue. Do not silently delete checked or manually entered items when plans change.

## Recipes screen
Library and search; add via camera, URL/share sheet, barcode/product, manual entry. Import review shows source, parsed steps, ingredient mapping, missing amounts, uncertain nutrition. Support recipe scaling, subrecipes, buy-vs-make choice and cooking launch.

## Cooking mode
Large readable step, ingredients for current step, previous/next, independent timers, background-safe notifications and preparation tasks spanning days. Clear pause/cancel/recover behavior. No requirement for always-on network during an active cooking session.

## Activity screen
One-tap activity type, duration (15-min increments by default), optional intensity, save. Keep estimates explicitly approximate and separate from meal-plan hard constraints.

## Interaction rules and states
- Every critical action needs loading, empty, error, offline and conflict behavior.
- Shopping check-off should be optimistic with eventual synchronization and visible recovery.
- All dietary conflict warnings must be explicit and actionable.
- Respect font scaling, screen readers, contrast, touch target sizes, keyboard navigation where applicable, and localization (German first; i18n-ready).
- Never convey allergies or safety state by color alone.

## Key user journeys / acceptance examples
1. Vegan user allowing only own eggs + omnivore household member: generate shared dinner or explicitly explain why no safe shared recipe is possible.
2. Change Thursday dinner and participants: recalculate only relevant ingredient contributions; preserve manual shopping entries and purchased items.
3. Cook 6 portions Sunday; assign 4 to Monday/Tuesday and freeze 2; avoid purchasing ingredients again for those portions.
4. Import photo recipe with missing '1 cup' size: mark amount unresolved, do not fabricate nutrition.
5. Add shopping item offline on device A, check it on device B after sync; handle conflicts deterministically.
