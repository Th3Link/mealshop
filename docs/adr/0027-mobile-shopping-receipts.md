# ADR 0027: Android shopping credentials, storage and request receipts

Status: Accepted for F35 P1
Date: 2026-11-13

## Context

P1 must remain usable after a disconnected Android process restart. Existing shopping mutations use
an expected revision but cannot identify a successful request whose response was lost. The native
Vue contracts and shared shopping/inventory transaction must remain compatible. Android is the
customer-approved first platform, including a real emulator. The account credential, household
names and shopping content are private; no medical profile is needed for P1.

## Alternatives

- Retry ordinary shopping POSTs: can allocate a second manual source after response loss.
- Add a global sync engine or separate receipt file: unnecessary domain scope and non-atomic writes.
- Return new fields in all existing responses: risks strict existing client schemas.
- Use an explicitly versioned mobile route backed by the existing atomic shopping file.

## Decision

Use `GET/POST /api/households/{id}/domain/api/household/shopping/mobile`. GET returns version 1,
server time, maximum operation age, current role and the unchanged shopping snapshot. A separate
`ingredient_groups` map projects only group IDs, names and translations for ingredients actually in the list;
manual text stays unclassified. This metadata comes from existing explicit group membership, never inferred
nutrition or new category rules. Both GET and POST include it; the legacy snapshot remains identical. POST accepts an
operation ID, issuance timestamp, frozen expected revision and an existing shopping command. It returns
an immutable applied revision/operation ID and **separately the current snapshot**, including on replay.
A receipt does not claim that a later snapshot is the original historical response.

Check current household permission on every request, before inspecting any receipt. Namespace receipt
keys by household and authenticated actor. Reusing a key with a changed command, revision or issuance
time conflicts. Publish the command effects and receipt in the same existing clone/validate/fsync/rename
transaction, including the existing inventory reconciliation. No new Rust crate or storage writer.
An unsent new manual addition may carry `purchased: true`; create its source and exactly that
source's purchase atomically. It must never check older aggregated contributions. The client can
change that flag only before the first dispatch; attempted envelopes remain immutable.

Never rebase an attempted request. In particular, a stale aggregate check conflicts if another client
has added an unseen contribution. A new add without an existing source can bind the current revision
on its first dispatch; the serialized request is persisted before network dispatch and then immutable.

Accept intent age at most seven days (five minutes of forward clock tolerance). Retain receipts for
eight days from issuance; never evict a live receipt to make room. Limit to 1,024 receipts per household,
8 KiB per request and the existing 4 MiB shopping file. Capacity errors preserve all existing state.
Expired intent is rejected even if its receipt has been pruned, so it cannot become a new duplicate.
This is a bounded small-household delivery contract, not an unlimited operation history.

Use optional `mobile_receipts` in shopping schema v1, absent for untouched legacy stores. The new reader
validates them; existing response bodies omit them. An older binary using deny-unknown-fields refuses a
receipt-bearing file: do not downgrade that file into an old writer. Recover using a verified pre-upgrade
backup or a reviewed forward-compatible migration, never strip receipts while clients may retry.

Use Flutter 3.50.0 (official stable commit `ab2912035e160d9cad6306df307576da6128ebdb`), Dart 3.14.0,
the official extracted `material_ui` package, `http` and `url_launcher`.
A focused Android platform channel uses the OS keystore AES-256-GCM key and `AtomicFile`, explicitly
fsyncing contents and the directory before acknowledging a write. The reviewed secure-storage plugin
11.2.0 uses `SharedPreferences.apply` for ordinary writes, which acknowledges before durable disk write;
it was therefore removed before delivery. No iOS persistence implementation is claimed in P1.
Keep a single bounded protected application record (1 MiB) for credential, locale, household selection,
shopping snapshots and at most 100 pending intents per household. Avoid a new database/encryption layer.
Review/pin the complete package lock and licences in the mobile guide. No analytics SDK.

Acquire a revocable device app password using Nextcloud Login Flow v2 in the external system browser.
Keep poll token only in memory. Preserve installation prefixes; reject changed login/poll/credential
origins and credential-bearing redirects. HTTPS and normal certificate verification are mandatory;
only debug builds may use explicit loopback HTTP. Never collect a master password in the app or ship
an `ExApp` secret. Logging out removes the app's protected record; server revocation remains a Nextcloud
account action. Disable Android backup and device-to-device extraction of protected app data.

Only the selected account has local records, scoped to its configured origin/user/household. Bound
cache and intent lifetime to seven days; clear unauthorized household data after reconnect. A disconnected
device cannot observe revocation immediately. Freeze uncertain requests across restart, retry the same
ID after response loss, expose conflicts for explicit review and prevent silently dropping unknown
outcomes. Foreground polling/resume refresh is sufficient; no background execution promise.

## Consequences and verification

The UI can distinguish a saved local intent from a confirmed purchase. Permission removal and
concurrent demand cannot silently expand offline purchases. Storage failure stops transmission.
The protected-record limit can require finishing pending work before more entries are added.

Android API 36 Pixel 6 emulation uses KVM, a dedicated ADB port and its own AVD; API 36 build tools
follow the pinned Flutter SDK. A retained local development signing key supplies upgradeable test APKs;
untrusted PR builds use ephemeral keys. Production signing and iOS provisioning remain separate gates.
See F35 verification and the mobile guide for actual evidence; emulator tests are not physical-phone proof.

## Primary references

- [Nextcloud Login Flow v2](https://docs.nextcloud.com/server/stable/developer_manual/client_apis/LoginFlow/index.html#login-flow-v2)
- [Official Flutter Material package](https://pub.dev/packages/material_ui)
- [Dart HTTP package](https://pub.dev/packages/http)
- [Official URL launcher](https://pub.dev/packages/url_launcher)
- [Android Keystore](https://developer.android.com/privacy-and-security/keystore)
- [`AtomicFile`](https://developer.android.com/reference/android/util/AtomicFile)
- [Flutter platform channels](https://docs.flutter.dev/platform-integration/platform-channels)
