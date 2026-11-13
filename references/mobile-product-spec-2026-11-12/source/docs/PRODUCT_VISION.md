# MealShop — Product vision

Status: proposed product specification, 2026-11-12. This is a **target state**, not a verified inventory of the repository.

## One-sentence vision
MealShop is a household-first assistant that turns people's dietary needs, preferences, schedules, recipes and available ingredients into a practical meal plan, shopping list and cooking workflow.

## Core loop
1. Create household and add people.
2. Build each person's food profile in a progressive guided tour.
3. Configure recurring meal attendance, meal windows and cooking/meal-prep habits.
4. Request a server-generated meal plan; review and adjust it.
5. Convert planned dishes into a traceable shopping list, including subrecipes and homemade ingredients.
6. Shop together, prepare and cook with reminders/timers.
7. Log lightweight activity and feedback to improve future plans.

## Primary users and jobs
- Household organizer: make a plan that fits everyone, coordinate shopping and preparation.
- Household member: communicate food rules and availability, see what's planned, add shopping items.
- Solo user: enjoy the same features without household complexity.

## Product principles
- Household-first by default, with independent individual profiles and extensibility for separate plans.
- Explainable suggestions, not opaque automation; user retains control.
- Fast daily flows; complexity is progressive, not front-loaded.
- Shopping list is a complete standalone feature, not just a meal-plan export.
- Unknown nutrition or imported recipe quantities remain unknown until reviewed.
- Health goals are optional and nonjudgmental; medical rules require explicit input.
- Nextcloud and standalone server should expose equivalent domain behavior.

## Major capabilities
### Household and onboarding
Household membership and roles; person profiles; diet templates (omnivore, vegetarian, vegan, pescetarian, flexitarian, custom); individual allowed/forbidden animal products and provenance conditions (e.g., own eggs only); allergies and intolerances; favorites/dislikes; goals (maintain/lose/gain weight, low-fat, low-carb, high-protein, balanced, custom); optional medically informed constraints; eating windows (16:8, 18:6, custom); meal attendance per weekday and exceptions; meal-prep strategy per meal; cooking-time and storage preferences.

### Meal planning
Server-generated proposals respecting hard constraints and optimizing soft preferences, attendance, time, ingredient reuse, cost/effort where data exists, nutrition where reliable, and batch cooking. Swap dishes, change participants, adjust servings, confirm individual slots. Explain recommendations and infeasibility. Never silently override allergies or explicit exclusions.

### Shopping and pantry
Fast manual entries, shared live list, offline operations, categories/stores, quantities and units, checked states, generic ingredients vs concrete barcode products, provenance of each required amount, careful diff when meal plan changes, optional pantry and homemade vs buy decision.

### Recipes and capture
Manual recipe editing, OCR from images, website/share-sheet import, legally accessible video/reel analysis when technically possible, barcode scanning and nutrition label recognition. Human review and confidence/unknown indicators. Nested subrecipes and ingredient preparation.

### Cooking assistant
Step-by-step large-format cooking mode, multiple timers, durable scheduled prep tasks, multi-day baking/fermentation workflows, reminders and later optional smart-scale integration.

### Activity
Lightweight activity type, duration, optional intensity, approximate energy expenditure only when enough information is available; no requirement to count calories or weigh oneself.

## Non-goals for first release
- Clinical nutrition treatment or automated disease-specific diets.
- Full fitness wearable integration.
- Automatic download of restricted third-party video content.
- Smart-scale hardware integrations.
- Fully autonomous purchasing.

## Success criteria
A household can onboard at least two people with different diets, generate/review a week plan, swap one meal, produce a correct editable shopping list, shop with two devices, and cook a meal with timed prep — without having to manually reconcile quantities or violate dietary exclusions.

## Open product decision
Default to a shared household meal plan with person-specific portions/alternatives. Retain the option of fully individual plans as a later extension. Confirm with product owner before committing to irreversible data structures.
