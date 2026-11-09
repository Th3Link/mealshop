# Feature specifications

Each stable NNNN-slug folder contains specification.md (what/why/acceptance), adrs.md
(decisions) and plan.md (implementation and validation). Add ux-notes.md for UI features.
Lifecycle: Draft → Ready → In progress → Done. Draft implementation tasks are intentionally
unchecked. Must means required for the intended product, not all in the initial milestone.
New folders take the next free number; keep index/status changes in the same commit.

| Requirement | Feature | Status | Priority | Milestone |
|---|---|---|---|---|
| F01 | [Server and delivery foundation](0001-bootstrap/specification.md) | Done | Must | M0 |
| F02 | [Ingredient catalogue and nutrition](0002-ingredients-nutrition/specification.md) | Done | Must | M1 |
| F03 | [Precise quantities and configurable units](0003-quantities-units/specification.md) | Done | Must | M1 |
| F04 | [Recipe authoring and scaling](0004-recipe-authoring/specification.md) | Done | Must | M1 |
| F05 | [Recipe sharing and collections](0005-sharing-collections/specification.md) | Draft | Must | M2 |
| F06 | [Shopping lists and recipe provenance](0006-shopping-list/specification.md) | Draft | Must | M2 |
| F07 | [Preferred shops and learned shopping order](0007-shops-order-learning/specification.md) | Draft | Should | M2 |
| F08 | [Barcode scanning and product matching](0008-barcode/specification.md) | Draft | Could | M3 |
| F09 | [Household members and dietary constraints](0009-household-profiles/specification.md) | Done | Must | M1 |
| F10 | [Family meal planning and calendar attendance](0010-meal-planning/specification.md) | Done | Must | M2 |
| F11 | [Cooking history and online preference learning](0011-recommendation-feedback/specification.md) | Draft | Should | M2 |
| F12 | [Guided cooking and preset timers](0012-guided-cooking/specification.md) | Draft | Must | M2 |
| F13 | [Multi-day bread, rolls and pizza scheduling](0013-bread-schedules/specification.md) | Draft | Should | M3 |
| F14 | [Brewing and fermentation workflows](0014-brewing-fermentation/specification.md) | Draft | Could | M4 |
| F15 | [Inventory and shopping offers](0015-inventory-offers/specification.md) | Draft | Should | M3 |
| F16 | [Customer-operated recipe link import](0016-recipe-import/specification.md) | Draft | Should | M3 |
| F17 | [MCP recipe tools](0017-mcp/specification.md) | Draft | Should | M3 |
| F18 | [Storage adapters, Nextcloud `ExApp` and Flutter](0018-storage-deployment/specification.md) | In progress | Must | M1 |
| F19 | [Hosted plans, paid app and recipe marketplace](0019-commercial-service/specification.md) | Draft | Could | M4 |
| F20 | [Federated recipe and meal-plan exchange](0020-federation/specification.md) | Draft | Could | M5 |
| F21 | [Internationalization and localized content](0021-internationalization/specification.md) | In progress | Must | M1 |
| F22 | [Vue 3 product web clients](0022-product-web-clients/specification.md) | In progress | Must | M1 |
| F23 | [English-base kitchen ingredient catalogue](0023-kitchen-ingredient-catalogue/specification.md) | Done | Must | M1 |
| F24 | [Native Nextcloud daily workflows](0024-nextcloud-daily-workflows/specification.md) | Done | Must | M1 |
| F25 | [Ingredient groups and nutrition](0025-ingredient-groups/specification.md) | In progress | Must | M1 |
