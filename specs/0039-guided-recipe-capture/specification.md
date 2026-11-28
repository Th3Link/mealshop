# Guided recipe capture, ratings and readable nutrition

Status: In progress
Priority: Must
Milestone: M2
Origin: Customer request, 2026-11-27.

## Objective
Adding a recipe in the Nextcloud client is guided and pleasant instead of form-driven: the cook
brings text, speech, a picture or a link, reviews once and saves. Recipes show household ratings,
and nutrition is readable at a glance while exact values, sources and unknowns stay inspectable.
Assistants and customer-operated tools hand recipes in as suggestions that go through the same
review step. AI structuring inside Mealshop will use Nextcloud Task Processing (customer decision
2026-11-28) and is not implemented yet.

Customer wording (German, summarized): the current UI "feels like phpMyAdmin"; "I do not want to
enter a hundred things"; "adding recipes should be a treat"; recipes need ratings; nutrition
calculation is not yet optimal; check MCP, dictation, screenshots/photos and Instagram as AI inputs.

## Acceptance
Slice 1 (this delivery):
1. One **Add recipe** entry offers paste/dictate, photo or screenshot, website link and an empty
   form. The existing link import and editor remain reachable and unchanged in their rules.
2. Pasted, typed or dictated text is read deterministically into title, servings, ingredient lines
   and steps with a live preview. Amounts become numbers only when they are exact decimals of at
   most six fractional digits; ranges, repeating fractions and ambiguous separators stay unresolved.
   Units are recognised, never converted. The original line stays visible during review.
3. An ingredient is identified automatically only by exactly one exact name or alias match.
   Otherwise the cook picks from offered candidates or the catalogue; nothing nearby is substituted.
4. A picture is stored privately, shown next to the text and saved through the existing reviewed
   photo import (`captured` origin, private source when shared). No server OCR or AI is implied.
5. Dictation uses the browser's speech recognition where available, states who processes the audio
   and degrades to typing elsewhere.
6. Volume/count quantities no longer require filing evidence: one bundled reference portion is
   attached automatically when exactly one fits; otherwise the cook enters what one unit weighs.
   That household-declared weight is stored with its unit, date and an explicit "not a source
   record" uncertainty, and is labelled in the recipe. Declaring a source record remains possible.
7. Recipe list and recipe page show the average reported household rating (person ratings when no
   household rating exists) and how often it was cooked, independent of opt-in learning. A recipe
   can be rated from its page; this records a completed cook in the existing history.
8. Recipes are shown as cards with image, rating and sort order (name, rating, recently cooked).
9. Nutrition opens with rounded per-serving energy, protein, carbohydrate, fat and fibre. Partial
   sums are labelled as a lower bound, ranges as estimated, unknown is never shown as zero. The
   exact table, sources and uncertainty remain one disclosure away and unchanged.
10. German/English, keyboard, axe (WCAG 2.2 AA tags), narrow and wide layouts are verified in the
    embedded Nextcloud client.

Slice 2 (suggestion inbox and accepted estimates; customer decisions F39-INBOX A, F39-WEIGHTS B):
11. An assistant with the creation scope can hand in an unreviewed recipe (`recipe_suggest`): title,
    original ingredient lines and steps, optional HTTPS source URL and note, without ingredient IDs.
    It is stored as a suggestion, never as a recipe, is never calculated and repeats are recognised.
    At most 50 suggestions wait per household; readers can neither add nor see them.
12. **Add recipe** lists waiting suggestions with date, line count and source host. Reviewing one
    opens the same review as typed text. A suggestion that names a source URL is saved as an import
    of that source (source-link-only sharing, original wording retained as import evidence); one
    without is the household's own recipe. Saving removes the suggestion; discarding needs a second
    click. The recipe list shows how many suggestions wait.
13. A customer-operated companion can submit suggestions from its own pipeline with
    `scripts/suggest_recipe.py` (one JSON object per line). Mealshop fetches nothing from
    Instagram or any other login-walled source.
14. For millilitres, litres and metric tea/tablespoons the cook may accept a general estimate
    "as if it weighed like water" instead of typing a weight. It is offered, never applied
    silently, is stored and labelled as a rough estimate with its assumption, and typing a value
    replaces it. Pieces have no general weight and get no estimate.

Slice 4 (nutrition gaps, first part; ADR 0039):
16. A line in millilitres, litres, spoons or pieces may be saved without a weight. Nothing is
    derived: the original quantity is kept, scaled and shown, the line is marked **weight
    unknown**, and nutrition leaves it out. Every affected nutrient reports no total, keeps the
    known part as "at least" and names the ingredient; the overview says in one sentence which
    ingredients have no weight. A stated mass next to an unweighed quantity is rejected.
17. Accepted rough estimates and household-declared weights are carried into nutrition: warning
    codes `rough_mass_estimate` and `household_mass`, a `mass_basis` per ingredient, and one
    sentence each in the overview naming the ingredients.
18. An unweighed line reaches the shopping list in its own unit (millilitres, pieces; spoons as
    millilitres of the metric 5/15 ml spoon) and never as grams. Meal-plan and inventory mass
    arithmetic leave it out. Cooking shows its quantity for the chosen servings.

Later slices (not implemented):
15. AI structuring of text, speech and pictures through Nextcloud Task Processing into the same
    review step, after the task types available on the customer's instance are known. Small,
    low-cost models are the target (customer preference, 2026-11-28): deterministic reading
    first, short constrained tasks, a larger model only as an explicit fallback.
19. Wider source-backed reference portions including pieces, and a cooking yield/retention
    model. Both need additional primary source data that is not in the repository.

## Out of scope
Server-side scraping of Instagram, automatic ingredient substitution, health claims, a redesign of
planning/shopping/inventory views, the standalone and Flutter clients.
