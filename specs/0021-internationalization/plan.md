# F21 Nextcloud delivery

- [x] Keyed de/en resources, explicit choice, regional normalization and English fallback.
- [x] Reviewed localized ingredient names/search, source-language fallback and exact decimal boundaries.
- [x] Locale switching preserves valid and invalid authoring input; dates/counts/serving plurals use the active locale.
- [x] Typed API contracts and stable error/warning codes preserve server calculations and original facts.
- [x] Customer-approved follow-up F29 retains the original full acceptance contract.

## Acceptance evidence
`node --test tests/i18n.test.mjs` covers the demo baseline. `npm --prefix clients/nextcloud run test`
covers native exact boundaries, localized input, regional normalization, serving plurals and date/meal presentation.
Embedded Nextcloud E2E contracts cover German entry, switching, fallback labels and quantity persistence.
See F06 validation for the final combined run and F22 validation for the previously delivered native shell.

No standalone or Flutter implementation is claimed.
