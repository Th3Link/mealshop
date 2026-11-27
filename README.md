# Mealshop

Recipes, informed nutrition, shopping and family meal planning. **Early development:** a
Rust HTTP server includes a curated ingredient catalogue (F02/F23, accepted in PRs #4/#5),
plus local recipe authoring/persistence (F04/F18) and a native Nextcloud client (F22).
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

## Install in Nextcloud 35

Mealshop is an **External App (`ExApp`)**: the released container runs the Rust backend and Vue
interface, integrated through Nextcloud `AppAPI`. Do not copy this repository into Nextcloud's `apps/`.

Stable tags `vX.Y.Z` publish Linux AMD64/ARM64 images to `ghcr.io/th3link/mealshop` and an OCI Helm
chart to `oci://ghcr.io/th3link/charts/mealshop` after all release gates pass.
[Release v0.1.0](https://github.com/Th3Link/mealshop/releases/tag/v0.1.0) is published: both
architectures and the chart are anonymously pullable, with verified checksums and image-digest pinning.

With Nextcloud 35/`AppAPI` 35 and a dedicated existing `APP_SECRET` Secret, install the matching chart:

```sh
helm upgrade --install mealshop oci://ghcr.io/th3link/charts/mealshop \
  --version 0.1.0 --namespace nextcloud \
  --set nextcloud.url=https://cloud.example.org \
  --set appApi.existingSecret=mealshop-appapi \
  --set persistence.storageClass=longhorn --wait
```

Set your actual URL, Secret and storage class. The chart uses a retained PVC and a single rootless
writer; browser requests use Nextcloud's authenticated `AppAPI` proxy. Complete the one-time `AppAPI`
registration before opening Mealshop. See the [Flux example](deploy/helm/flux-example.yaml) and
[release, installation and recovery guide](docs/guides/releases-and-helm.md) for the full workflow,
secret handling and package visibility. No local image build is needed for a published release.

The [Nextcloud development guide](docs/guides/nextcloud.md) retains local/test transport details;
the [Android guide](docs/guides/mobile-capture.md#install-on-a-smartphone) covers phone connection.

## Run the ingredient and nutrition demo
Install stable Rust (declared minimum: 1.89), then:
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
(or `MEALSHOP_RECIPE_DIR`) and keeps the default local/demo mode loopback-only. Nextcloud uses the authenticated transport described below.
[Recipe API, storage and recovery guide](docs/guides/recipes.md).
Use Ctrl-C for graceful shutdown. A saved recipe can be previewed at another serving count. Optional protected local household profiles use an operator token; see [household setup and limits](docs/guides/household.md). Standalone hosted authentication remains future work. Tests: `cargo test --locked`. Full gate: [development guide](docs/guides/development.md).

The current browser surface is a pragmatic test demo, not the product UI. It supports German
and English interface/quantity presentation; untranslated source content stays explicitly original.
A native Vue 3 Nextcloud client now covers shared-household recipes, profiles, planning and the catalogue ([setup and verification](docs/guides/nextcloud.md)). The standalone Vue shell remains planned ([F30](specs/0030-standalone-vue-client/specification.md));
full cross-client localization is tracked in [F29](specs/0029-cross-client-localization/specification.md).

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
Renovate refreshes reviewed mobile licence inventories with dependency versions; see
[automatic refresh and manual licence review](docs/guides/development.md#renovate-licence-inventories).

## License
AGPL-3.0-only. See [LICENSE](LICENSE). Paid distribution does not remove recipients' rights
to use, modify or redistribute the software under that licence.

Protected family planning links saved recipe revisions with household constraints, preparation
budgets and consented calendar attendance imports. Changes have an explicit shopping-quantity
preview before apply. See the [family planning guide](docs/guides/meal-planning.md).

Native household shopping lists offer searchable item tiles, inline exact-quantity entry and last-check
undo while preserving recipe/plan contributions and purchased receipts in item details.
Choose shops, set manual order or opt into local session learning; packaged products support manual
GTIN entry, optional supported-camera scanning and consented Open Food Facts lookup.
[Shopping guide](docs/guides/shopping.md) · [source/provenance evidence](specs/0006-shopping-list/validation.md) ·
[visual shopping](specs/0032-visual-shopping/specification.md).

Native recipe reading/editing uses compact, always-open parts and ingredient plus actions.
Explicit homemade ingredients keep their finished quantities while measured base inputs alone feed
nutrition and shopping. [Recipe guide](docs/guides/recipes.md) ·
[F33 contract](specs/0033-compact-recipe-preparation/specification.md).

Native guided cooking now saves independent preset timers and reviewed multi-day baking schedules.
Personal Nextcloud reminders retain delivery uncertainty and deadlines across restart.
[Cooking guide](docs/guides/cooking.md) · [F12 evidence](specs/0012-guided-cooking/validation.md) ·
[F13 evidence](specs/0013-bread-schedules/validation.md).

F34 adds explicitly labelled comparison envelopes alongside immutable reported composition. All 313
bundled foods and 73 organizational groups have energy/protein/fat/carbohydrate coverage; unsupported
individual nutrients remain unknown. Recipe totals include exact source readings and comparative
ranges, with outward rounding and visible incomplete subtotals. Own declared ranges are supported.
See [F34](specs/0034-evidence-nutrition-ranges/specification.md) and
[the evidence protocol](data/README.md#supplemental-nutrition-and-comparison-evidence-v1).

Native collections organize ordered, explicitly pinned recipe revisions. Household owners can grant
read access to Nextcloud accounts or whole households; incoming invitations and content appear in
Shared with me. Imported recipes expose only their original source URL to recipients, including
when an older revision was shared before its origin was corrected.
[Sharing and collections guide](docs/guides/sharing.md) · [F05](specs/0005-sharing-collections/specification.md).

Completed cooking history records actual attendees, optional reported effort and household/profile
ratings. Owner-controlled local learning starts fresh on opt-in/reset, explains suggestion evidence
and keeps the planner's hard exclusions and explicit review. [History and preferences](docs/guides/history-and-preferences.md).

Household assistant tools now offer explicitly granted MCP ingredient/recipe search, canonical previews
and confirmed, idempotent recipe creation through existing Nextcloud authentication. Imported content
remains source-only. [Connection, permissions and recovery](docs/guides/mcp.md).

Native reviewed recipe import fetches supported public structured sources after explicit consent,
retains original fields separately, and offers manual private entry when retrieval is blocked.
Ingredient/quantity review is mandatory and imported sharing remains source-link-only.
[Import, source boundaries and recovery](docs/guides/recipe-import.md).

Household inventory records exact lots, reviewed shopping reservations, explicit purchased/completed-cooking
bookings, reversals and manually entered shop offers. [Inventory guide](docs/guides/inventory.md) ·
[F15 evidence](specs/0015-inventory-offers/validation.md).

Native brewing adds source-linked BIAB/all-grain/extract/partial-mash and named/DIY equipment templates,
plus independently researched cider, white/red wine and traditional mead processes,
immutable brew recipes, reviewed independent batches, physical equipment reservations, personal reminders
and exact instrument/manual reading history. [Brewing guide](docs/guides/brewing.md) ·
[F14 research](specs/0014-brewing-fermentation/research.md).

The native Android client starts in [F35](specs/0035-flutter-mobile-client/specification.md).
[Five device-testable implementation PRs](specs/0035-flutter-mobile-client/plan.md) reconcile the
[customer mobile pack](references/mobile-product-spec-2026-11-12/README.md) with existing APIs.
P1 implements Nextcloud Login Flow v2, shared shopping and a protected durable offline outbox.
P2 adds the resumable person tour, explicit dietary rules, online recipe/ingredient/collection/sharing
views and private person activities with explicit grants; [permissions and API](docs/guides/people-activity.md).
See the [mobile guide](docs/guides/mobile.md) for the pinned emulator, APKs and upgrade instructions,
and F35 verification for actual evidence and physical-handset testing limits.

The Android app also supports real [on-device operation with encrypted backups](docs/guides/app-only.md),
without a Nextcloud account or running server. Choose **Use on this device**, then manage manual
backup and restore from **More**. Local data and connected households remain separate.

F35 P3 adds native Today/Plan, reviewed server-owned household weeks, recurring attendance and
explicit physical prepared portions alongside existing inventory. P3 emulator/live-proxy verification is recorded in
[F35 evidence](specs/0035-flutter-mobile-client/verification.md#p3-implementation-evidence);
[week/stock contract](docs/guides/mobile-planning.md). Cooking and device reminders remain P4.


F35 P4 adds recoverable native cooking, reviewed multi-day/brew workflows, Android device reminders,
preparation agenda and completed cooking history/owner learning controls. Absolute deadlines, offline
intent, uncertain delivery and explicit physical-stock booking remain distinct. See
[mobile cooking and recovery](docs/guides/mobile-cooking.md); final delivery evidence is in the P4 PR.

F39 (in progress) adds guided recipe entry to the Nextcloud client: paste or dictate text, keep a
picture beside it, review once and save; spoons and pieces need only what one unit weighs. Recipes
show ratings from completed cooks and a rounded per-serving nutrition overview above the exact
table. No AI provider is integrated yet. [F39](specs/0039-guided-recipe-capture/specification.md).
