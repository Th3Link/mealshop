# Products and barcode entry

Manual EAN/UPC/GTIN entry is always available. Camera scanning is an explicit action, uses supported
native `BarcodeDetector` formats, stops tracks on close, and falls back to typing when permission,
secure context or browser support is missing. Product identity and labelled package quantity are
separate from the optional generic ingredient match, which requires explicit confirmation.

Prefer household product records. Open Food Facts lookup is optional per request: disclose the barcode
and server IP sent to that provider, attribution/ODbL licence, retrieved/modified dates and manual fallback.
No household names, recipes, quantities, profiles or camera frames leave the instance. A lookup candidate
must be confirmed before saving. Preserve the original label text; require explicit exact package units
instead of guessing ambiguous package strings. Product matches do not establish nutrition/allergy safety.

Adding packages is an explicit extra shopping contribution, with package count and original units retained.
Use native Nextcloud dialogs/forms and keyed de/en text. Do not require external connectivity for shopping.
