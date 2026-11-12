# Feature specifications

Each stable NNNN-slug folder contains specification.md (what/why/acceptance), adrs.md
(decisions) and plan.md (implementation and validation). Add ux-notes.md for UI features.
Lifecycle: Draft → Ready → In progress → Done. Draft implementation tasks are intentionally
unchecked. Must means required for the intended product, not all in the initial milestone.
New folders take the next free number; keep index/status changes in the same commit.

Features are grouped by status, then sorted by milestone and requirement ID.

## In progress

| Requirement | Feature | Priority | Milestone |
|---|---|---|---|
| F18 | [Storage adapters, Nextcloud `ExApp` and Flutter](0018-storage-deployment/specification.md) | Must | M1 |
| F16 | [Customer-operated recipe link import](0016-recipe-import/specification.md) | Should | M3 |

## Draft

| Requirement | Feature | Priority | Milestone |
|---|---|---|---|
| F29 | [Cross-client localization and versioned content translations](0029-cross-client-localization/specification.md) | Must | M1 |
| F30 | [Standalone Vue client and cross-client integration](0030-standalone-vue-client/specification.md) | Must | M1 |
| F15 | [Inventory and shopping offers](0015-inventory-offers/specification.md) | Should | M3 |
| F14 | [Brewing and fermentation workflows](0014-brewing-fermentation/specification.md) | Could | M4 |
| F19 | [Hosted plans, paid app and recipe marketplace](0019-commercial-service/specification.md) | Could | M4 |
| F20 | [Federated recipe and meal-plan exchange](0020-federation/specification.md) | Could | M5 |

## Done

| Requirement | Feature | Priority | Milestone |
|---|---|---|---|
| F01 | [Server and delivery foundation](0001-bootstrap/specification.md) | Must | M0 |
| F02 | [Ingredient catalogue and nutrition](0002-ingredients-nutrition/specification.md) | Must | M1 |
| F03 | [Precise quantities and configurable units](0003-quantities-units/specification.md) | Must | M1 |
| F04 | [Recipe authoring and scaling](0004-recipe-authoring/specification.md) | Must | M1 |
| F09 | [Household members and dietary constraints](0009-household-profiles/specification.md) | Must | M1 |
| F21 | [Nextcloud internationalization](0021-internationalization/specification.md) | Must | M1 |
| F22 | [Native Nextcloud Vue client](0022-product-web-clients/specification.md) | Must | M1 |
| F23 | [English-base kitchen ingredient catalogue](0023-kitchen-ingredient-catalogue/specification.md) | Must | M1 |
| F24 | [Native Nextcloud daily workflows](0024-nextcloud-daily-workflows/specification.md) | Must | M1 |
| F25 | [Ingredient groups and nutrition](0025-ingredient-groups/specification.md) | Must | M1 |
| F26 | [Comfortable native authoring](0026-authoring-experience/specification.md) | Must | M1 |
| F27 | [Complete catalogue organization](0027-catalogue-organization/specification.md) | Must | M1 |
| F28 | [Native usability review](0028-native-usability-review/specification.md) | Must | M1 |
| F05 | [Recipe sharing and collections](0005-sharing-collections/specification.md) | Must | M2 |
| F06 | [Shopping lists and recipe provenance](0006-shopping-list/specification.md) | Must | M2 |
| F07 | [Preferred shops and learned shopping order](0007-shops-order-learning/specification.md) | Should | M2 |
| F10 | [Family meal planning and calendar attendance](0010-meal-planning/specification.md) | Must | M2 |
| F11 | [Cooking history and online preference learning](0011-recommendation-feedback/specification.md) | Should | M2 |
| F12 | [Guided cooking and preset timers](0012-guided-cooking/specification.md) | Must | M2 |
| F31 | [Unified preparation, mealprep and ingredient extensions](0031-unified-preparation/specification.md) | Must | M2 |
| F32 | [Visual household shopping](0032-visual-shopping/specification.md) | Must | M2 |
| F33 | [Compact recipes and explicit homemade ingredients](0033-compact-recipe-preparation/specification.md) | Must | M2 |
| F34 | [Source-backed nutrition ranges](0034-evidence-nutrition-ranges/specification.md) | Must | M2 |
| F08 | [Barcode scanning and product matching](0008-barcode/specification.md) | Could | M3 |
| F13 | [Multi-day bread, rolls and pizza scheduling](0013-bread-schedules/specification.md) | Should | M3 |
| F17 | [MCP recipe tools](0017-mcp/specification.md) | Should | M3 |
