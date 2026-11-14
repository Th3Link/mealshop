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

## P3 implementation verification (in progress, 2026-11-14)

P2 was merged in PR #45; this slice is `story/flutter-week-preparation`. Acceptance remains
F35 AC4/AC7, with ADR 0029 and the mobile planning guide as the explicit protocol/design contract.

- Focused backend planning/inventory tests cover civil attendance, hard exclusion/infeasibility,
  server-owned confirmations, stale recipe heads/expiry/restart, harmless applied replay, actual
  shopping preview/manual purchases, six/four/two prepared portions, allocation/use/release/storage/
  reversal, duplicate linked output and protected production.
- Flutter format/analyze and 44 unit/widget tests include P3 review/conflicts, immutable ambiguous
  stock retry, retained destination, exact localized stock, observed-yield preview and 320dp/200%
  reader/physical-stock layouts. Full final-head gates are still pending below.
- [ ] Live Nextcloud proxy and two-role/second-client P3 journey with this run's synthetic household.
- [ ] Retained P2-to-P3 Android upgrade, screenshots and actual emulator workflow.
- [ ] Remove/archive only P3-owned fixtures, leaving retained records/directories untouched.
- [ ] All AGENTS gates, native Vue regressions, package audit and final-head CI/APK identity.

Physical-handset, TalkBack/release profiling and P4/P5 are not claimed by these local tests.

### P3 diagnostic performance case

The same isolated in-process debug test creates 100 candidate recipe revisions for two synthetic
people and proposes all 28 empty meal slots. On this host, the proposal call alone measured
22.683 s before request-local assessment reuse and 1.316 s after it; fixture creation is excluded.
These are one diagnostic sample per implementation, not a release/device/network benchmark. All
28 results remain canonically feasible; ordinary/date-override/physical-stock/stale/replay tests pass.
The bounded cache contains only immutable per-request assessment facts; fresh slot constraints and
apply-time checks remain mandatory.
