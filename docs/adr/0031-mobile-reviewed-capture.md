# ADR 0031: Local capture and reviewed original evidence

Status: Accepted for P5 implementation (Android initial delivery).

## Context

F35/P5 requires Android share-sheet URLs, camera/gallery, barcodes and local OCR candidates, reviewed
against immutable originals. Existing F16 provides bounded HTTPS fetch, review and durable idempotent
import receipts. F08 has consented GTIN lookup, explicit product mapping and exact package amounts.
Neither accepts photos as original provenance or stores reviewed label declarations.

## Alternatives

- External OCR service: rejected; no provider/upload decision authorizes it.
- Bundled ML Kit: processes images locally, but its terms permit Google usage/performance telemetry;
  rejected in favour of a self-contained open-source option.
- `Tesseract4Android` + bundled fast English/German models, `ZXing` core and existing `image_picker`: selected.
  Larger APK and slower still-image recognition are acceptable; no continuous camera stream is needed.

## Decision

Pin `Tesseract4Android` 4.9.0 (Tesseract 5.5.1/Leptonica) and `ZXing` core 3.5.4. Download fast English/German
models at build setup from an immutable official commit with checked SHA-256. Bundle the models for
offline first use. Android uses one bounded worker, image bounds before decode, and normalized upright
pixels from `image_picker`. User rotation is explicit. Neither SDK has a network path in this adapter.
Display raw OCR and its uncalibrated engine score; all quantities/mappings need review. Unknowns remain
unknown. Photos/text stay local until explicit Nextcloud upload consent. A private original source photo
is immutable household evidence, separately from the editable recipe image. Photo imports are restricted
across sharing/MCP: no content, image, title or OCR escapes as a shared recipe. A photo has no fabricated
web link. Existing imported recipes continue to share only their real original link.

Extend F16 preview mode with `photo`, retained capture metadata and a `captured` recipe origin. Use the
same actor-bound preview expiry and immutable import receipt/replay. Extend F08 products with optional
reviewed label declarations including the exact printed basis, value/unit and optional definition. These
are declarations, not catalogue nutrition, conversions or allergy guarantees; ingredient mappings never
replace those declarations with a nearby food profile. Existing records remain readable without a rewrite.

Incoming text/plain Android shares are bounded and never fetched automatically. Review the destination
account/household before using them. Persist an import envelope before sending; uncertain outcomes retry
exactly, including after process restart. Scope removal/sign-out clears local state. No new Rust crate or
Pub plugin. Android-only platform support is explicit; manual entry and URL input remain available.

## Consequences

Record original sources in private append-only records; backups/downgrades must preserve new variants.
Old clients need compatible schema projections in this same PR. Camera quality, text layout and OCR scores
do not prove correctness. Unsupported videos retain manual/private entry only. Physical Android camera,
`TalkBack` and manufacturer background behavior still require handset evidence; emulator results are distinct.

## Sources

- <https://github.com/adaptech-cz/Tesseract4Android/releases/tag/4.9.0>
- <https://github.com/tesseract-ocr/tessdata_fast/tree/87416418657359cb625c412a48b6e1d6d41c29bd>
- <https://github.com/zxing/zxing/releases/tag/zxing-3.5.4>
- <https://developers.google.com/ml-kit/terms>
- <https://developer.android.com/training/sharing/receive>
