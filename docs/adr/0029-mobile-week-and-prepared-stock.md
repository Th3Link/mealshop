# ADR 0029: Reviewed household weeks and declared physical prepared portions

Status: Accepted for P3 implementation, 2026-11-14

## Context

F35 P3 requires a usable native household week and physical mealprep, extending F10/F15/F31 together.
Existing plans pin recipe revisions and count production once. Inventory already atomically publishes
stock, shopping coverage and actor-bound retry receipts. Planned allocations are not physical stock.

## Alternatives

- Generate the week in Dart: rejected; it duplicates dietary constraints and cannot guard web writes.
- Treat prepared meals as grams of a nearby ingredient: rejected; portions have a different dimension.
- Add another stock database or generic sync framework: rejected; it splits atomic receipts and balances.

## Decision

Use additive default-empty state in the existing bounded plan and shopping snapshots. No new crate.
Attendance defaults to the current profiles, with optional per-person recurring weekdays/meals and
explicit date/meal overrides. Person hard availability remains a separate upper bound. Explicit
attendance selections cannot bypass an exclusion; the canonical single-slot web paths enforce it too.
Dates are civil ISO dates: adding a day is not adding 24 elapsed hours across DST.

Week proposals are bounded server-owned records with monotonic identity, expiry, current plan/profile/
catalogue/allergen references and pinned candidate revisions. Generation never changes meals or stock.
Apply accepts a proposal identity and explicit confirmations for its feasible slots, rechecks all
constraints/references, and publishes the whole accepted set in one plan replacement. Unfilled slots
stay explicitly infeasible/skipped; no exclusion is relaxed. Deterministic greedy ranking explains
declared dislikes, variety, ingredient reuse and active time, without claiming global optimality.
Within one locked proposal request, at most 100 immutable recipe/participant/portion assessments
are reused; date/attendance/physical-stock/time/slot constraints are still recomputed for every slot.
There is no cross-request or disk cache. Price/nutrition are not optimized without comparable coverage; unknown is never zero. Nutrition remains
the canonical per-meal evidence projection. Editing one meal continues to use F10 preview/apply.

Actual prepared output is an explicit household observation, independently of planned production or
elapsed cooking timers. Record the pinned recipe, original whole portions, declared preparation/storage
dates and condition/location. Store it alongside inventory operation receipts, with separate whole-
portion holds/use/release and explicit reversal. Replays cannot produce twice; allocations cannot
exceed observed yield. Updating storage records a new declaration, never an inferred safe shelf life.
Ingredient/product mass, volume and count lots retain their existing dimension/identity rules.
Booking output does not silently debit raw ingredients; F15's separately reviewed completed-cooking
consumption remains authoritative. Planned production is counted once; distributing its portions does
not create a new ingredient-shopping source.

Native planning/stock writes are online with preview/confirmation. Ambiguous stock writes retry the
same immutable actor-bound key; definite conflicts retain drafts and require another review. No new
unprotected disk cache, background notification promise or offline plan replay is introduced.

## Consequences

Existing snapshots remain readable. Once new fields/commands are published, downgrade needs a complete
validated migration; never delete receipts/private fields to repair it. Capacities and no-store/current
membership checks remain. P4 still owns cooking/timers, device reminders and feedback; P3 exposes only
real plan/preparation/shopping actions. F35 and physical-handset acceptance remain open.
