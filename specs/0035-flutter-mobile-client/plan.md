# Flutter delivery plan — five complete phone releases

Status: Proposed; planning PR only. Implementation requires customer merge **and a separate Go**.
Task/checkpoint authority: this file, following the repository's spec-folder convention. Do not
create a second `tasks/` backlog or mark the imported pack's checklist as delivered.

## Delivery order and scope

| PR | Usable smartphone outcome | Required server work in the same PR | Dependencies |
| --- | --- | --- | --- |
| P1 — Connect and shop | Install app, log into Nextcloud, choose household, shop together online/offline | Mobile auth/proxy proof, capability/version contract and bounded shopping request receipts | Approved plan/Go; chosen test platform and reachable test origin |
| P2 — People and recipe library | Guided profiles, explicit rules, recipes/ingredients/collections, optional activity | Compatible profile/rule/privacy extensions, server enforcement, activity persistence | P1; approved sensitive-data policy |
| P3 — Household week and physical preparation | Review a whole week, swap meals, allocate mealprep/stock and update shopping | Whole-week proposal/validation, recurring attendance/windows, physical prepared-batch/stock consistency | P2 effective rules and recipe data |
| P4 — Cook, bake and ferment | Large step mode, offline-readable recipes, timers/reminders, feedback and brew workflows | Audited cooking replay/reconciliation and native notification contract | P3 plan/stock links; P1 identity/cache/outbox |
| P5 — Camera, share and import | Capture URL/photo/barcode, review uncertainty and save correct original-source content | Existing reviewed import plus approved OCR candidate contract only where needed | P2 authoring/provenance; approved OCR/platform capability |

These are **five implementation PRs**, not five architectural layers. Build backend, phone UI and
verification together. Each PR may contain several small verified commits; avoid separate scaffolding,
API-only, empty-navigation or last-minute “make everything work” PRs. Platform signing/CI belongs in
P1. Larger review units are the customer's preference; internal tasks remain bounded and sequential.

## P1 — Install, authenticate and complete a shopping session

Phone demo: install the APK, enter a Nextcloud URL, authenticate in the system browser, choose/create a
household, add `200.000001 g` of an ingredient, check/undo, see the change in Nextcloud or a second phone,
use airplane mode, terminate/reopen the app, reconnect and reconcile safely. No meal plan is required.

- [ ] P1.1 **Prove the transport and install path.** Set up pinned Flutter/Dart and Android CI in
  `clients/flutter/`; choose a development application ID and stable test signing identity. Exercise
  Login Flow v2, cancellation, expired/revoked credentials, server subpaths and real proxy GET/POST on
  Nextcloud 35. Keep app credentials in platform-protected storage, never the `ExApp` secret. Define an
  authenticated capability/version response as an explicitly new contract before relying on it.
  Verify: a physical-device install/login/relaunch/sign-out; server owner/reader/denied tests;
  the ordinary Nextcloud client still works. Likely areas: new Flutter platform/config/network modules,
  `src/exapp/`, contract tests and CI. No unrelated authentication rewrite.
- [ ] P1.2 **Add safe offline request semantics.** Extend the existing shopping transaction with
  bounded durable operation receipts: actor/account/household scope, immutable payload/key, expected
  revision, atomic result and exact replay after lost responses. Revalidate current permission before
  replay. A reused key with a different request conflicts. Existing callers retain their current
  contract; negotiate the new capability. Record migration, capacity and downgrade/recovery behaviour.
  Verify: apply-then-drop-response/restart/replay produces exactly one entry/purchase; two clients
  cannot silently replace sources. Likely areas: `src/shopping/{api,model,store}.rs`, inventory receipt
  invariants, `tests/shopping.rs`, native client compatibility tests. Receipts are not a new sync engine.
- [ ] P1.3 **Deliver the real list and persistent client outbox.** Manual items, exact units,
  check/undo, quantities, source details, shops/order and existing manual product entry/consented lookup.
  Foreground refresh and refresh-on-resume first; no assumed WebSocket or guaranteed background worker.
  Scope encrypted/minimized cache and outbox by canonical server/account/household.
  Define offline retention/authorization limits; remote revocation cannot erase an offline copy.
  Revalidate on reconnect and stop pending writes when access is removed. Represent pending,
  confirmed, failed and conflict states separately. On account/home switch, isolate in-flight responses;
  sign-out wipes local credentials/data. Preserve drafts on errors and reader views without write actions.
  Verify: widget/decimal/DTO tests and two-device integration with process death and role revocation.

