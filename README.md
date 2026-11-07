# Mealshop

Recipes, informed nutrition, shopping and family meal planning. **Early bootstrap:** only a
Rust HTTP server with `GET /health` exists. All product features below are planned.

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

## Run the bootstrap
Install stable Rust (declared minimum: 1.85), then:
```sh
cargo run --locked
curl --fail http://127.0.0.1:3000/health
```
The response is `ok` plus a newline. `MEALSHOP_LISTEN_ADDR` overrides `127.0.0.1:3000`.
Use Ctrl-C for graceful shutdown. No authentication, storage or recipe endpoints exist yet;
this is a development stub. Tests: `cargo test --locked`. Full gate: [development guide](docs/guides/development.md).

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
