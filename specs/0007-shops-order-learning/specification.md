# Preferred shops and learned shopping order

Status: Done
Priority: Should
Origin: Customer
Milestone: M2

## User value
Find ingredients at the right shop and walk the shop in a useful order.

## Acceptance criteria
1. Given an ingredient, a household can set first and second choice shops and manually edit shop-specific aisle ordering.
2. Given live check-off events, an opt-in learner derives order separately per shop from actual shopping sessions; undo/corrections do not become confident observations.
3. Given sparse history, a manual or default order works and the learned order is labelled uncertain; manual overrides take precedence.
4. Given a shop change or missing stock, items can move to a fallback shop without losing quantities or recipe provenance.
5. Given learning history, a household can inspect, reset and disable it without losing its shopping list.

## Out of scope
Location surveillance or unconsented cross-household training.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F07](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).