**P1 release gate:** downloadable installable test artifact + checksum/commit/version, installation and
upgrade instructions, real phone evidence, Rust/native regression and Flutter gates below. Initial active
tabs are Shopping and More, without placeholders for unimplemented flows. P1 closes F35 AC1–2 only.

Critical conflict case: device A checks an item while offline; device B adds a new requirement for the
same ingredient. Do not replay an aggregate check against unseen new contributions. Freeze the observed
item/contribution scope, refresh and require reconciliation when it changes. Manual pending additions
must remain separate from previously purchased amounts. Do not auto-rebase financial/stock effects.

## P2 — Household tour, content and optional personal activity

Phone demo: create two people, one with a vegan baseline plus an explicit own-egg exception and one
omnivore; inspect the resolved rule summary; interrupt/resume the tour; author a nested recipe, scale it,
inspect unknown/ranged nutrients, share the allowed projection, and record a 45-minute jog without weight.
The diet label must reflect exceptions rather than silently claiming an unmodified vegan diet.

- [ ] P2.1 **Extend effective rules and privacy together.** Resolve person-vs-account ownership,
  child-profile access and health-field permissions/retention before collection. Keep allergies and
  explicit exclusions hard, dislikes soft, goals optional and medical context non-prescriptive. Add
  reviewed presets/exceptions, ingredient-origin conditions, windows/attendance/prep settings only with
  versioned storage, validation and migration. Every new hard rule must also constrain existing
  Nextcloud evaluation/suggestions/plan apply; do not store a rule the server then ignores.
  Verify: conflicting presets/exceptions, unknown origin, unrelated allergy, revoked access, export/
  deletion and legacy profile round trips; adapt existing typed Vue contracts in this same PR.
- [ ] P2.2 **Build the resumable profile flow.** One or two questions per step, optional skip,
  protected local drafts, explicit save/review and editable summary. No required diagnosis, weight or
  calorie target. A login account and a household person remain distinct identities.
  Verify: interruption/relaunch, no cross-person draft leakage, screen reader/large text and DE/EN.
- [ ] P2.3 **Complete the mobile content library.** Search localized names/aliases; read/create/edit
  recipes and ingredient groups/subtypes/nutrients/components/tags/images, with inline ingredient creation,
  source/range/unknown presentation, portions, always-visible parts/steps, homemade inputs and cycle
  rejection. Add collections, pinned shared revisions and invitations. Source-only imported projections
  must also govern local cache/export/share. Share sheets for URL capture follow P5.
  Verify: exact values and original language survive edits; restricted imports are not copied into shares;
  reader/revocation, conflicts and unknown ingredient identities preserve the server boundary.
- [ ] P2.4 **Add the small personal activity log.** Type, date, duration and optional intensity with
  save/history/edit/delete, independent of food restrictions. Review permission/retention and add a
  bounded versioned server record. Leave energy unknown without an accepted evidence/inputs model.
  Verify: a 45-minute jog and 60-minute ride save without weight; no kcal is fabricated or used to
  weaken hard constraints. Likely areas: profile/activity API/model, phone More/profile flows and tests.

**P2 release gate:** installed P1 upgrade retains account/cache and existing user records. Recipes joins
Shopping/More; no inactive Plan/Today tab. All introduced hard rules have server enforcement evidence.
Sensitive fields stay unavailable if their separate privacy decision is not approved; report that scope
explicitly rather than claiming the full P2 acceptance complete.

## P3 — Explainable household week, mealprep and stock

Phone demo: generate/review a week for the two people, change Thursday dinner/attendance, inspect the
shopping delta, then prepare six portions on Sunday, allocate four to later meals and retain two frozen
portions. A plan edit must neither erase manual purchases nor count cooked production twice.

- [ ] P3.1 **Implement a server-owned full-week proposal.** Reuse current evaluation, slot suggestions,
  pinned recipes and profile/allergen revisions. Add proposal identity/expiry, per-slot confirmation,
  effective recurring attendance/date overrides/windows and optional prep/time preferences. Explain
  infeasibility; rank only supported soft criteria. Missing price/nutrition data is not zero cost/energy.
  Verify: two-person hard-rule fixtures, impossible week, stale profile/recipe, DST/date boundaries and
  deterministic recommendation evidence. Any new persisted constraint also protects existing web paths.
