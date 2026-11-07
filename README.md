# Mealshop

Recipes, informed nutrition, shopping and family meal planning. **Early development:** a
Rust HTTP server includes a curated ingredient catalogue (F02/F23, accepted in PRs #4/#5),
plus a local recipe authoring/persistence slice (F04/F18, review pending).
The broader product features below remain planned.

[![CI](https://github.com/Th3Link/mealshop/actions/workflows/ci.yml/badge.svg)](https://github.com/Th3Link/mealshop/actions/workflows/ci.yml)
[![Audit](https://github.com/Th3Link/mealshop/actions/workflows/audit.yml/badge.svg)](https://github.com/Th3Link/mealshop/actions/workflows/audit.yml)
[Documentation](https://th3link.github.io/mealshop/) · [Feature specs](specs/README.md) ·
[Requirements](docs/requirements.md) · [ADRs](docs/adr/index.md)

## Vision
- Ingredient database, precise configurable units, scalable recipes, macro- and micronutrients
  per serving, and evidence-backed explanations of beneficial food components.
- Shared recipes and later paid recipes/collections; source-link-only sharing for imported recipes.
- Shopping lists with recipe provenance, extra items, preferred shops, learned aisle order and barcodes.
- Family meal plans with attendance, intolerances, preferences, cooking time and ongoing feedback.
- Guided cooking with preset timers, multi-day bread/pizza schedules, and later brewing/fermentation.
- Inventory, offers, customer-operated link imports and MCP recipe tools.

## Run the ingredient and nutrition demo
Install stable Rust (declared minimum: 1.85), then:
```sh
cargo run --locked
curl --fail http://127.0.0.1:3000/health
```
The response is `ok` plus a newline. `MEALSHOP_LISTEN_ADDR` overrides `127.0.0.1:3000`.
Open [the demo](http://127.0.0.1:3000/) to calculate recipe/serving nutrients and inspect
sources, unknowns and component evidence. The English-base catalogue contains 313 ingredients
with German names: 284 source-backed BLS/USDA profiles and 29 explicitly unknown product identities.
[Data and licences](data/README.md) · [Demo, API and custom catalogue guide](docs/guides/nutrition.md).
Recipes can be created, saved, loaded and revised using catalogue ingredients; per-serving
estimates retain their saved source snapshots. The executable stores data in `var/recipes`
(or `MEALSHOP_RECIPE_DIR`) and requires loopback access until authentication exists.
[Recipe API, storage and recovery guide](docs/guides/recipes.md).
Use Ctrl-C for graceful shutdown. A saved recipe can be previewed at another serving count. Optional protected local household profiles use an operator token; see [household setup and limits](docs/guides/household.md). Hosted authentication remains future work. Tests: `cargo test --locked`. Full gate: [development guide](docs/guides/development.md).

The current browser surface is a pragmatic test demo, not the product UI. It supports German
and English interface/quantity presentation; untranslated source content stays explicitly original.
Product web clients are planned in Vue 3 for Nextcloud and standalone hosting ([F22](specs/0022-product-web-clients/specification.md));
full cross-client localization is tracked in [F21](specs/0021-internationalization/specification.md).

## Deployment and business model (planned)
Rust/Tokio/Axum server, standalone or as a Nextcloud `ExApp`; Flutter client connecting to an
own server or our hosted service. Storage options: Git-backed files, PostgreSQL, SQLite and
MariaDB. Nextcloud installations remain free. Hosted accounts have free and paid tiers.
The distributed app costs a few euros; self-compilation and installation remain possible.
Only paid, signed app releases are intended to connect to our operated service; technical
admission and licensing details need validation (ADR 0004). Future federation may exchange
recipes and compatible meal plans between independent servers.

## Contribute
Start with [AGENTS.md](AGENTS.md), the [roadmap](docs/roadmap.md) and an affected feature spec.
The repository uses spec-driven development inspired by Curvyo and CI adapted from
homeautomation; exact source revisions and adaptation notes are in [provenance](docs/provenance.md).

## License
AGPL-3.0-only. See [LICENSE](LICENSE). Paid distribution does not remove recipients' rights
to use, modify or redistribute the software under that licence.
