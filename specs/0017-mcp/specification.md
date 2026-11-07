# MCP recipe tools

Status: Draft
Priority: Should
Origin: Customer
Milestone: M3

## User value
Manage recipes through authorized assistant tooling.

## Acceptance criteria
1. Given an authenticated MCP client, scoped tools search/read authorized recipes and preview insertion of a structured recipe.
2. Given recipe creation, validation and imported-content sharing restrictions match the regular API and the action is auditable.
3. Given a household boundary or read-only scope, tools cannot expose another household or mutate data.
4. Given repeated insertion with an idempotency key, the server avoids duplicate recipes and returns a structured error for invalid quantities.

## Out of scope
Unauthenticated access or treating imported recipe text as tool instructions.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F17](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).
