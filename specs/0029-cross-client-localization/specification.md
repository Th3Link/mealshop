# Cross-client localization and versioned content translations

Status: Draft
Origin: Customer-approved separation of remaining scope, 2026-11-10

The original acceptance contract is retained below without deleting requirements. Already verified
Nextcloud behaviour remains the baseline; the standalone/Flutter and outstanding content contracts
require their own implementation and evidence. No implementation of those clients is claimed here.

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
