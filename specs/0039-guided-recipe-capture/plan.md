# Delivery plan

## Slice 1 — guided capture, ratings, readable nutrition
- [x] Review the embedded client, F16/F17/F33/F34/F35 boundaries and the customer's `reelcatch`.
- [x] Deterministic text reader with unit tests (AC 2, 3).
- [x] Add-recipe chooser, text/dictation and picture capture feeding the existing review (AC 1–5).
- [x] Household-declared unit weight in server validation, client and MCP schema (AC 6).
- [x] Rating summary endpoint, stars on cards/page, rate-from-recipe (AC 7, 8).
- [x] Nutrition overview tiles with exact details behind a disclosure (AC 9).
- [x] Embedded browser verification for the new flow (AC 10); see validation.
- [ ] Real-microphone dictation check in Chromium and Firefox (AC 5) — not testable headless.
- [ ] Customer review of the new screens on the customer's instance.

## Slice 2 — suggestion inbox, MCP and accepted estimates
- [x] Suggestion store, `recipe_suggest` MCP tool, list/discard endpoints (AC 11).
- [x] Inbox on **Add recipe**, review hand-off as import or owned recipe, waiting count (AC 12).
- [x] `scripts/suggest_recipe.py` for customer-operated pipelines, documented in the MCP guide (AC 13).
- [x] Accepted water-density estimate for volume and spoon units, labelled (AC 14).
- [ ] Adapt the customer's `reelcatch` to emit JSON lines — customer-side, outside this repository.

## Slice 3 — AI structuring through Nextcloud Task Processing (decision F39-AI: A)
- [ ] Learn which Task Processing types the customer's instance offers; the local test instance
  has no provider, so nothing can be verified here yet.
- [ ] Design for small, low-cost models first (customer preference, 2026-11-28).
- [ ] Text/speech/picture → suggestion, always through review; consent and provider shown.

## Slice 4 — nutrition calculation
- [x] Recipes saveable with unweighed quantities and explicitly incomplete nutrition (AC 16).
- [x] Carry rough estimates and household weights into nutrition as a visible note (AC 17).
- [x] Shopping, planning, cooking and the Flutter cooking view handle a missing mass (AC 18).
- [ ] Wider source-backed reference portions, including pieces (AC 19). The retained USDA
  snapshot has no further usable household portions; BLS has none. Needs a source decision.
- [ ] Cooking yield/retention model (AC 19, F02/F03 debt). Needs retention-factor source data.

Merge gate: remote CI must pass on the exact PR head.
