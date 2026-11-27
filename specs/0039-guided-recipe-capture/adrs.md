# Decisions

See [ADR 0038](../../docs/adr/0038-guided-capture-and-household-unit-weights.md).

- Text capture is a client-side deterministic parser (`recipe-text.ts`); it adds no endpoint and
  sends nothing anywhere. Its output is a draft for the existing editor, not a saved recipe.
- Picture capture reuses F16/F35 `mode: photo` with `manual_transcription`; no new engine value.
- Ratings stay in F11 history entries. `GET /api/household/recipe-ratings` only counts reported
  values per recipe; it does not expose or depend on learned state.
- The nutrition overview is presentation only. Rounding happens in the client for five tiles; the
  server calculation, exact strings and range rules are untouched.

## Proposals awaiting a customer decision
- **AI provider boundary.** Proposed: Nextcloud Task Processing called by the ExApp on behalf of the
  user (the OCS impersonation pattern already used for notifications), so the instance admin picks
  a local or remote model and Mealshop stores no key. Which task types (text, speech, vision) the
  customer's instance offers is unverified. Alternative: an owner-supplied provider key.
- **Suggestion inbox.** Proposed: an MCP tool and endpoint that accept an unreviewed candidate
  (raw lines, steps, source URL, optional transcript) without ingredient IDs. The cook reviews it
  in the capture review. Imported candidates keep source-link-only sharing (N06).
- **Instagram.** The server cannot and should not fetch reels (login walls, robots, terms).
  Proposed: a customer-operated companion modelled on the customer's `reelcatch` (saved collection
  → download → local Whisper transcript → model summary) that posts candidates to the inbox.
  It uses the customer's own session on the customer's machine; Mealshop ships no scraper.
