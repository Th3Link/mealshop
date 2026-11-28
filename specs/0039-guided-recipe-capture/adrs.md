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
- **Assistant transcription (2026-11-28, ADR 0040).** The customer's instance offers text,
  speech-to-text, picture and handwriting tasks. Mealshop requests words only
  (`core:audio2text`, `core:image2text:ocr`, `core:analyze-images` for handwriting) and keeps
  structuring deterministic. File inputs are Nextcloud file IDs, so a temporary copy lives in
  `.mealshop-capture/` of the user's files while the task runs. Probed on the local Nextcloud
  35: app authentication works for the OCS task routes and file, `OC-FileId` carries the
  file ID, and scheduling without a provider answers 412.
- **Low-cost models first (2026-11-28).** The customer prefers AI steps that work with small,
  cheap models and wants larger models only where needed. Deterministic reading stays the first
  step; AI tasks are to be short, constrained and structured, with a larger model as a fallback.
- **Unweighed quantities (2026-11-28, ADR 0039).** Continuing with the nutrition gaps, a
  non-mass line may be saved without a weight. The mass is optional in the model and unknown is
  carried explicitly; nothing is defaulted.
- **F39-WEIGHTS: general estimates, clearly marked.** The only general estimate offered is water
  density for volume units and metric spoons (1 ml = 1 g, 5/15 ml). It is stored like a household
  declaration with version prefix `household estimate` and an uncertainty text naming the
  assumption. No per-ingredient densities or piece weights are invented; those stay
  source-backed reference portions or the household's own value.