- [ ] P3.2 **Deliver mobile Today/Plan and reviewed shopping changes.** Day/week navigation, participant/
  portion edits and swaps preserve selected slots and protected source amounts. Show changes before
  apply and retain drafts after concurrency failures. Today links real meals/preparation/shopping actions.
  Verify: Thursday-only change, skipped attendee, manual/checked shopping preservation and two clients.
- [ ] P3.3 **Unify physical batches and inventory carefully.** Extend current planned allocations and
  exact inventory transactions for actual prepared yield, storage location, portion allocation/release,
  leftovers, lots, offers and reversal. Track user-declared dates/conditions without invented safe shelf
  life. No allocation exceeds recorded available quantity; planning alone cannot create physical stock.
  Preserve ingredient/product/batch dimensions and original quantities. Verify: six = four allocated + two
  remaining, freeze/thaw or declared storage changes, over-allocation, cancel/reversal/restart and
  production-once shopping; keep existing F15/F31 web behaviour compatible.

**P3 release gate:** all five functional tabs now exist. Upgrade the installed app and complete the
source pack's shared-week success journey through shopping/preparation, with current permissions and
source/uncertainty explanations. Fully independent personal plans remain the explicitly tracked extension.

## P4 — Cooking, long preparations, fermentation and feedback

Phone demo: start two timers, close/kill the app and disconnect; reopen to the original deadlines and
step recipe. Delay a multi-day baking task, reconcile reminders, complete actual work, book stock only
through review, and inspect a separate brewing batch with its equipment/calibrated reading history.

- [ ] P4.1 **Native cooking and offline recovery.** Large step UI, current-step ingredients,
  dependencies/part relationships, actual-action waits, multiple timers and server-backed rescheduling.
  Audit current cooking mutations; add durable operation replay where absent before queueing them.
  Store offline intent separately and reconcile current state; never reset a timer from “now” on restart.
  Verify: process death, clock change, overlapping timers, lost responses, explicit manual completion
  and stale role/recipe/equipment state. Reuse the shared Rust scheduler, not a Dart scheduler copy.
- [ ] P4.2 **Permission-aware device reminders.** Select maintained platform integration after SDK,
  licence and OS-capability review. Schedule stable notification IDs from authoritative deadlines,
  cancel/update on replan and reconcile on foreground/reconnect. Distinguish local reminders, server
  delivery and acknowledged user actions. Existing Nextcloud notifications are not Flutter push.
  Verify foreground/background/reboot and denied/revoked permissions on the supported OS versions;
  record actual delivery limits. APNs/FCM infrastructure is a separate explicit gate, not assumed.
- [ ] P4.3 **Complete fermentation/history integration.** Mobile brew recipes/templates, batches,
  equipment reservations, original measurements/calibration/corrections, history/ratings, optional learning
  consent/reset and planned-prep links. Verify concurrent physical holds, independent batches, exact
  readings and food/brewing separation. Completion does not silently imply stock movement or safety.

**P4 release gate:** the full source-pack core loop works on the phone with disclosed offline/reminder
limits. No medical claim or physical fermentation-complete claim follows from elapsed time.

## P5 — Camera and reviewed capture

Phone demo: share a supported public recipe URL, import a photo with an unresolved cup measurement,
review ingredient mappings/quantities, scan a product and recover after camera/lookup denial. Save and
reopen the original source and reviewed values; share an imported recipe as its source link only.

- [ ] P5.1 **Share-sheet and URL review.** Reuse F16 network bounds and explicit retrieval consent.
  Mobile original/reviewed views, manual fallback, preview expiry/idempotent save and source-only
  exports must preserve the same rules as Nextcloud. Verify blocked/private/redirect URLs and retries.
- [ ] P5.2 **Camera/barcode and on-device OCR candidates.** Review platform/plugin/licence support and
  privacy before dependencies. Default proposal: local OCR, explicit photo access and retained original
  candidates with confidence/unknowns; no external upload. Use manual GTIN and consented existing lookup
  when camera support fails. Verify real phone permissions, cancellation, large/rotated images,
  unresolved `1 cup`, exact known decimals and no silent ingredient or medical inference.
