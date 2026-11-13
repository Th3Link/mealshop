# ADR 0025: native brewing and fermentation processes on the durable household scheduler

Status: Accepted for the customer-requested F14 delivery
Date: 2026-11-13

## Context
F14 adds independently validated beverage-specific templates, measured batches and equipment reservations. The customer explicitly
asks for researched BIAB, Grainfather, Brew Monk, Braumeister, DIY all-grain, extract and partial mash.
The [research](../../specs/0014-brewing-fermentation/research.md) distinguishes fermentable strategy,
vessel layout, heat and sparging. Existing cooking timers already persist deadlines, dependencies,
IANA civil time and personal `AppAPI` delivery uncertainty. Their food recipe/servings reference is not
a valid representation of a brew recipe and target liquid volume.

## Alternatives
A second scheduler/notification worker duplicates recovery and delivery semantics. Fake food recipes,
servings-as-litres or synthetic recipe IDs corrupt provenance and F15/history integration. Browser-only
countdowns fail across restart. Automatic manufacturer/device integrations would invent unsupported
protocol and credential contracts. Unrestricted generic workflow frameworks are unnecessary.

## Decision
Keep the existing single household cooking transaction and scheduler. Add optional/default-empty brewing
recipes, equipment and readings in that atomic file. Make each process source explicitly a food-recipe
reference or a brew-recipe reference. Preserve the exact old food JSON through a validated wire adapter;
no false food/servings record is created for drinks. Existing food APIs, history and inventory consumption
remain food-only. Brew routes reuse the same deadline/dependency/timezone and notification operations.
Cross-family preview/apply boundaries must be checked, not inferred from a client URL.

Brew recipes have immutable revisions, an exact target liquid volume, entered ingredient quantities,
source-linked process choices and editable phases. A batch pins the revision and its actual equipment.
Reservation checks share the same writer lock and publication as batch plans/actions. Planned overlaps
are visible and block conflicting allocation. Started or paused work retains physical equipment until
explicit completion/cancellation, including overdue fermentation. Brewing/fermentation dependencies require explicit
completion; an elapsed biological timer cannot imply readiness. Support long declared fermentation
windows without broadening food-only policy accidentally.

Store original readings with exact units, civil/absolute measurement time, authenticated entry time/
actor, manual/instrument attribution and explicit known/unknown calibration. Corrections append a new
record with a reason and reference the original. Instrument transcription is supported; no unconfigured
polling, vendor-cloud connection, implicit SG/Plato/Brix conversion or inferred chemistry is claimed.

All mutations retain current household permissions, review points, optimistic revisions, private synced
atomic publication and fail-closed recovery. Batch creation/reading retries must not duplicate facts.
UI uses existing native components and EN/DE contracts. No new crate or storage adapter is needed.

## Consequences
Old snapshots remain readable; new brewing state needs the current reader or an explicit preserving downgrade
migration. Back up the complete household cooking file, including receipts and notification state.
No protected household inspection or reset is required. Cider, white/red wine and traditional mead have separately reviewed sources and typed process choices,
without beer-method fields. Keep the original beer method JSON and default-omitted beer batch identity
unchanged for preserving old snapshots and operation receipts. No heater/pump control, automatic inventory consumption, ABV/IBU/priming calculator,
universal yeast handling or food-safety guarantee is introduced.
