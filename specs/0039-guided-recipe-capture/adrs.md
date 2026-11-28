# Decisions

See [ADR 0038](../../docs/adr/0038-guided-capture-and-household-unit-weights.md).

- Text capture is a client-side deterministic parser (`recipe-text.ts`); it adds no endpoint and
  sends nothing anywhere. Its output is a draft for the existing editor, not a saved recipe.
- Picture capture reuses F16/F35 `mode: photo` with `manual_transcription`; no new engine value.
- Ratings stay in F11 history entries. `GET /api/household/recipe-ratings` only counts reported
  values per recipe; it does not expose or depend on learned state.
- The nutrition overview is presentation only. Rounding happens in the client for five tiles; the
  server calculation, exact strings and range rules are untouched.

## Customer decisions, 2026-11-28
- **F39-AI: Nextcloud Task Processing.** AI structuring will be called by the server on behalf of
  the user through Nextcloud, so the instance admin picks the model and Mealshop stores no key.
  Not implemented: the local test instance has no provider, and which task types (text, speech,
  vision) the customer's instance offers is unverified.
- **F39-INBOX: suggestion inbox plus customer-operated companion.** Suggestions live in
  `suggestions.json` per household (schema 1, at most 50, 64 KiB each, replaced atomically).
  The only write path is the MCP tool `recipe_suggest` (creation scope); the client lists and
  discards through `GET/DELETE /api/household/suggestions`. Review reuses the import preview with
  `mode: suggestion`, which stages the suggested wording as a `manual` import of its source URL, so
  N06 source-link-only sharing and import evidence apply unchanged. A suggestion without a source
  URL becomes an owned recipe. The inbox is not an activity log and keeps no history.
- **Instagram.** The server does not fetch reels (login walls, robots, terms). The customer's own
  pipeline (modelled on `reelcatch`: saved collection → download → local transcript → summary)
  pipes JSON lines into `scripts/suggest_recipe.py`. Mealshop ships no scraper.
- **F39-WEIGHTS: general estimates, clearly marked.** The only general estimate offered is water
  density for volume units and metric spoons (1 ml = 1 g, 5/15 ml). It is stored like a household
  declaration with version prefix `household estimate` and an uncertainty text naming the
  assumption. No per-ingredient densities or piece weights are invented; those stay
  source-backed reference portions or the household's own value.
