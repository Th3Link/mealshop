# Barcode scanning and product matching

Status: Draft
Priority: Could
Origin: Customer
Milestone: M3

## User value
Add and identify packaged products quickly.

## Acceptance criteria
1. Given a supported device permission, scanning a barcode resolves a product or offers manual entry when unresolved.
2. Given a packaged product, package quantity and product identity remain separate from generic ingredient identity; matching requires confirmation where ambiguous.
3. Given unavailable camera/network or a denied permission, the user can complete the shopping flow manually.
4. Given an external product lookup, source, licence, freshness and consent for transmitted data are visible.

## Out of scope
Guaranteed global barcode coverage.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F08](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).
