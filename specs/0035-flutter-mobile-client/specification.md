# Flutter mobile client and smartphone delivery

Status: In progress
Priority: Must
Origin: Customer, 2026-11-13; supplied product specification dated 2026-11-12
Milestone: M1

## User value and approval boundary

Use Mealshop on a real smartphone from the first implementation PR, initially against the existing
Nextcloud installation. Prefer a few substantial, complete user journeys over separate scaffolding,
backend-only and screen-only PRs. Each delivery extends an installable app.

The customer approved Android APK delivery, explicitly requested an Android emulator, and gave the
implementation Go on 2026-11-13. P1 was merged in PR #44. The customer approved P2 on 2026-11-13; implementation is on
`story/flutter-people-library`.
The earlier planning PR remains a separate review record; its preserved source import is unchanged.

The [original pack](../../references/mobile-product-spec-2026-11-12/README.md) remains a preserved
reference. [Current state](current-state.md) distinguishes shipped server/web behaviour from missing
mobile functionality. [The plan](plan.md) proposes **five larger implementation PRs**, each containing
its required backend, Flutter UI, tests and phone-install instructions. Internal tasks remain small;
there is no extra technical-foundation PR before the first usable app.

## Acceptance contract

1. **Install and connect (P1).** An installable development build connects to a configurable Nextcloud
   base URL, authenticates with that device's revocable app credential, selects/creates a household,
   exposes the current role and supports sign-out/revocation/account switching without cross-account
   cache leakage. Real phone connectivity and update installation are demonstrated. Android APK is
   the accepted initial platform; a real accelerated Android emulator is required in addition to a
   downloadable handset-installable build. Physical-handset testing is recorded separately.
2. **Independent shopping (P1).** Manual exact-quantity entry, editing/removal, check-off/undo,
   source details, shops/order and existing product capture work without a meal plan. Two authorized
   clients see canonical changes. Bounded durable offline add/check operations survive process death,
   show pending/failed state and replay without duplicate purchases or entries. Changed contribution
   scope, concurrent edits and revoked membership require safe reconciliation, not blind overwrite.
3. **People and content (P2).** A skippable, resumable tour creates two people with different explicit
   dietary rules, distinguishes persons from login accounts, and separates hard exclusions from soft
   preferences. Presets/exceptions and optional goals/windows/prep preferences have visible effective
   rules; every introduced hard rule is enforced on the server's existing evaluation/planning paths.
   Sensitive health fields require an approved access/retention model before collection. Recipes,
   ingredients, subrecipes/homemade inputs, exact nutrition uncertainty, collections and sharing work
   through existing contracts. Optional activity records duration/type without requiring weight.
4. **Plan and prepare (P3).** The server proposes a reviewed household week with per-meal attendees,
   portions, substitutions and explicit infeasibility. Hard exclusions are never relaxed. A Thursday
   swap previews only relevant shopping changes and preserves manual/purchased amounts. Six physical
   prepared portions can allocate four to later meals and retain two as explicitly stored leftovers;
   planned allocations alone never prove cooked stock or create invented storage/safety guarantees.
5. **Cook and ferment (P4).** Guided cooking, step ingredients, simultaneous timers, multi-day baking,
   preparation calendar, brew recipes/batches/equipment/readings and completion/history/learning have
   native phone workflows. Server deadlines persist; cached cooking remains readable offline. Local
   reminders are permission-aware and reconcile on resume/restart without resetting deadlines or
   claiming guaranteed delivery. Disconnected actions are not reported as server-confirmed.
6. **Capture and review (P5).** Share-sheet URLs, manual recipe entry, supported barcode scanning and
   on-device OCR candidates lead to explicit source/ingredient/quantity review. Unknown units or
   nutrition stay unresolved. Imported sharing remains source-link-only across server and mobile
   cache/export/share surfaces. Restricted video downloads and external OCR/AI uploads are not assumed.
7. **Quality across P1–P5.** DE/EN keyed messages ship from P1 with English canonical code/content,
   exact decimal-string quantities and source-language fallback. Each active screen has loading,
   empty, error, offline/conflict and appropriate reader states, accessible touch/semantics and text
   scaling. Real-device evidence, release/test artifact identity, upgrade test, domain/API tests and
   current-head CI accompany each PR. No sensitive records, credentials or screenshots enter CI logs.

## Scope reconciliation

The five-tab target is Today / Plan / Shopping / Recipes / More. Enable a tab when its real workflow
ships; do not fill the initial build with dead or dummy destinations. The first launch opens Shopping.
Solo use follows the same journey with one person; additional accounts/profiles are not mandatory.
Shared household planning is proposed, with per-person attendance/alternatives; independent personal
plans remain an explicit later extension. No irreversible model decision is inferred from the pack.

Nextcloud is the first working connection, reusing its account and household authority. Standalone
hosted authentication, Git/SQL adapters, commercial admission and federation retain F18/F19/F20/F30
scope and are not prerequisites for a useful Nextcloud phone app. Unsupported deployment modes are
identified honestly. Flutter localization contributes to F29 without declaring its full cross-client
and versioned-content contract Done.

Optional medical context is not a diagnosis-to-diet engine. Smart scales, wearables, autonomous
purchasing and restricted video retrieval remain explicit non-goals. Nutrition/energy estimates
need defensible inputs and uncertainty, not clinical claims or mandatory calorie tracking.

## Approval and traceability

Review [plan.md](plan.md), [ux-notes.md](ux-notes.md), [adrs.md](adrs.md) and proposed ADR 0026.
N01/N03/N04/N06–N10 apply. F18 AC4 and the Flutter part of F29 are related contracts, not closed by
this planning PR. All implementation checklist entries remain unchecked until delivered and verified.

## P2 person privacy decision

Activities have person-specific read/edit grants, including care without a child login. The customer
selected owner-managed permissions **without automatic owner access**: an owner must visibly grant
permission to their own account before reading/editing activity. Existing shared dietary profiles keep
their household roles; no new medical/weight fields are introduced. ADR 0028 defines atomic storage,
revocation, generation/revision guards and live deletion/export. P2 remains part of the same substantial
frontend/backend PR; P3–P5 are not marked delivered.
