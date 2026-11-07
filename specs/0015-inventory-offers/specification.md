# Inventory and shopping offers

Status: Draft
Priority: Should
Origin: Customer
Milestone: M3

## User value
Use existing stock and relevant offers in planning and shopping.

## Acceptance criteria
1. Given household stock, quantity, unit, location and optional expiry/batch can be recorded and corrected.
2. Given a plan/list, available stock can reduce the proposed purchase with a visible reservation; a plan alone never consumes stock.
3. Given confirmed cooking/purchases, stock updates are reviewable and idempotent; corrections reverse the relevant movement.
4. Given a shop offer, shop, product/ingredient match, price, package size, validity and source are recorded; dietary exclusions still win.

## Out of scope
Unlicensed offer scraping or automated purchasing.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F15](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).
