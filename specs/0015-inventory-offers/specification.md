# Inventory and shopping offers

Status: Done
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
See [ux-notes.md](ux-notes.md) for native entry, review, recovery and accessibility.

## Links
Requirement: [F15](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).

## Nextcloud delivery contract
- Manual lots retain original exact quantity/unit, explicit ingredient/product or unmatched identity,
  location, optional batch/date and append-only physical movement history. Correction records the
  observed remaining quantity and reason; used identities/units are immutable and empty lots archive.
- Shopping/plan exports reserve exact compatible lots after review, with dated lots ordered first;
  passed dates are not proposed. Unknown dates are explicit. No mass/volume/count assumptions.
- Source changes release obsolete holds. Held stock reduces proposed purchases; confirmed usage
  remains visibly distinct from purchased receipts. Plans/check-off alone never debit physical stock.
- Reviewed purchased receipts add lots; completed cooking books reviewed actual available inputs.
  Incomplete coverage is explicit. Reversals reference the original movement, append its opposite
  and cannot create negative balances or consume another reservation.
- Stable actor-bound operation keys, identical payload checks and expected shopping revisions are
  atomically persisted with effects. Readers cannot mutate; definite conflicts retain local drafts
  and ambiguous responses retry the immutable confirmed request.
- Offers are manually entered with shop, explicit binding, positive exact price (EUR/GBP/USD/CHF),
  positive package quantity, inclusive date range and a source label/optional HTTPS link. Optional
  product bindings retain their reviewed snapshot. Adding offers checks the explicit shopping date
  and current intended profiles; exclusions block it. There is no provider/scraping or substitution.
- Preserve old snapshots/default-empty state, the existing 4 MiB cap and canonical private publication.
  Bound lots/offers/allocations/movements/receipts and validate persisted references on restart.

Decisions: [ADR 0024](../../docs/adr/0024-household-stock-and-offers.md). Complete AC 1–4 with native
owner/reader tests, durable restart/failure tests, required gates and synthetic-only test cleanup.

## Acceptance evidence

AC 1–4 are implemented and verified in the native Nextcloud delivery. See [validation.md](validation.md)
for isolated domain tests, all installed browser scenarios, transport/restart/cleanup evidence and the
mandatory current-head CI boundary. No provider or automatic purchasing is implied.
