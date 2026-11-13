# Roadmap

This is a proposed delivery order, not a promise of dates. Dependencies can change after review.

| Milestone | Outcome | Features |
|---|---|---|
| M0 | Runnable server, specs, docs and delivery pipeline | F01 |
| M1 | Recipe/nutrition foundation, household constraints, first storage/deployment slice, localization and product web clients | F02–F04, F09, F18, F21–F23, F29–F30 |
| M2 | Shared recipes, family plans, shopping, guided cooking, incremental preference feedback | F05–F07, F10–F12 |
| M3 | Barcode, long dough processes, inventory/offers, imports, MCP | F08, F13, F15–F17 |
| M4 | Brewing/fermentation and commercial operations | F14, F19 |
| M5 | Opt-in federation | F20 |

M1 starts with local recipe revision files and pinned BLS/USDA nutrition sources.
The authenticated identity model, Git/SQL adapters and Flutter interaction remain to be specified. F18 spans several milestones: a first backend
is not completion of the four-backend/Nextcloud/client acceptance contract. Online learning
starts with inspectable rules and feedback; production ML follows measured benefit.

PR #4 is accepted. Its browser surface is a demo only. F21 starts with pragmatic demo localization;
F22 specifies Vue 3 shells for Nextcloud and standalone hosting before product UI implementation.

PR #5 is accepted (F23 Done; initial F21 slice accepted). F04/F18 now starts local recipe
authoring and file persistence; proportional scaling and production integrations remain open.

## Delivered Nextcloud scope
F06/F07/F08 provide native household shopping, optional shop ordering and barcode products.
The customer approved closing F21/F22 for the delivered Nextcloud client. F29/F30 retain their
original complete cross-client acceptance contracts for unfinished content/standalone/Flutter work.
F18 remains In progress for its other deployment/storage/client requirements.

## Proposed phone delivery sequence (F35)

This is a plan for customer approval, not delivered app functionality or a dated commitment.
Each implementation PR contains its necessary Flutter, Rust, compatibility and device evidence:

1. P1: real Nextcloud connection and independent shared shopping, including safe offline replay.
2. P2: household/profile tour, recipe/ingredient/shared library and optional activity.
3. P3: reviewed whole-week planning, physical mealprep allocations and inventory/offers.
4. P4: native guided cooking, baking/fermentation timers, reminders and feedback.
5. P5: phone camera, share-sheet URL/barcode/OCR review and final portability audit.

[Detailed dependencies, acceptance and install gates](../specs/0035-flutter-mobile-client/plan.md).
The customer must merge the planning PR and give a separate Go. The initial phone platform still needs
confirmation; Android APK is a proposal. Standalone identity, other persistence adapters and hosted
commerce keep their own contracts rather than delaying the first Nextcloud phone journey.
