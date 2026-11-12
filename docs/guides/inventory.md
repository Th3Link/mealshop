# Household inventory and manual offers

Open **Inventory** in the native Nextcloud navigation. **Inventory**, **Offers** and **Stock history**
share the current household and its existing owner/editor/reader permissions. Readers can inspect
records; only current writers can review and confirm changes. Nothing uses a separate login or host.

## Record what is actually there

**Add stock** records an ingredient, an explicitly saved product or an unlinked item, an exact
quantity/unit, location and optional date/batch. Unlinked stock never matches recipes by name.
Quantities retain the original input and normalize only within mass, volume or count: kg/g and l/ml
are compatible; water does not imply a volume-to-mass conversion. A product purchase matches its
exact barcode; an ingredient requirement may use product stock only through its explicit mapping.

Edit location/date without rewriting the original quantity. **Correct quantity** records an observed
remaining balance, including zero, and a reason. Active reservations cannot be undercut; release them
first. Empty lots can be archived. Passed dates remain visible but are not proposed for use; missing
dates are explicit and never represented as a safety guarantee.

Every change first shows **Review changes**. Compare before/after stock, reservations, purchase
requirements and any unbooked remainder before **Confirm reviewed changes**. Review alone writes
nothing. Stock history retains who booked what and when, the original source and correction reason.

## Cover shopping and record physical changes

- From the shopping list choose **Use stock**, select contributions and a use date, then review.
  Compatible lots are reserved in date order. A planned meal's production date also constrains the
  reservation. Held quantities reduce proposed purchases but do not consume physical stock.
- Fully reserved/consumed entries appear under **Covered by stock**, separately from **Purchased**.
  Partial coverage shows reserved and used quantities alongside the remaining purchase. Changing or
  removing a source releases obsolete holds; already consumed coverage retains its original facts.
- Shopping check-off creates a purchase receipt. **Record purchased stock** explicitly selects
  unbooked receipts and their storage metadata; confirmation creates lots. Check-off alone never
  changes physical stock. Reverse the stock booking before unchecking a booked receipt.
- After completing a cooking session, choose **Record cooking consumption**. Select the compatible
  shopping contributions it fulfils and whether to use unreserved stock. Requirements come from the
  saved recipe revision and servings, not guessed screen text. Own reservations are used first;
  another contribution's holds cannot be consumed. Missing quantities remain visibly unbooked.
- **Reverse booking** appends the opposite movement with a reason. It does not erase history and
  cannot make stock negative or steal reservations. Restoring a previously archived lot revives it.

## Manually record an offer

Create shops in the shopping list, then **Record offer** with an explicit ingredient/saved product,
package quantity, positive price (at most two decimal places), currency, inclusive validity dates,
source label and optional HTTPS source link. No website is contacted and no substitution is inferred.
Supported currencies are EUR, GBP, USD and CHF; prices are recorded without currency conversion.

**Add offer to shopping** chooses package count, shopping date and intended profiles. Current known
exclusions block the operation; unknown dietary data requires explicit review. Confirmation checks
current profiles again. Archived/out-of-date offers cannot be added. Editing or archiving an offer
never changes existing shopping contributions: each retains its package, shop, price and source.

## Retry, concurrency and persistence

The expected revision protects changes shared with shopping. On a definite conflict, load latest
state while keeping entries, adjust the draft and review again. A confirmed request with an ambiguous
network result is locked. **Retry confirmed booking** sends the same actor-bound key and exact body,
returning the original booking even after server restart; it cannot double-book. Leaving requires
explicit discard. Check history before starting another request after leaving an uncertain result.
No operation keys or household content are saved to browser storage.

Inventory, offers and operation receipts live together in the existing private `household/shopping.json`
transaction, with 0600 permissions, synced atomic replacement and the 4 MiB cap. Preserve the complete
file for backup/restore. Old snapshots with no inventory remain readable. Once used, downgrade requires
an explicit migration that preserves movements, receipts and shopping coverage; never delete receipts
or retry keys to repair a failure. Pending/ambiguous publication fails closed until storage is recovered.

Limits per household: 500 lots, 200 offers, 2,000 allocations, 1,000 physical movements and 2,000
operation receipts; selections are bounded to 100 entries. History is paginated. Capacity errors are
explicit; there is no silent deletion of audit facts. Provider imports, automatic purchasing and
nutrition/expiry safety claims are outside this delivery.

[Specification](../../specs/0015-inventory-offers/specification.md) ·
[Atomicity decision](../adr/0024-household-stock-and-offers.md) ·
[Acceptance evidence](../../specs/0015-inventory-offers/validation.md)

## API contract

Paths are relative to the authenticated household domain and retain its existing JSON request marker,
CSRF boundary, enabled-state and current membership checks.

| Request | Contract |
| --- | --- |
| `GET /api/household/inventory` | Exact lot balances, allocations, offers and current dietary status, unbooked purchase receipts and completed cooking metadata. `limit` 1–500 (default 50), `offset`; `history_limit` 1–200 (default 50), `history_offset`. Optional `run_id` adds canonical completed-recipe requirements and compatible source IDs. |
| `POST /api/household/inventory/preview` | `{ expected_revision, idempotency_key, reviewed: false, command }`; validates the same bounded transaction without publishing and returns before/after lot/shopping changes, allocations, new movements and changed offers. |
| `POST /api/household/inventory` | Same immutable request with `reviewed: true`; returns operation ID, current inventory and `replayed`. Reuse the exact body/key on an uncertain result. |

Commands are `create_lot`, `update_lot`, `adjust_lot`, `archive_lot`, `reserve`, `release`, `receive`,
`consume`, `reverse`, `save_offer`, `archive_offer` and `add_offer`. The Rust tagged commands and typed
Vue contracts define their payloads. Keys are actor-bound opaque ASCII strings of 16–128 characters.
The revision belongs to the shared shopping transaction; changing key payloads or stale new intents
returns a conflict. Replays require current write permission and healthy storage, then return the
current projection without reapplying the original physical movement. No operation keys or private
command receipts appear in the read projection.
