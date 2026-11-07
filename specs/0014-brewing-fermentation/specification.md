# Brewing and fermentation workflows

Status: Draft
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
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F14](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).
