# Guided recipe capture, ratings and readable nutrition

Status: In progress
Priority: Must
Milestone: M2
Origin: Customer request, 2026-11-27.

## Objective
Adding a recipe in the Nextcloud client is guided and pleasant instead of form-driven: the cook
brings text, speech, a picture or a link, reviews once and saves. Recipes show household ratings,
and nutrition is readable at a glance while exact values, sources and unknowns stay inspectable.
AI-assisted capture reuses the same review step; no provider is chosen by this slice.

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

Later slices (not implemented, see plan and open customer decisions):
11. AI structuring of text, speech and pictures into the same review step.
12. A suggestion inbox that assistants (MCP) and a customer-operated Instagram companion fill with
    unreviewed candidates.
13. Nutrition calculation gaps: wider reference-portion coverage, cooking yield/retention and
    unresolved quantities that keep a recipe saveable with explicitly incomplete nutrition.

## Out of scope
Server-side scraping of Instagram, automatic ingredient substitution, health claims, a redesign of
planning/shopping/inventory views, the standalone and Flutter clients.
