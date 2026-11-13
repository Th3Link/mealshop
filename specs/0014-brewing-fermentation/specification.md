# Brewing and fermentation workflows

Status: Done
Priority: Could
Origin: Customer
Milestone: M4

## User value
Use configurable processes and equipment for beer, then cider, wine and mead.

## Acceptance criteria
1. Given beer brewing, separate process templates support brew-in-a-bag, all-grain and extract with equipment-specific steps and measurements.
2. Given a brew day, a recipe may schedule a yeast-out-of-fridge reminder the day before and further phase-specific timers.
3. Given readings from a hydrometer/temperature device or manual input, units, calibration, timestamp and source are retained.
4. Given several simultaneous batches, their equipment reservations, stages and reminders remain independent.
5. Given cider/wine/mead, additional templates are introduced only after domain validation; beer assumptions do not silently carry over.

## Out of scope
Automatic heater/pump control, food-safety guarantees or unvalidated non-beer chemistry.

## UX notes
See [ux-notes.md](ux-notes.md) for native layout, review, recovery and accessibility.

## Links
Requirement: [F14](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).

## Customer research clarification and delivery contract

The customer requested internet research on BIAB, Grainfather, Brew Monk, Speidel Braumeister, DIY
all-grain, extract and partial mash on 2026-11-13. [research.md](research.md) retains primary sources
and separates those facts from implementation choices. BIAB is represented within all-grain;
fermentable strategy, vessel layout, heating and sparging are distinct.

The native delivery includes beer, cider, wine and mead recipes with immutable revisions and exact target liquid
volume/entered ingredients, source-linked editable phase templates, explicit equipment bindings,
reviewed independent batches and phase reminders. Readings retain raw units, measurement/entry
timestamps, manual/instrument attribution and known or unknown calibration; corrections preserve
the original. Manual instrument transcription is supported without claiming a live connector.

Use the existing durable scheduler and atomic household file with an honest food/brew source boundary.
No food servings are repurposed as litres, no fake food recipe is created and no automatic stock
consumption or chemistry calculation is inferred. Entered yeast instructions/times remain recipe-
specific. Independently reviewed cider-juice, white-wine, red-wine and traditional-mead templates
use their own process classifications; see the primary-source validation in research.md.

Decision: [ADR 0025](../../docs/adr/0025-native-brewing-processes.md).

## Acceptance evidence

See [validation.md](validation.md) for all five criteria, source review, real Nextcloud/browser/restart
checks, identified defects and their passing repeats, privacy-preserving cleanup and publication gates.
