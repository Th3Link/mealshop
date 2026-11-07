# Federated recipe and meal-plan exchange

Status: Draft
Priority: Could
Origin: Customer
Milestone: M5

## User value
Exchange selected content between independent Mealshop servers.

## Acceptance criteria
1. Given opt-in federation, a user can share an authorized recipe between servers without opening private household data.
2. Given a shared meal-plan template for similar intolerances, sharing uses explicit consent and minimal disclosed attributes, never raw health profiles by default.
3. Given imported recipes, the exchanged representation contains the original source link only.
4. Given an untrusted remote server, authentication, protocol versioning, moderation, revocation limitations and abuse controls are specified and tested before enablement.

## Out of scope
A federation protocol choice or automatic cross-server health-profile discovery.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F20](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).
