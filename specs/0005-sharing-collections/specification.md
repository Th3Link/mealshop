# Recipe sharing and collections

Status: Draft
Priority: Must
Origin: Customer
Milestone: M2

## User value
Share owned recipes and organize collections on a server.

## Acceptance criteria
1. Given an owned recipe or collection, its owner can save it on the server and select private or explicitly shared access.
2. Given a recipient without access, recipe content is withheld; revoking a share prevents subsequent server access.
3. Given an imported recipe, sharing emits only its original source URL and never republishes cached ingredients/instructions.
4. Given a recipe revision, collections and meal plans retain a defined revision reference rather than changing historical cooking data silently.

## Out of scope
Payments, federation and claims that downloaded copies can be revoked.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F05](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).
