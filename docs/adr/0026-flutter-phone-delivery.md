# ADR 0026: Flutter phone delivery and the first offline boundary

Status: Accepted — customer Android/emulator decision and explicit implementation Go, 2026-11-13
Date: 2026-11-13

## Context

The customer supplied a mobile product pack and requests fewer, larger PRs which can be tried on a
smartphone from the start. Flutter is already the app target (N01 / ADR 0001). `main` now has substantial
Rust/Nextcloud domain functionality, but no Flutter application, mobile login or shopping outbox.
The source pack explicitly describes a target, not an audited inventory. Its agent rules must not
replace the repository's `AGENTS.md`.

Existing shopping revisions prevent stale writes but do not provide durable request replay. A manual
add with no source ID allocates a new source; a lost successful response followed by a fresh retry can
repeat intent. Timers have canonical server deadlines, while Nextcloud notification delivery is not
native mobile push. These risks must be resolved in their user-facing slices, not hidden by UI mocks.

## Alternatives

1. Port the pack in its listed order, starting with a five-tab shell/profile tour: easy to scaffold,
   but delays a useful phone workflow and discovers transport/offline failures late.
2. Separate backend, sync framework and each screen into many small PRs: locally reviewable, but
   conflicts with the requested larger smartphone-testable deliveries and leaves disconnected pieces.
3. Rewrite the server or build all adapters/hosted authentication first: discards working contracts and
   greatly delays a usable Nextcloud app without a corresponding accepted requirement.
4. Deliver five coherent end-to-end PRs with smaller internal commits and explicit install evidence.

## Decision

Choose option 4. P1 connects a real native Flutter app to Nextcloud and completes independent shopping;
P2 adds people/content/activity, P3 week/physical mealprep/stock, P4 cooking/fermentation/history, P5
camera/share/OCR review. All required Rust, Nextcloud compatibility and phone changes stay in that
feature's PR. Do not implement a placeholder tab, `WebView` product shell or a new general sync framework.

Use `clients/flutter/`, feature-oriented UI/state modules and focused API/cache services. Keep Rust as
canonical domain/authorization/calculation/scheduling authority; Dart retains exact DTO values and local
presentation/intent. Pick concrete maintained packages, toolchain versions and cache migrations within
P1 with licence/OS review, rather than installing a speculative dependency set in this planning PR.

Prefer Login Flow v2 in the system browser, device-specific revocable Nextcloud app credentials in
platform-protected storage, and authenticated proxy access. Retain installation prefixes and validate
returned login/poll origins. No client receives `ExApp`/operator secrets or embeds the user's master
password. Probe actual proxy writes, permissions and revocation on Nextcloud 35 before claiming support.
Standalone/hosted identity remains its own explicit unresolved contract.

P1 adds bounded atomic shopping receipts before automatic offline write replay. Scope keys and cached
records by canonical server/account/household, preserve the reviewed request, recheck current authority,
and distinguish pending/confirmed/conflict. Do not rebase a check-off over unseen new contributions.
Local persistence and receipt publication are one domain-specific contract, not global last-write-wins.
Later offline domains must audit their own replay and irreversible effects before being enabled.
Offline devices cannot observe immediate remote revocation: define bounded cache retention and local
protections, revalidate on reconnect, and stop pending writes/clear protected data when revocation is known.

Android APK is the accepted initial distribution path, with an accelerated Android emulator explicitly requested. Establish
application ID, stable development signing/upgrade path and physical-device reachability in P1. For
iPhone-first delivery, establish macOS/Xcode/provisioning there instead; an unsigned build is not phone
acceptance. Do not leak signing keys to untrusted PR jobs. Protected distribution/local retained-key
builds provide upgradeable phone artifacts; ordinary untrusted CI still compiles and tests without keys.

## Consequences

Every implementation PR has immediate phone value, explicit dependencies and regression evidence.
The planner and mobile notification work remain larger separate review units because they introduce
different correctness/OS risks. Each can contain thin, tested commits without multiplying customer PRs.

The imported source remains byte-preserved in `references/mobile-product-spec-2026-11-12/`; its agent
file is renamed `.md.reference` to prevent activation. F35 is In progress after the explicit Go. F18, F29,
F30, hosted admission and unimplemented storage adapters retain their honest independent statuses.
P1 adds the Android app and shopping receipts under ADR 0027; the preserved source remains unchanged.

## Primary references

Reviewed on 2026-11-13; they support the proposed approach, not a claim of completed integration:

- [Nextcloud 35 Login Flow v2](https://docs.nextcloud.com/server/stable/developer_manual/client_apis/LoginFlow/index.html#login-flow-v2): system-browser device credential acquisition.
- [Flutter offline-first](https://docs.flutter.dev/app-architecture/design-patterns/offline-first): separate cached/remote access and deliberate synchronization semantics.
- [Flutter Android distribution](https://docs.flutter.dev/deployment/android): APK build, signing and device installation.
- [Flutter iOS distribution](https://docs.flutter.dev/deployment/ios): macOS/Xcode, provisioning and publishing requirements.
- [Android Debug Bridge](https://developer.android.com/tools/adb): device connection and installation tooling; verify loopback reverse forwarding during P1 setup.
