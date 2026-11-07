# Nutrition workbench UX

Scope: a small responsive browser review surface; the planned Flutter app remains F18.

- Header identifies Mealshop and the ingredient/nutrition workbench, with a short limits notice.
- Ingredient search filters by canonical name and aliases. Detail shows form, basis, source,
  version, uncertainty and evidence conditions; missing values show “Unknown”, never 0.
- Recipe input uses labelled native controls for ingredient, grams, integer servings and
  preparation context; adding/removing ingredients is keyboard-accessible. No auto-save.
- Calculate is explicit. Controls disable only while a request is pending. Errors use a live
  region and retain the entered recipe; replacing results clears stale data on errors.
- Per-serving energy, protein, carbohydrate and fat appear prominently. A full table compares
  recipe/portion values and units. Partial rows label known subtotals and name missing ingredients.
- Source uncertainty and unmodelled cooking are shown next to results; education never asserts
  a dose is met without measured component data. Source links open safely as HTTPS links.
- Narrow screens stack controls/cards; tables scroll within their own container. Use semantic
  headings, native buttons/labels, visible focus, readable contrast and status text beyond colour.
- Empty catalogues show a clear message; failed requests offer retry without destroying input.