- [ ] P5.3 **Complete the release and portability audit.** Test all enabled journeys on each supported
  phone platform, upgrades, cache/outbox schema migrations, account removal/export and accessibility.
  Document provider/platform limits and safe unsupported-video/manual paths. Do not add social-media
  bypasses, scale drivers, wearables, autonomous purchases or commercial admission to finish the checklist.

**P5 release gate:** all F35 acceptance criteria have real evidence. Source-pack proposals blocked by
external product/provider decisions remain explicit follow-up requirements, with original criteria;
F35 cannot be marked fully Done by silently dropping them.

## Build, install and validation contract (starts with P1)

The following are **future commands**, not commands claimed to pass in this planning PR:

```sh
cd clients/flutter
dart format --output=none --set-exit-if-changed .
flutter analyze
flutter test
flutter build apk --debug
flutter test integration_test -d <test-device-id>
flutter install --debug -d <test-device-id>
```

Pin SDK/toolchains when P1 starts, record `flutter doctor -v`, commit the app lockfile and review package
licences. Provide versioned CI debug APK artifacts from a stable development application ID/key plus
commit/checksum/install instructions; keep production keys out of source and all retained signing keys out of untrusted PR jobs.
A protected distribution workflow or local build with the retained test key supplies upgradeable
artifacts; ordinary untrusted PR builds compile/test without those secrets.
Test an upgrade without uninstalling/wiping. Production signed APK/AAB and store admission are separate
from debug acceptance; release builds must reject cleartext/invalid TLS by default.

Android USB proposal: after installing ADB, `adb reverse tcp:8080 tcp:8080` can forward the phone's
loopback endpoint to the existing host test instance. Validate the browser/login poll and returned
canonical Nextcloud URL as part of the real test. A phone/emulator's `127.0.0.1` alone is not this host.
Permit loopback cleartext only in explicit debug configuration; never disable certificate verification.
For Wi-Fi/iPhone use a configured reachable test origin and approved trusted-domain/proxy settings.
Do not expose or reset the test instance in this planning PR.

iOS source support is a target, but physical iPhone installation needs macOS/Xcode and provisioning;
`TestFlight` needs the relevant Apple team/program setup. A Linux APK build or unsigned simulator build
is not iPhone acceptance. If iPhone is the chosen first device, P1 must establish that install path
before claiming a usable first delivery. Keep all secrets in protected signing storage.

Each PR runs affected Flutter unit/widget/integration tests, Rust contract/migration tests and existing
Nextcloud regressions, plus the repository's complete mandatory gate. Use a phone and a second phone
or real Nextcloud browser as distinct authenticated household clients; mocked transports do not replace
end-to-end proof. Record OS/device/build, conditions, screenshots using synthetic data and results,
including pending unsupported capabilities. Do not commit personal credentials or health fixtures.

## Decisions and approval gates

| Decision | Proposed treatment / owner | Required by |
| --- | --- | --- |
| First physical phone/platform | Customer question pending; Android APK proposal, no inferred approval | Before P1 implementation |
| Install origin/signing | Test setup with customer-controlled reachability and keys; no hostname baked into client | P1 |
| Shared vs individual plans | Shared household default with per-person alternatives; preserve separate-plan extension | P2 model review |
| Sensitive/child profile access and retention | Explicit policy/projections before any diagnosis fields or health-cache sync | P2 |
| OCR/provider support | Prefer on-device; validate package/platform/licence; external service requires separate consent | P5 |
| Notification/push capabilities | Local deadline reminders first with measured limits; no assumed provider | P4 |
| Standalone/hosted identity and commercial distribution | Keep open in F18/F19/F30, not silently implemented as master tokens in the app | Separate accepted slice |

## Source-pack coverage

Phase 0 → this audit/plan; shell → P1; tour/recipes/activity (original phases 1/7) → P2;
planning/mealprep/pantry (2/4) → P3; cooking (6) → P4; capture (5) → P5; complete shopping (3) → P1.
The five source journeys map to P2+P3 (mixed dietary rules), P3 (Thursday swap and six portions),
P5 (uncertain photo import) and P1 (two-client offline list). None is treated as implemented by import.
