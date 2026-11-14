# Android app development and installation

F35 P1 provides native shared shopping against the existing Nextcloud installation. Rust remains the
authority for quantities, membership and shopping transactions. P1 delivered Shopping/More and P2 delivered Recipes and people. P3 activates all five
destinations: Today, Plan, Shopping, Recipes and More. [Planning and stock](mobile-planning.md)
documents the P3 server/client contract; cooking and capture remain P4/P5.

## Toolchain and emulator

The customer selected Android APKs and requested an actual emulator. The pinned toolchain is in
`clients/flutter/toolchain.json`: Flutter 3.50.0/Dart 3.14.0, JDK 17, Android compile/target 36,
minimum 24, NDK 28.2.13676358, API 36 Google APIs `x86_64` Pixel 6 emulator. Linux `x86_64` and working
`/dev/kvm` access are required by the local setup helper. It verifies official download checksums,
keeps tools under ignored `var/mobile/`, accepts the Android SDK package licences for this development setup, and never resets Nextcloud or another AVD.

From the repository root:

```sh
python3 scripts/mobile.py bootstrap
python3 scripts/mobile.py doctor
python3 scripts/mobile.py emulator --host-gpu
python3 scripts/mobile.py status
python3 scripts/mobile.py browser
python3 scripts/mobile.py build
python3 scripts/mobile.py install
```

Wait for status `1` and `Mealshop_API36` before installing. The `browser` step selects the Chrome
browser supplied by the official Google APIs system image, only inside this AVD. No personal browser
profile or account synchronization is used. A compatible external browser is required for Login Flow v2.
The abandoned API 35 guest had system-service/FUSE failures; its data was retained. The default/CI uses
`SwiftShader`; `--host-gpu` selects local hardware rendering. Both disable Vulkan and `GuestAngle`. On this
NVIDIA Linux host, `SwiftShader` blocked the guest OpenGL pipe and caused system-server watchdog
restarts; the visible emulator uses `--host-gpu`. Stop only this AVD before changing renderers; retain
its data. These are emulator settings, not application rendering changes.

`install`, `browser` and `integration` verify the AVD identity, restore port forwarding, supply AC power
and wake/unlock only this emulator. A cold restart otherwise loses its reverse forwarding.

The emulator uses its own ADB server on
5038 and console port 5556, leaving the default ADB server alone. Use `emulator --headless` in CI;
the default opens a real emulator window. Physical keyboard input is enabled in the AVD: click a
text field and use the desktop keyboard. The helper retains an existing AVD. The pinned old
`avdmanager` compatibility command creates only this API 36 AVD; package installation uses the new
Android CLI with metrics disabled. Tools/downloads/keys and emulator state are ignored, never committed.

`install` forwards emulator loopback port 8080 to the host's 8080. Enter `http://127.0.0.1:8080` in
this debug build. An emulator's loopback alone does not reach the host. Real installations use a
configurable HTTPS Nextcloud URL, including its installation prefix. Do not disable TLS verification.
Only debug manifests and client validation allow explicit loopback HTTP; arbitrary LAN HTTP is denied.

## APKs and stable upgrades

The development application ID is `org.mealshop.pocket.dev`; display name Mealshop. The helper creates
and retains `var/mobile/development.keystore`, owner-only, for local development upgrades. This is a
development identity, with Android's standard development key password, not production signing.
Back up the key privately if its installed development app must survive moving this checkout.
Never commit or upload it. Losing the key requires uninstalling that development app and loses its
local credential/cache. Production release signing and store distribution remain separate work.

`python3 scripts/mobile.py build` writes:

- `clients/flutter/build/app/outputs/flutter-apk/app-arm64-v8a-debug.apk`: most current Android phones.
- `app-armeabi-v7a-debug.apk`: supported older ARM phones.
- `app-x86_64-debug.apk`: this emulator.
- Matching `.apk.sha256` and `.apk.build.json`: checksum, source commit, dirty-source flag, app ID and actual APK version name/code.

Install an upgrade with `adb install -r <apk>` using the same retained signing identity. Do not uninstall
or clear application data as an upgrade step. Debug build numbers can be overridden using the Flutter
command passthrough if needed. Physical-device USB installation requires that device's USB debugging
and authorization; no physical handset was supplied to the agent. Emulator and real-phone evidence
are distinct in F35 verification. For this local HTTP-only test instance, an explicitly authorized
USB-connected phone can use `adb -s <phone-serial> reverse tcp:8080 tcp:8080`, then enter the same
loopback origin. Install the matching ARM APK with `adb -s <phone-serial> install -r <apk>`. The helper
intentionally targets only its owned emulator; it never automatically changes a physical phone.

