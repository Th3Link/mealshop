# F06 delivery plan

## Implemented slice
- [x] AC1: exact recipe quantities and explicit selected meal-plan slots; pinned revisions, strict dimensions.
- [x] AC2: contributor/receipt disclosure and editable unrelated goods.
- [x] AC3: revision-checked source edits preserve immutable purchases and manual sources.
- [x] AC4: native checkboxes, 48px touch areas, focused shopping mode, responsive layouts.

## Evidence
`cargo test --locked --test shopping`: four protected API integration tests pass, including role denial,
conflicts, restart recovery, unit separation, rescaling and selected plan synchronization.
`npm run build` passes with strict Vue/`TypeScript` checking. The embedded Nextcloud browser contract
is `clients/nextcloud/e2e/shopping.spec.ts`; final integrated results are recorded in the combined delivery.

## Boundaries
Purchased receipts are not an inventory ledger. Each recipe source owns its purchases; unused amounts
are not automatically reassigned to another source. Capacity/ambiguous writes fail closed.

Final delivery: all four ACs pass the API/native browser acceptance mapping in validation.md.
The combined suite passes 16/16, followed by 4/4 final-image shopping/dialog checks.
