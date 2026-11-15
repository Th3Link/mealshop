# F35 verification

## Historical planning PR evidence

The following section records the earlier planning-only PR at commit 77b6cd5. It is not the
implementation status after the customer Go; current P1 evidence is tracked below.

Scope: imported source documents, source-based capability audit, proposed delivery/UX/ADR and navigation
links. No Flutter application, new API behaviour, SDK setup, device build or mobile runtime acceptance
is claimed. Existing test-instance data and deployment configuration were not accessed or changed.

## Import and consistency checks

- The seven extracted files match the customer ZIP byte for byte; `manifest.json` records each length
  and SHA-256 plus the original archive hash. Only the agent reference filename changes.
- Root `AGENTS.md` matches its pre-import checksum. No nested active `AGENTS.md` is imported.
- Relative links in the changed planning/index/reference documents resolve locally.
- Current-state claims use actual Rust routes/models and current Nextcloud/MCP client transport code.
  Required mobile capabilities are explicitly proposed rather than labelled implemented.
- F35 and ADR 0026 remain Draft/Proposed. Existing F18/F29/F30 requirements retain their status and
  unchecked work. The client folder does not exist; all implementation tasks are still unchecked.
- The only executable-source change is generated rustdoc metadata. No Flutter/domain implementation,
  pipeline change or test-instance mutation is included.

## Mandatory security-gate maintenance

The initial `cargo audit` refreshed the advisory database and found three issues in the pre-existing
lockfile: `RUSTSEC-2026-0523` and `RUSTSEC-2026-0524` in `quinn-proto 0.11.18`, and
`RUSTSEC-2026-0538` in `quinn-udp 0.5.15`. The official `RustSec` records specify patched versions
`quinn-proto >=0.11.19` and `quinn-udp >=0.5.16,<0.6.0` (or the separate newer minor branch).

Use a separate maintenance commit for the minimal generated lockfile updates to **0.11.19 / 0.5.16**.
Cargo also refreshes existing Windows-target dependency edges; no direct manifest, feature selection,
new crate, advisory ignore or licence exception is added. `cargo tree --locked -i quinn-proto` reports
no active reverse dependency for the current host/features; that does not waive the whole-lock audit.
No application or phone behaviour is inferred from a passing security gate.

Primary records were read from the refreshed official advisory checkout (origin
`https://github.com/RustSec/advisory-db.git`):

