# Recipe authoring and scaling

Status: Done
Priority: Must
Origin: Customer
Milestone: M1

## User value
Create structured recipes that remain useful when portions change.

## Acceptance criteria
1. Given a recipe, an author can edit title, ingredients from the catalogue, quantities, serving count, instructions and active/passive durations.
2. Given a new serving count, quantities and per-serving nutrition update while the original yield remains recorded; invalid or zero servings are rejected.
3. Given ingredient substitutions or unknown ingredients, identity and nutrition gaps are visible instead of silently matched.
4. Given a saved revision, its ingredients, steps, source, author and calculation inputs can be retrieved consistently.

## Current slice (2026-11-07)
Customer-authorized local authoring/persistence with catalogue ingredients and per-serving nutrition:
- Create, list, retrieve and edit immutable revisions via API and a pragmatic de/en demo (AC 1, 4).
- Preserve exact gram inputs, title/author attribution, en/de content language, ordered instructions,
  active/passive seconds and batch servings. Empty steps are allowed for drafts.
- Pin the referenced catalogue subset and source metadata per revision. Reopening after restart or
  changing the current catalogue must not change old estimates (AC 3–4).
- Reject unknown IDs, zero/invalid servings and stale revisions. Known identities without verified
  composition retain unknown/partial nutrient totals (AC 2 validation, AC 3).
- Changing servings repartitions the same batch; original revisions remain retrievable. A read-only scaling preview uses the saved revision as its original yield; scaled quantities
  are rounded half up to 0.000001 g and nutrition uses the exact original serving ratio before display rounding.
- File storage is local/single-household, loopback-only in the executable, with exclusive locking,
  bounded versioned records, durable publication, fail-closed recovery and documented backup (F18 slice).
  No authenticated author/household identity is claimed; shared/hosted access is out of scope.

## Out of scope
Paid content delivery and external imports.

## UX notes
See [demo UX notes](ux-notes.md); product UI remains F22.

## Links
Requirement: [F04](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).

Completion scope: local single-household authoring, revision persistence and scaling.
Authenticated ownership, alternate storage adapters and product clients remain separate features.
