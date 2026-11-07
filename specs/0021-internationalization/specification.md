# Internationalization and localized content

Status: In progress
Priority: Must
Origin: Customer (after PR #4 merge, 2026-11-07)
Milestone: M1

## User value
Use Mealshop in German or English, find ingredients by familiar names, and enter precise
quantities in a familiar notation without changing their meaning.

## Acceptance criteria
1. UI messages are keyed resources with German/English support, an explicit language choice,
   regional-tag normalization and a documented fallback. Missing translations never render keys.
2. Ingredient, recipe and educational content have language-independent identities and
   reviewed, versioned translations. Search covers localized names, aliases and source names;
   unavailable translations show source text with a clear original-language label.
3. Quantity input accepts the active locale's decimal separator without binary floating point
   or guessing grouped numbers; formatting uses locale separators while retaining exact values.
   Locale changes preserve valid input and do not silently reinterpret invalid input.
4. Number/date/plural formatting and messages work in Vue/Nextcloud, standalone web and Flutter
   clients. Locale settings are independent of preferred units and serving quantities.
5. API IDs, nutrient meanings/units and stored canonical decimal strings remain stable.
   Localizable errors/warnings use stable codes plus parameters, not parsing of English prose.
6. Translation coverage, fallback, locale switching, search and exact decimal boundaries have
   focused checks; nutrition calculations and original source provenance are unchanged.

## Current slice
Only the disposable nutrition demo gets de/en UI resources, language selection (saved locally
when browser storage is available), browser-language detection, localized starter names/forms,
multilingual demo search and exact decimal entry/output. Reviewed content absent from the demo
translation catalogue is explicitly presented in its original language. Calculation warnings
get additive stable codes so the demo does not translate backend sentences by string matching.
This is a first step, not completion of all six cross-client criteria.

## Out of scope for this slice
A production translation service, account-wide language preferences, complete ingredient/recipe
content translation storage, API content negotiation, all backend error codes, Vue/Flutter clients
and additional languages. Preserve source facts instead of generating unreviewed health translations.

## UX notes
See [ux-notes.md](ux-notes.md). Keep the demo pragmatic; do not add product design work.

## Links
Requirements: N09/N10 in [requirements](../../docs/requirements.md).
Decisions: [adrs.md](adrs.md). Work: [plan.md](plan.md).