The GitHub Android workflow analyzes/tests, runs an accelerated emulator and uploads APKs. Each job
uses an ephemeral development key, with **no retained distribution key in an untrusted PR job**.
Those CI APKs are useful fresh-install artifacts, but are not upgrades of a locally signed app or a
different CI run. The retained local build supplies the continuous handset trial path.

## Sign in and choose a household

Enter the installation URL and continue in the external system browser. Nextcloud Login Flow v2
creates this device's revocable app password; the app never asks for your master password and never
receives the `ExApp` secret. Login/poll targets and the returned canonical installation must match the
configured server origin/prefix. Credential-bearing redirects are rejected. Authenticated API calls use the explicit `index.php`
front controller, so installation prefixes work without assuming Pretty-URL rewrites.

Return to Mealshop after approving access. Select or create a household, or accept an invitation.
The last selected household is remembered. With several households on a first login, choose explicitly;
the app does not automatically load the first private household. Reader roles have no mutation actions.
More contains account/server details, household switching, language, shops/order, products, pending
changes and sign-out. Sign-out deletes the protected local record and its encryption key. Revoke the
app password in Nextcloud account security to remove the device's server access as well.

## Shopping and offline behaviour

Add a free-text item or select a real ingredient. Existing ingredient groups supply localized labels and
a group filter; unclassified free text remains unclassified. Shop ordering is preserved within filters. Enter an exact positive quantity with up to six
decimal places and an explicit unit. German uses a comma and English a dot; thousands separators
are not accepted. No binary floating point or implicit mass/volume conversion. Separate purchased
amounts remain visible, with undo; item details show each manual/recipe/plan/product/offer source.
Manual sources can be edited or removed without rewriting purchase provenance.

Assign shops in item details; manage shops, ordering, shopping rounds and opt-in order learning in
More. Products support manual GTIN/check-digit input, package quantity, reviewed ingredient matching,
existing saved products and explicit consent before Open Food Facts lookup. Lookup evidence retains
its original label, source and licence. Camera scanning follows P5.

A locally saved add/check is visibly pending until the server confirms it. New offline entries
appear directly in the list and can be checked before first transmission. The server then creates
and purchases only that new source atomically; older demand for the same ingredient stays untouched.
An already attempted addition must be reconciled before changing its local purchase flag. Offline changes survive
process death in an AES-256-GCM encrypted bounded record, with a non-exportable Android Keystore key.
The native adapter uses `AtomicFile` and explicitly syncs file contents and directory metadata before
acknowledging a save. Backup/device transfer is disabled. No household content, app password or login
poll token is logged; the poll token remains in memory only.

On reconnect the app refreshes authority and shopping data before replaying the frozen request.
The server publishes effects and an actor-scoped operation receipt atomically. A successful request
whose response was lost is replayed using the same ID/payload/revision, including after restart.
A changed revision stops the request for explicit review; it never silently checks newly added demand.
Do not discard an unknown outcome: retry it safely first. Source edits, shops, ordering and product
management require an online connection; only manual additions and check/undo are queued offline.

The record is bounded to 1 MiB and 100 pending changes per household. Cache and intent lifetime is
seven days; expired local shopping data is removed. Receipts are retained eight days, at most 1,024 per
household and 8 KiB per command. Capacity errors preserve existing data. The server rejects expired
intent even after receipt pruning. A known revocation clears the affected household cache; a disconnected
device cannot learn immediate remote revocation. Storage errors stop writes rather than pretending
that a volatile change is durable. The app refreshes in the foreground and on resume; no background
execution or live push is promised.

## Verification and dependency review

```sh
python3 scripts/mobile.py flutter pub get --enforce-lockfile
python3 scripts/verify_mobile_packages.py --audit
python3 scripts/mobile.py flutter analyze
python3 scripts/mobile.py test
python3 scripts/mobile.py integration
python3 scripts/mobile.py build
```

