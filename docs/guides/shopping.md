# Household shopping

Open **Shopping list** inside a household. Readers inspect; owners/editors change it.

## Finding and checking goods
The list opens in shopping mode. Open goods appear as compact, named tiles with exact quantities;
tap a tile or focus its checkbox and press Space to check it off. Purchased goods remain under the
**Purchased** disclosure and can be unchecked. Text and checkboxes accompany color. Category icons
come only from explicitly linked ingredient metadata; unlinked goods use a neutral basket.

**Find an item** filters canonical and translated names without changing saved data or order. Long
names wrap to two lines; their complete names remain accessible and visible in item details. Press
Enter in search or choose **Add item** to enter an explicit quantity/unit inline. A name never selects
an ingredient automatically. Saving returns focus to search; failed writes retain the draft. The last
confirmed check/uncheck exposes **Undo** until another mutation or reload replaces that context.

Each tile has a separate **Item details** action for full provenance, receipts and shop actions.
**Manage list**, beside search, reveals recipe/meal-plan sources, shops, products and history controls.
Clear search before changing a shop's item order; a filtered subset cannot replace the full order.

## Sources and purchases
Add goods with exact g/kg/ml/l/piece quantities, optionally linked to an ingredient. Select a saved
recipe revision/servings, or explicitly choose meal-plan slots. Updating that selection replaces only
meal-plan contributions. Identical ingredient IDs/base dimensions aggregate; mass, volume and counts
never merge. Inspect **Recipes and contributions** for original purchased receipts. Source edits/removal
change only outstanding requirements. Undo reverses purchases without resurrecting removed requirements.
Purchases belong to their source; excess amounts are not reassigned automatically to another recipe.
**Shopping mode** returns to the focused list while retaining item details and large check-off targets.

## Shops and learning
Create/rename/remove shops and set ingredient first/second choices under **Shops and shopping order**.
Assignments/fallback moves change only metadata. Filter by a shop for keyboard/touch up/down ordering;
manual order wins. Clear it to use learning/default order. Unranked items use deterministic name order.

Learning defaults off. Enable it, select a shop and explicitly start/complete a session. Only actual
check-offs at that assigned shop count. Three completed multi-item sessions per item are required for
an integer normalized position average. A correction excludes its whole session, including historical
sessions containing that item. Sparse history is labelled uncertain. No location or remote training.
Inspect, discard, reset or disable history independently of shopping data. Disabling discards an active
session; completed history remains. Shop removal clears its metadata/history, preserving list sources
and receipts; complete/discard an active session first.

## Products and barcodes
Type a checked EAN-8/UPC-A/EAN-13/GTIN-14. Original leading zeros remain; equivalent codes share a
canonical GTIN-14 key. UPC-E expansion is unsupported. Household products resolve locally. Unknown
products have complete manual entry. Preserve package labels verbatim and explicitly enter quantity/unit;
never guess amounts from a label. Confirm generic ingredient matches before aggregation. A match groups
shopping amounts only, establishing no nutrition/allergy/health facts. Package counts add explicit sources
with immutable product snapshots; later product edits cannot change them.

Camera starts only on request in a supported secure browser with native `BarcodeDetector` and permission.
Frames stay local; tracks stop after detection, stopping, dialog closure/navigation or document hiding.
Manual entry works without camera support or network. Optional Open Food Facts lookup needs per-request
consent and discloses barcode, server IP and program user agent. No household names, recipes, quantities
or frames leave the instance. Fixed HTTPS, no redirects, timeouts, 64 KiB responses and local throttling
bound requests. Review source, `ODbL` 1.0, retrieved/modified dates and original provider label before saving.
Package quantity/matching remain explicit. Candidates expire after 24 hours; only the latest twenty stay.
[Provider API](https://openfoodfacts.github.io/openfoodfacts-server/api/) and
[barcode normalization](https://openfoodfacts.github.io/openfoodfacts-server/api/ref-barcode-normalization/).

## API and recovery
GET /api/household/shopping returns revisioned items, sources, shops/history and products. POST takes
`{expected_revision,command:{operation,...}}`. Rust and strict `TypeScript` contracts define operations;
quantities are decimal strings. POST /api/household/shopping/lookup requires a writer and
`{expected_revision,barcode,consent:true}`. A lookup reserves a revision before requesting the provider;
after failure the client refreshes that revision while retaining the draft. Stale writes return 409.
Never automatically replay a mutation after an unknown outcome: reload/review to avoid duplicate sources.
Nextcloud uses existing household proxy authentication, roles and CSRF marker.

The private mode-0600 household/shopping.json is atomically replaced under the household writer lock.
Limits: 4 MiB, 1,000 sources, 5,000 purchases, 100 shops, 200 sessions and 1,000 product records. Capacity
errors are explicit. Reset learning history before its limit fills. Source/receipt archival needs a future
accepted migration; never delete the ledger to bypass a limit. Corrupt data or .shopping.pending fails
startup closed. Stop the writer and restore a verified complete backup before restarting.
New metadata fields default on older shopping files; older binaries may reject them. Rollback restores
the matching complete backup. Back up membership, recipes, profiles, plans, images, ingredients and
shopping together. Never run host/container writers simultaneously. Test fixtures are archived only
with ownership/content evidence; customer households are never selected by age or UI position.

Recipe contributions buy measured base inputs. An explicit homemade output (`prepared_part`, F33)
is displayed in the recipe but excluded from purchases; its preparation's measured ingredients
count once. This also applies to meal-plan synchronization and rescaling. Availability is never
inferred globally from the ingredient name.

## Household stock and offers

Use **Inventory** or **Use stock** to review reservations before reducing purchases. Covered-by-stock
entries remain separate from purchase receipts. Recording purchased stock and completed cooking is
explicit; booked receipts must be reversed before unchecking. Offers retain frozen package/price/source
facts and recheck current dietary constraints. See the [inventory guide](inventory.md).
