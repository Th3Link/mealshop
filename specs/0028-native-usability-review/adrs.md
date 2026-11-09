# Decisions

- Reuse Vue Router guards and the browser's native before-unload protection through one small shared
  composable, so household switches and browser exit have the same draft protection as app navigation.
- Keep recipe search in the route query; filter existing loaded titles without additional API calls.
- Keep structured intolerance IDs out of editable free-text notes, merging those notes with the
  selected canonical IDs at the existing API boundary. No backend/schema changes.
- Keep timing inputs in their explicit existing units; do not introduce rounding or hidden conversion.
