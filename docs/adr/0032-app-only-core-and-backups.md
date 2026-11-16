# ADR 0032: App-only core execution and portable backups

Status: Accepted implementation direction for the corrected customer request, 2026-11-16.

## Context

The customer replaced a requested demo with real app-only operation and backup. The mobile client
currently calls Nextcloud while its authoritative domain validation lives in Rust. A seven-day
offline cache cannot become authoritative merely by retaining it longer.

## Alternatives

- Reimplement domain rules in Dart: duplicates nutrition, quantities, planning and mutation semantics.
- Introduce Rust FFI exports: conflicts with the repository's prohibition on unsafe application code.
- Embed the existing Rust executable and call its routers through private standard I/O: reuses the
  same domain behaviour without an external server, network listener or new FFI boundary.

## Decision

Package a native Android Rust executable with the APK. A bounded, serialized stdio bridge invokes
the same household routers under an explicit device host, distinct from the Nextcloud host. There
is one fixed on-device principal, no simulated Nextcloud URL/login, and no automatic Nextcloud
notification worker or cross-account sharing. Explicit source imports keep existing consent rules.
Android background alarms remain the existing device facility.

Keep local datasets in the private no-backup app directory, separate from remote credentials/caches.
Use standard-library exclusive file locking for lifetime ownership and safe stale-marker detection;
raise the declared Rust minimum to 1.89, where safe file-lock APIs are available. Existing deployed
server directories retain their current fail-closed recovery semantics. An actual Android probe
confirmed that app `SELinux` policy rejects hard links. Immutable image/revision publication therefore
uses an atomic same-directory rename on Android, under device lifetime ownership and the existing
store mutex, with an explicit existing-target check. Other targets retain hard-link publication and
its existing sync/marker ordering. No unsafe syscall wrapper or extra filesystem crate is introduced.

The optional device feature uses the `age` 0.11.5 and `tar` crates for standard passphrase-encrypted,
streaming archives rather than a new cryptographic format. These adapters are needed by the accepted
backup feature and are absent from ordinary server builds. The 0.12.1 line adds an unconditional
HPKE/P-256 chain affected by [RUSTSEC-2026-0387](https://rustsec.org/advisories/RUSTSEC-2026-0387).
Use the current 0.11 patch line for the required standard v1 passphrase format, removing that chain
rather than suppressing the advisory or implementing replacement cryptography. A synthetic archive
from the actual 0.12 Android build verifies compatibility across this dependency change. Backup includes saved local data only.
Restore rejects non-regular/traversing/duplicate or excessive archive entries and validates the
complete candidate through the domain core before changing a synced, atomic active-dataset pointer.
Existing live data stays intact during decryption/validation. Whole-backup authorization consults
the existing person capability API before reading saved private activities; unreadable records block
the complete export rather than being omitted or bypassing explicit self-grant consent. Cleanup excludes the selected dataset.
Every local API request is bound to its dataset generation. Restore invalidates old views and cached
intents, including when the app dies after publication but before refreshing. Device alarm projections
carry that generation and refuse delivery against a different active pointer. The Android build gate
checks actual binary/source checksums instead of silently packaging an outdated executable.

## Consequences

Native binaries must be built/tested for the shipped Android ABIs; extracted writable assets are
not executed. The installed APK's native-library location supplies executable code, consistent with
Android's execution restrictions. The private protocol cannot authenticate another physical user of
the same unlocked app, and the UI must not imply such isolation. Backups require the user's passphrase;
the app cannot recover a forgotten passphrase. Automatic cloud backup/sync remains separate scope.

## Primary references

- [Android execution restrictions](https://developer.android.com/about/versions/10/behavior-changes-10#execute-permission)
- [Rust Android targets](https://doc.rust-lang.org/rustc/platform-support/android.html)
- [Safe File locking](https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock)
- [age streaming encryption](https://docs.rs/age/0.11.5/age/)
- [tar security model](https://docs.rs/tar/latest/tar/)