Run integration tests on the dedicated synthetic-data emulator, not on a personal installation: the
official Flutter integration driver installs/replaces its test app and **uninstalls it during cleanup**,
including after success. This intentionally clears that installation’s local key/cache; perform the
separate same-key APK-upgrade test only after restoring and signing into the normal app.
`integration_test` runs through the official `flutter drive` integration driver with `--no-dds`.
Flutter 3.50's direct test runner failed to start DDS on this host, and its no-DDS golden-stream
subscription failed even though both native tests passed. The integration driver keeps every native
assertion and exits successfully without that unused golden subscription. It exercises real Android
keystore/file persistence and native shopping widgets with
synthetic transport; real Nextcloud browser login/proxy/offline tests are separately recorded in
`specs/0035-flutter-mobile-client/verification.md`. Mocked widgets alone are not transport proof.

`clients/flutter/dependencies.json` records all resolved Dart package versions, licence classification
and licence-file SHA-256. The verifier checks it against the immutable lock installation. `--audit`
sends only public package/version pairs to the OSV Pub advisory API; it never reads device data.
This inventory covers Dart packages, not every native SDK/engine notice; Flutter includes its platform
and package licence notices in the APK. SDK/JDK/emulator build tools are not shipped as app code.
The full repository Rust/catalogue/i18n/native-client gates still apply before push.

Do not downgrade a receipt-bearing shopping file into an older server writer. See ADR 0027 for
fail-closed downgrade/recovery behaviour and F35 for actual acceptance evidence and remaining work.

## P2: People, activities and the library

Recipes joins Shopping and More. More opens People, Activities, Ingredients, Collections and incoming
shares; household owners also manage outbound shares. Incoming shares remain reachable without a selected household, and pending invitations do not expose content before acceptance. The existing theme and DE/EN strings follow
the phone, with explicit source language retained in authored records. Library/profile/activity changes
require a live connection and server confirmation; shopping remains the P1 offline boundary.

Create a person through the resumable tour, review presets/exceptions, allergies/exclusions and
optional availability/active-time limits. A visible generic person label permits skipping a personal
name. Children need no login account. See [person rules and activity access](people-activity.md) for
the precise permissions, deletion/export, retention and legacy-client contract. Owners must explicitly
grant themselves activity access; a household reader may have person-specific edit access.

Recipe details keep ingredients, parts, dependencies and steps open. Serving changes scale only the
selected revision; normalized grams/kilograms keep exact decimal digits. Editing preserves original
language, quantity/conversion evidence, temperature extensions, handmade output links and imported
provenance. The ingredient search offers direct creation when a match is missing. Galleries use the
system photo picker; only JPEG/PNG/WebP up to 5 MiB are uploaded to the selected household. Cancelled
selection leaves the form intact. Interrupted picker results are consumed at startup and prompt a new
selection, never silently attached to a different account or draft. Camera/OCR and URL capture remain P5.

Ingredients support explicit groups/subtypes, declarations/inheritance, ranged or unknown nutrients,
optional constituents, composition, allergens/sorbitol, tags, favorites, archive and reverse recipe lookup.
Reference records and comparison sources stay visible. No unknown value is replaced by zero and no
constituent declaration implies a health benefit. Pinned collection revisions and shared invitations
use existing server contracts. Incoming imported recipes expose only the source link, including in
collections. Incoming shared content is never put into the offline cache.

`python3 scripts/mobile.py integration` runs shopping and P2 native journeys using the separate
`org.mealshop.pocket.verification` package. The helper verifies its manifest ID and uses a private stable APK copy, rejecting target/device/binary overrides. Flutter drive uninstalls that target; it cannot erase the
installed `org.mealshop.pocket.dev` record. Normal `build`/`install` still upgrade the development app
with its retained key. Do not run raw driver commands against the development package when preserving
an account/cache. Automated/mock native evidence and real Nextcloud proxy evidence are distinct.

## P3 development upgrade

Version `0.3.0+4` upgrades the same development package/key (`x86_64` version code 4004). It retains
the existing protected credential, household, shopping outbox and person drafts. The selected tab
is an optional protected field; older records default to Shopping. No plan/stock record is copied
to disk on the phone. P3 writes need a connection; shopping keeps P1 offline behaviour.

Normal installation inspects the actual APK application ID and uses `install -r`. The dedicated
emulator cold-boots retained data without loading/saving Quick Boot snapshots; physical keyboard
input stays enabled. Do not uninstall, reset the AVD or clear app data for this upgrade.
