# Delivery plan: products and barcode entry

- [x] AC1: optional native scanning, local product resolution and complete manual entry.
- [x] AC2: validated GTIN, separate package quantity and explicit generic-ingredient confirmation.
- [x] AC3: manual flow survives denied camera and failed network lookup.
- [x] AC4: per-request external consent; retained attribution/licence/retrieved/modified dates and original provider label.

## Evidence
See [combined validation](../0006-shopping-list/validation.md). Backend acceptance contracts are in
`tests/shopping.rs`, provider/GTIN boundary tests in `src/shopping/lookup.rs`, native browser flows
in `clients/nextcloud/e2e/shopping.spec.ts`.
