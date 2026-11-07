# Delivery validation

Date: 2026-11-07. Scope: F21 demo slice and F23 curated catalogue; acceptance pending.

## Automated checks
All checks passed locally:
- `python3 scripts/generate_docs.py --check`
- `python3 scripts/import_catalogue.py --check`: 313 identities, 1,026,680 bytes.
- `python3 scripts/verify_catalogue.py`: 284 source profiles, 29 identity-only records,
  3,778 mapped numeric values independently compared with frozen primary records.
- `node --check src/web/demo.mjs` and `node tests/i18n.test.mjs`: six locale tests passed.
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo nextest run --workspace --locked`: 19 tests passed, none skipped.
- `cargo build --release --locked`
- `cargo deny check licenses`
- `cargo audit`: no vulnerabilities reported for the 54-dependency lockfile.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`
- `cargo msrv verify -- cargo check --all-targets --locked`: Rust 1.85.0 compatible.

## Browser exercise
The existing demo loaded 313 ingredients and 17 nutrient definitions. Switching English to
German changed keyed messages and ingredient names. Entering 100,5 g oats and 25 g chia for
two servings produced 256,22 kcal and 10,55 g protein per serving. Switching to English
preserved the quantities and displayed 100.5 and 256.22. Invalid German `1.000` remained
invalid after switching to English and could not be calculated until explicitly edited.
German search `Weizen 405` returned the exact wheat flour Type 405 record with BLS identity
C214100 and CC-BY-4.0 attribution. Combining it with 1.5%-fat milk displayed available
carbohydrate separately. Reloading retained the selected language. At a 390 × 844 viewport,
measured document scroll width equalled client width (375 px), without horizontal overflow.

## Limits and review
Source profiles are reference composition, not complete 17-nutrient profiles or measured
matches for every retail product. The 29 identity-only records deliberately contain no
invented values; manufacturer/package sources are future curation. BLS and USDA carbohydrate
and folate definitions stay separate. The catalogue is close to the existing 1 MiB loading
limit; further substantial growth needs the F18 loading/storage slice. No limit was raised.
The current UI remains a disposable demo; product Vue 3 and Flutter localization are future
work. PR checks and review evidence are recorded in the pull request; no main deployment is
part of this unaccepted feature branch.
