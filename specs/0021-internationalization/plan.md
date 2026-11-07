# Plan for internationalization

## Current slice
- [x] Record customer direction and distinguish demo scope from production clients.
- [x] Add keyed de/en demo resources, starter ingredient labels and original-text fallback (AC 1–2).
- [x] Add exact decimal entry/display and preserve inputs during language changes (AC 3, 5).
- [x] Add stable warning codes and focused locale/API checks (AC 5–6).
- [x] Verify repository gate and prepare a reviewable PR.

## Remaining product work
- [ ] Specify versioned content translations and API locale negotiation (AC 2, 5).
- [ ] Integrate localization across Vue/Nextcloud, standalone web and Flutter (AC 4).
- [ ] Complete locale-aware date/plural/error contracts across those clients (AC 4–6).

## Validation
Use pure locale tests for supported regional tags, dictionary coverage/fallback, source-name
fallback, localized search and precise decimal strings beyond JavaScript's safe integer range.
Retain Rust nutrition tests; add a warning-code contract test. Exercise German entry with a
comma and language switching without changing the canonical recipe. Reuse the existing demo
layout and do not add visual design work.

## Delivery evidence (2026-11-07)
See [shared validation](../0023-kitchen-ingredient-catalogue/validation.md). The current slice
is implemented; cross-client localization and customer acceptance remain open.

Customer accepted the initial demo/catalogue slice by merging PR #5. Broader product work remains open.
