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

## Slice 2 — suggestion inbox and MCP (needs decision F39-INBOX)
- [ ] Candidate store, `recipe_suggest` MCP tool, inbox list and review hand-off.
- [ ] Companion contract for customer-operated sources (Instagram), documented, not bundled.

## Slice 3 — AI structuring (needs decision F39-AI)
- [ ] Verify available Task Processing types on the customer's instance.
- [ ] Text/speech/picture → candidate, always through review; consent and provider shown.

## Slice 4 — nutrition calculation (needs decision F39-WEIGHTS)
- [ ] Wider source-backed reference portions; offer them as prefilled, labelled suggestions.
- [ ] Recipes saveable with unresolved quantities and explicitly incomplete nutrition.
- [ ] Cooking yield/retention model (F02/F03 debt).

Merge gate: remote CI must pass on the exact PR head.
