# Implementation and verification plan

- [x] Record the corrected customer request and the completed v0.1.0 release evidence.
- [x] Add a device host for the existing Rust household core and a bounded private stdio protocol (AC1–4).
- [x] Add exclusive ownership, durable dataset selection and encrypted backup/validated restore (AC3, AC5–7).
- [x] Package the native core for Android ABIs and add its lifecycle/document bridge (AC2, AC3, AC5, AC6).
- [x] Implement native mode entry, capability states, backup/restore UI and DE/EN messages (AC1, AC4–7).
- [x] Verify core/Flutter contracts, real emulator flows, update identity and all repository gates (AC8).
- [x] Prepare APKs and one reviewable PR with actual CI and remaining acceptance limits.

## Verification

Implemented and verified locally: 226 Rust tests with all features, 207 default server tests,
80 Flutter tests, all 55 connected-mode Android integration tests, strict Flutter analysis and Rust Clippy in both configurations, and Rust 1.89
compilation. The native API 36 run covers encrypted document export/import, rejected bad passwords
and truncated archives, exact recipe/image/shopping restoration, stale reminder rejection, hard core
process death, signed APK update and normal app restart with an empty Flutter record cache.
Screenshots were visually inspected; a stale-dump false-positive was eliminated by requiring fresh
UI snapshots across several frames. The workstation uses host GPU rendering and 2 GiB guest RAM.
All three Android targets compile and their packaged binaries pass checksum/source and 16 KiB
alignment checks. Upgradeable local-key APKs are version 0.6.0 (`org.mealshop.pocket.dev`),
with ABI-specific version codes 1007 / 2007 / 4007. Build metadata and SHA-256 files accompany
`app-armeabi-v7a-debug.apk`, `app-arm64-v8a-debug.apk` and `app-x86_64-debug.apk` under
`clients/flutter/build/app/outputs/flutter-apk/`.
The standard age format also restores a retained synthetic archive from the earlier 0.12 Android build.
Tests use synthetic app-only datasets and a dedicated Android verification package. Do not inspect, copy, hash, reset or back up retained private household or phone-account data.
The customer's production Nextcloud/GitOps configuration remains untouched.


## Security and engineering review

- Existing per-person activity permissions apply to whole-device backups: saved unreadable activities
  refuse the complete export, with no omission or automatic owner/self grant.
- Version 0.12.1 of age introduced a transitive `crypto-bigint` advisory through its unconditional
  HPKE/P-256 dependencies. The required standard v1 password format uses age 0.11.5 instead;
  no advisory exclusion was added. `cargo audit` reports zero vulnerabilities and two upstream
  unmaintained build-tool warnings (`basic-toml`, `proc-macro-error2`). The Android compiler is pinned.
- Binary image/revision storage was verified on Android after a native probe proved that app
  `SELinux` disallows hard links. The default server publication/sync ordering is retained.
- An OS ownership lock, generation-bound requests/reminders, nonce-bound cancellation and atomic
  active-pointer publication prevent concurrent/stale operations from silently replacing data.
- Document import/export accepts only Android document-provider content URIs; external file URIs
  cannot redirect the app into overwriting its own private files.
- Fresh Android CI exposed a missing Cargo-source preparation step. Bootstrap now fetches the locked
  source graph before offline native licence verification; this is verified with an empty temporary
  Cargo cache, which is removed after the check.
- Native licence inventories include exact upstream supplements and pinned runtime notices.
  Build guards reject missing/stale/modified Android core files and check the packaged binary digest.
- Raising the Rust minimum to 1.89 enables safe standard-library file locks. Mechanical Clippy updates
  follow that minimum and do not change quantities, provenance or dietary constraints.

## Delivery and acceptance limits

[PR #60](https://github.com/Th3Link/mealshop/pull/60) contains the complete frontend/backend slice.
CI is checked on its published head before marking the PR ready; the final run links and outcome
are recorded in the PR description. Generated APKs and synthetic screenshots are CI artifacts;
retained-key workstation APKs are the upgradeable handset builds. Physical-phone installation and
confirmation remain customer work; no handset was connected. Production deployment is not part of
this slice.