- [RUSTSEC-2026-0523](https://github.com/RustSec/advisory-db/blob/main/crates/quinn-proto/RUSTSEC-2026-0523.md)
- [RUSTSEC-2026-0524](https://github.com/RustSec/advisory-db/blob/main/crates/quinn-proto/RUSTSEC-2026-0524.md)
- [RUSTSEC-2026-0538](https://github.com/RustSec/advisory-db/blob/main/crates/quinn-udp/RUSTSEC-2026-0538.md)

## Publication checks

Run the complete required repository gate after the patch updates, plus generated-doc synchronization,
strict rustdoc and the import/link/status checks above. Record actual command/results in the PR; final
head CI must pass before handoff. A passing Rust/docs pipeline does not establish Flutter functionality.
Physical-phone build, installation, two-client offline and OS notification proof belong to P1–P5.

## P1 implementation evidence

P1 implementation is in this PR; F35 remains **In progress** for P2–P5 and the physical-handset
acceptance. No handset is attached. Android APKs and a working visible accelerated emulator are
provided; emulator evidence is not relabelled as physical-phone evidence.

### Automated gates

- Full required catalogue/import/verifier/i18n/fmt/Clippy/nextest/licence/advisory gate; strict rustdoc
  and generated-document synchronization. The suite now contains **186 Rust tests**, including five
  mobile receipt/group tests: restart/replay, concurrent duplicate check, authority removal, stale
  contribution scope, expiry/capacity/corrupt receipts, purchase-only-new-source and legacy projection.
- Flutter format and strict analysis; **22 unit/widget tests** cover exact localized input, original
  precision, anonymous Login Flow polling, origin/prefix/redirect boundaries, frozen outbox after
  lost/malformed responses, durable-write failure, revocation, explicit first household choice,
  household removal while a draft is open, restart and local purchase state. Three shop-menu scenarios
  verify that starting a round requires consent and no already active round, matching server constraints.
- Real API 36 Android integration through `python3 scripts/mobile.py integration`: two named native
  tests (plus framework teardown) pass. They exercise the Android Keystore/atomic-file adapter and
  native German shopping/navigation with synthetic HTTP transport; this is separate from live login.
- Owner/reader shopping and connection at **320 dp and 200% text** in DE/EN; Android 48-dp tap target
  and labelled-target guidelines, group labels, pending checkbox semantics and error layout pass.
  These checks do not claim a manual `TalkBack` audit or complete WCAG certification.
- Existing Nextcloud client: **36 Vitest tests**, production type-check/build, npm audit (no
  vulnerabilities), and **five shopping Playwright tests** against the actual test instance pass.
- `python3 scripts/verify_mobile_packages.py --audit`: 53 locked package licence records; 45 external
  Pub package versions, no matched OSV advisories. The built APK includes the root AGPL notice.
- The seven imported ZIP records still match byte for byte; root `AGENTS.md` retains its original hash.

### Live Android / Nextcloud session

Environment: Nextcloud **35.0.2**, `AppAPI` **35**, own Google APIs Android 16/API 36 r07 Pixel 6 AVD,
Emulator 37.2.13.0, KVM and local hardware GPU, system Chrome **133.0.6943.137**. `SwiftShader` triggered
an independently traced guest OpenGL-pipe deadlock/system-server watchdog on this NVIDIA host; the
helper supports `--host-gpu` without clearing AVD data. CI uses its own software-rendered guest.

The following checks use only the ledger-owned synthetic `flutter-p1` household; retained customer
household directories are never inspected, copied or used as fixtures. Browser authentication is
operated using field positions and fixed button labels only; no authentication screenshot, raw
browser URL, poll token or app password is retained in evidence.

1. Enter the configured loopback origin in the native app, launch Login Flow v2 in system Chrome,
   submit the authorized test account in that browser, grant device access and return to Mealshop.
   Choose the own synthetic household. The app uses the resulting device credential through the
   real `index.php/apps/app_api/proxy/mealshop` front controller, never the `ExApp` secret.
2. Add **200.000001 g** natively. Compare the live legacy proxy snapshot: original quantity and
   remaining quantity retain every decimal digit. Native purchase and undo each match server state.
3. Remove only the emulator's port reverse, force-stop and restart the app. Last household and
   encrypted cached list remain. Add a separate **2 pcs** entry offline, check that draft locally,
   queue a check for the earlier item, force-stop/restart again and confirm the pending checkbox.
   The server still has only the original source while disconnected.
4. Change the original source to **250.000001 g** from a second authenticated proxy client. Restore
   the reverse and refresh natively. Only the new draft's 2 pcs are purchased; changed original
   demand remains entirely unpurchased, with a visible conflict. Explicit native review/confirmation
   then purchases the reviewed 250.000001 g. No blind rebase or duplicate source occurs.

5. Select the actual type-550 ingredient through the native search, add **30 g**, and verify its
   existing wheat-flour group in the live minimized projection. Switch the app to German.
6. Install the retained-key build **0.1.0+2** over **0.1.0+1** (`x86_64` Android version codes **4002 over
   4001**) using `adb install -r`. The existing device credential, selected household and three-source
   shopping snapshot survive. No browser approval or data reset is needed for this actual update.
   This check runs separately from the integration driver, whose cleanup intentionally uninstalls
   its test application.

Visually inspected own-fixture native screenshots: [light](screenshots/android-shopping-light.png),
[dark](screenshots/android-shopping-dark.png), [200% text](screenshots/android-shopping-large-text.png).
System theme/font settings are restored after capture. Screenshots contain only the synthetic household
and shopping data, not browser authentication or retained customer content.

Network loss above is simulated by removing ADB reverse forwarding; it is not a claim of an airplane-mode
OS test. Lost-response replay/restart and permission revocation are additionally proved by executable
server/controller tests. Manual product entry/shops reuse the existing server contracts; consented
Open Food Facts lookup remains dependent on the external service. Camera capture, background sync,
production signing, iOS, physical handset/`TalkBack`/performance profiling and P2–P5 are not claimed.

### Cleanup and artifact identity

Only the six fixtures created in this run (one mobile fixture, four shopping fixtures and one visual
shopping fixture) were selected from the ledger. With the sole backend writer stopped, the strict
fixture cleanup archived only those synthetic records; retained household membership/library entries
were compared unchanged, and retained household directories were not inspected. The backend returned
healthy. The native app was signed out and its production protected record was confirmed absent;
temporary UI-probe/optional browser packages were removed from the owned emulator. The normal app,
Chrome and the working emulator remain available.

Local handset APKs use the retained development key and build **0.1.0+2**, with actual manifest version
name/code, source commit, dirty-source flag and verified SHA-256 alongside each artifact. The first
complete PR CI run passed Rust, Vue, MCP, image/rootless TCP+Unix, security, Android native tests and APK
upload; the final-head CI is required again after this evidence/version-metadata update. No signing
key, device credential, raw UI dump or private household data is committed.


## P2 implementation evidence

P1 was merged as PR #44. P2 supplies Flutter, Rust and compatible Nextcloud editing in one PR;
F35 remains **In progress** for P3–P5 and the separately recorded physical-handset acceptance.
Customer decisions: activity permissions belong to people, and owners administer grants without
implicit read/edit access. [ADR 0028](../../docs/adr/0028-person-rules-and-activity.md) and the
[API/privacy guide](../../docs/guides/people-activity.md) record the exact boundary.

### Local checks

- All AGENTS gates: catalogue import check/verifier, Node i18n tests, Rust format, strict Clippy,
  **189 Rust tests**, deny licences, audit; generated docs and strict rustdoc also pass.
  A project-local Rust 1.85.0 `check --workspace --all-targets --locked` additionally passes;
  explicit crate paths avoid the older compiler's nested parent-import resolution difference.
- Flutter format/strict analyze and **35 unit/widget tests** pass. New coverage includes minimized,
  scoped, expiring drafts, durable restart, revoked membership, pending invitations without content/a selected household, capacity rejection without losing the
  previous draft; localized search/focus/inline-create entry, original language/exact multipart recipe
  round trip, 320dp/200% DE/EN recipe ranges, activity duration without weight and clearing a revoked form.
  Further regressions cover localized decimal fields/ranges/constituents, rejecting the other locale’s
  separator, saved-editor completion/result propagation and immediate scope closure during durable logout.
- **68 locked package licences**, **60 external Pub versions** checked with OSV, no matched advisory.
  `image_picker` 1.2.4 is the only new direct package, from the official Flutter publisher. No new Rust crate.
- **39 Vue tests**, strict typecheck/production build, Prettier and production npm audit pass.
  Five live shopping regressions plus the new rule-editor browser journey pass against Nextcloud;
  mouse selection, save/reload and Axe WCAG tags cover the new editor.
  Profile rules/avatar and omitted-vs-explicit production declarations survive typed legacy round trips.
- Native API 36/Pixel 6: **11 tests pass**, including the P2 journeys and protected-storage/shopping
  regressions. The helper prebuilds the verification APK and passes a private stable copy to Flutter drive only after checking its manifest package ID; target/binary/device overrides are rejected.
  This is necessary: deriving a package from stale project metadata selected the development app for
  driver cleanup. Retesting with an exact merged P1 build proved the corrected driver preserves its
  package, encrypted record, login and selected synthetic household.

### Real Nextcloud 35 and Android proxy journey

Only freshly generated ledger-owned fixtures were used. Retained customer household directories were
not inspected, copied or backed up. The existing Nextcloud 35.0.2/AppAPI installation served the locally
built P2 backend and Vue assets; Android connected through Login Flow v2 and the real `index.php` proxy.

The live contract run created two synthetic people, a declared household-produced ingredient with a
nutrient range and optional constituent, an image/avatar, and an explicit own-ingredient exception.
It proved owner denial before a self-grant, mandatory self-confirmation, reader read/edit separation,
45-minute running and 60-minute cycling without weight, null energy, export/profile isolation,
pagination and denied old grants after leaving and rejoining. It also proved exact recipe scaling,
prepared-part links, reverse ingredient lookup, pinned collection/shared revisions and source-link-only
imported projections. A retry of guarded recipe creation cannot duplicate after a lost response/restart.

The exact merged **P1 0.1.0+2 / `x86_64` version 4002** was built separately with the retained development
key, connected to the synthetic household and cached **200.000001 g**. With emulator forwarding removed,
**P2 0.2.0+3 / 4003** was installed using `install -r`; account, selected household, encrypted record and
exact cached amount remained. Forwarding was restored for online library checks. No application data
reset or uninstall was used for this upgrade. Instrumentation uses its own application package.

Synthetic native views were visually inspected: [shopping](screenshots/p2-shopping.png),
[recipe library](screenshots/p2-recipes.png), [recipe detail](screenshots/p2-recipe-detail.png),
[activity people](screenshots/p2-people-activities.png) and [history](screenshots/p2-activity-history.png).
No authentication page, arbitrary UI tree, private household content or real health entry was captured.

Physical handset, `TalkBack`, release-mode frame/memory profiling, iOS, camera/OCR, device reminders,
full native plan/stock/cooking and P3–P5 are not claimed. Gallery entry and document export use Android
system UI; OS cancellation leaves the owning form intact. Library/activity writes remain online-only.
Android gallery cancellation was exercised on the synthetic recipe; its image and exact input survived
native save. The system document export produced the two synthetic activities with null energy; the
uniquely named test file was verified in memory and removed.
The emulator's physical keyboard is enabled persistently; Android reports its alphanumeric keyboard.

All twelve recorded synthetic households from this run were archived with the sole writer stopped.
Retained household directories were not inspected or copied; retained membership/library/epoch
metadata was checked unchanged and allocation counters were preserved. The emulator returned to
explicit household selection after the removed fixture lost access.
Final PR-head CI and clean-source APK metadata are recorded in the PR description.

## P3 implementation evidence

P2 was merged in PR #45; P3 is implemented for review in PR #48 on
`story/flutter-week-preparation`. F35 stays **In progress** for P4/P5 and physical-handset acceptance.
AC4/AC7, [ADR 0029](../../docs/adr/0029-mobile-week-and-prepared-stock.md) and the
[mobile planning guide](../../docs/guides/mobile-planning.md) define the delivered boundary.

### Local and CI checks

- All mandatory AGENTS gates pass: catalogue import/verifier, keyed i18n, Rust format, strict Clippy,
  **198 nextest tests**, licences and audit. Generated documentation and strict rustdoc also pass.
  Backend cases include ordinary/date-override attendance, hard exclusion/infeasibility, per-slot
  confirmations, stale recipe heads/expiry/restart, harmless applied replay, current shopping previews,
  physical over-allocation/duplicate output and protected production.
- Flutter format/strict analyze and **48 unit/widget tests** pass: retained destination, civil leap/DST
  navigation, day/week 320dp/200% reader layouts, exact localized stock input/output, observed-yield
  preview, retained conflict drafts, explicit current-stock comparison, fresh shop directory/exact
  German offer price and identical frozen-key/body retry after an ambiguous result.
- **39 Vue tests**, strict typecheck/production build, formatting and production npm audit pass.
  Seven unique live browser regressions pass against Nextcloud: five inventory cases, the profile
  rules editor and unified planned/physical preparation. The reader case was rerun with its required
  reader credential after an initial environment omission; it passes without changing the test.
- Pub audit: **68 locked licence records / 60 external package versions**, no matched OSV advisory.
  No new direct package or Rust crate. The rootless TCP/Unix image smoke check passes.
- All twelve CI checks passed for `9f6a673`, including **23 native API 36 tests**. The subsequent three
  stock form/conflict regressions also run in the native suite; the final-head CI result and downloadable
  artifacts are recorded in the PR description. No local integration-driver uninstall/reset was used.

### Actual Nextcloud 35 / Android workflows

The existing Nextcloud 35.0.2/AppAPI test instance runs the P3 backend and Vue assets. The retained
Android 16/API 36 Pixel 6 AVD connects through the real authenticated `index.php` `AppAPI` proxy.
Only this run's ledger-owned synthetic fixtures were used; retained household directories and the
phone's protected record were never inspected, copied or hashed.

1. The live proxy exercise uses two synthetic people and three recipes, proposes a seven-slot week,
   skips one person's Thursday attendance, denies reader writes, confirms all six feasible meals and
   verifies harmless replay. A second-client Thursday swap preserves other plan sources, manual
   entries and exact purchased quantities. Impossible/unreviewed slots remain explicit.
2. Exact mass/volume lots, a sourced manual offer, reservation, actual **six = four held + two available**
   prepared portions, duplicate/over-allocation guards, storage change, eating/correction and request
   replay match server state. Prepared output creates no raw-lot movement.
3. An exact retained-key P2 build **0.2.0+3 / `x86_64` 4003** was installed and connected to the owned
   fixture with **200.000001 g**. With emulator forwarding removed, ordinary `install -r` upgraded it
   to P3 **0.3.0+4 / 4004**. Account, selected household and exact cached amount survived offline;
   forwarding was restored. No app data reset, uninstall or vault inspection was used.
4. The installed native UI records eating and its explicit correction; server counts change from
   four held/zero used/two available to two held/two used/two available and back. The native next-week
   review preserves two physically allocated slots and individually confirms/applies the other five.
   An initial UI-driver visibility failure was corrected before the successful workflow; no product
   bypass or direct API substitution was used for these native actions.
5. Actual system dark mode and a **320dp / 200% system font** emulator configuration were visually
   reviewed. Original density/font/theme settings were restored. Unit/widget semantics and layout
   checks complement these screenshots; this is not a manual `TalkBack` or WCAG certification claim.

Reviewed synthetic screenshots: [shopping](screenshots/p3-shopping.png),
[Today](screenshots/p3-today.png), [week/day](screenshots/p3-plan.png),
[physical allocation](screenshots/p3-plan-allocations.png), [week review](screenshots/p3-week-review.png),
[six/four/two stock](screenshots/p3-prepared-stock.png), [dark mode](screenshots/p3-plan-dark.png),
[320dp/200% text](screenshots/p3-plan-large-text.png). No authentication or retained customer content
was captured. The ordinary app and physical keyboard remain available in the working emulator.

### Cleanup and artifact identity

All **nine** households created in this run (one P3 workflow and eight browser fixtures, including the
reader rerun) were archived with the sole writer stopped. Exact ledger ownership and synthetic content
were checked before cleanup. Retained membership/library/epoch metadata and allocation counters were
preserved in memory; retained household directories were never opened, copied or hashed. Normal
refresh removed the archived selected fixture from the app. The temporary verification helper was
removed; the ordinary app and retained development signing key remain.

The three local ABI APKs retain the same application ID/development key, version **0.3.0+4** and
per-ABI manifest version code, source commit, clean-source flag and SHA-256 metadata. Their final
identities, exact deployed source and final CI are recorded in PR #48. Untrusted CI uses its own
independent ephemeral key; its artifacts are fresh-install tests, not upgrades over the retained key.
No signing key, device credential, raw UI tree, fixture archive or private household record is committed.

Planning/stock writes are online; the protected shopping outbox remains unchanged. Physical handset,
manual `TalkBack`, release frame/memory profiling, independent personal plans, cooking screens,
notifications/camera and P4/P5 are not claimed by this PR.

### P3 diagnostic performance case

The same isolated in-process debug test creates 100 candidate recipe revisions for two synthetic
people and proposes all 28 empty meal slots. On this host, the proposal call alone measured
22.683 s before request-local assessment reuse and 1.316 s after it; fixture creation is excluded.
These are one diagnostic sample per implementation, not a release/device/network benchmark. All
28 results remain canonically feasible; ordinary/date-override/physical-stock/stale/replay tests pass.
The bounded cache contains only immutable per-request assessment facts; fresh slot constraints and
apply-time checks remain mandatory.


## P4 implementation checkpoints (in progress)

The customer merged P3 in PR #48 and authorized P4 in one frontend/backend PR. Branch:
`story/flutter-cooking-fermentation`, base `d3b6dc7`. No retained household data is used as fixtures.

First cooking checkpoint: seven Rust cooking contracts pass, including immutable request replay after
lost response/restart, explicit reported-action review without publication and exact per-step ingredient
scope. Strict workspace Clippy passes. Food actions use the existing atomic 4 MiB process store;
normal reads exclude private receipt keys. Preview and apply take plans before cooking, matching
inventory lock order and rechecking a linked production before publication.

Flutter format/analyze and **56 unit/widget tests** pass at this checkpoint. New cases cover exact
request restoration, ordered independent offline intents, wall-clock change/new boot, generation-based
private reminder projections, revocation cleanup, explicit schedule confirmation and always-open
step ingredients at 320dp/200% German text. The Android platform adapter compiles into all three ABI
APKs; actual OS permission/background/reboot delivery checks remain pending. No local integration
uninstall/reset is used. Final source/APK identities and full gates are still required.

Remaining P4 checkpoints: combined preparation-calendar entry points, complete native brewing/history/
learning flows, actual Nextcloud/emulator journeys, retained-key upgrade, synthetic fixture cleanup,
all repository/browser/security gates and current-head CI. P4 is not marked delivered by this checkpoint.


## P4 implementation evidence

Status: implementation/local verification in progress on `story/flutter-cooking-fermentation`.
This section will be completed with real test-instance/Android proof and exact-head CI before the PR
is ready. No physical handset, `TalkBack`, release-mode performance or guaranteed notification delivery
is inferred from these development checks.

- `python3 scripts/mobile.py flutter analyze`: no issues; `python3 scripts/mobile.py test`: 63 tests.
  Recovery covers immutable lost-response replay after restart, independent offline timer intents,
  wall-clock/new-boot distinction, damaged envelopes, local-copy deletion, reader controls, original
  readings/corrections, explicit manual history unknowns and 320dp/200% learning/planning views.
- Mutation review: inverting the cache-version guard fails the actual lost-cooking-response/restart
  test; the source was restored, and analysis plus the full 63-test suite passed again. The malformed
  request test alone is intentionally insufficient for a valid-cache version inversion.
- Mandatory catalogue, i18n, format, strict workspace Clippy, 204 nextest tests, licence, vulnerability
  and strict generated-rustdoc checks passed. Cooking has nine integration cases, brewing eleven.
  Calendar cases prove reader authorization, pagination/range bounds, actual timer deadlines and
  separate manual fermentation. Legacy Vue callers retain their contract.
- Nextcloud Vitest: 39 tests/12 files; Vite production build and production npm audit passed.
  Flutter package verification: all 68 locked licences and OSV's 60 external Pub packages passed.
  No Rust crate, Flutter package, new font or design system was added.
- All three development APKs compile with the retained key. Their interim dirty-source metadata is
  not final delivery identity; final clean-source versions/checksums belong to the P4 PR.
