# On-device operation and encrypted backups

The Android client keeps real local data without a Nextcloud account, URL or running service.
Choose **Use on this device** on the connection screen. The bundled catalogue and saved shopping,
recipes, images, collections, people, planning, cooking, inventory and brewing use the same Rust rules
as the server. There is nothing to set up: the app keeps one internal household that is never shown
or named, and no demo content is pre-populated. Data from earlier builds with several local
households stays reachable through a switcher that appears only in that case. Explicit website imports still
need network access and source consent; saved local workflows work without a server.

## Start assistant

With fresh local data a short assistant opens once: yourself first, then the people you cook for.
Each person is asked one thing per screen — name, diet, allergies and intolerances, dislikes and
strict exclusions, meal availability — and a name alone is enough. Everything can be skipped; an
unfinished profile is kept as a protected draft and the assistant returns to the people overview.
"This is me" is remembered on the device only. **More → Start assistant** opens it again, also
when connected to Nextcloud. Goals, dish ranking, sharing consent and in-app dictation are
specified in F41/F42 and not available yet.

## Save and restore a backup

Open **More → Backup and restore**. A backup includes all saved local records and images. Unsaved drafts, pending client intents, server credentials, device reminder settings
and temporary capture buffers are excluded. Saved activities require explicit read grants for this
device; a whole backup is refused when any stored activity is unreadable. Grant access visibly
through **More → Activities → Access** before retrying. No protected records are silently omitted
and ownership does not automatically grant read access. Device records use Android's private app storage; portable
backups are separately encrypted with the passphrase you choose.

1. Choose **Create encrypted backup**, enter a passphrase of 12–256 Unicode characters and repeat it.
2. Choose a destination in Android's document picker. Local storage or an installed document provider
   can be used. Keep the resulting `.age` file and passphrase somewhere you can access again.
3. Wait for the successful-save message. Merely opening the picker does not complete a backup. The
   last-completed date changes only after the selected document has been written and closed.
4. On this or another Android installation, choose on-device mode and **Restore from backup**.
5. Select the file, enter its passphrase and review the verified date, household count and data size.
6. Confirm **Replace local data** only when this is the intended backup. This replaces every local
   household and discards current unsaved drafts/pending intents. Nextcloud data remains separate.

Wrong passwords, damaged/unsupported files and cancelled previews keep the active canonical data.
The app cannot recover a forgotten passphrase. Backups are manual; choosing a cloud document provider
does not enable ongoing synchronization. App updates retain local data when package/signing identity
is unchanged. Uninstalling the app or clearing its Android storage removes its local datasets.

Restored cooking/brewing deadlines keep their original values. Device reminder opt-in and alarms are
not restored; enable delivery again after reviewing the restored sessions. Already overdue work does
not receive a newly fabricated duration.

## Switching modes and access

**More → Connection options** returns to the connection screen. Saved local data remains on
the device. The current credential/cache/draft session is cleared; choosing a Nextcloud connection
does not upload or merge local data. Cross-account invitations and recipe/collection grants require
connected mode and are unavailable locally. Existing source-attribution and import-sharing rules
still apply. Ordinary local collections and device exports remain available.

An installation has one fixed device principal. It does not authenticate different people using
the same unlocked app. Person-activity access still requires explicit grants and self-access consent;
ownership does not automatically reveal activities. The grant selector names this principal
**On this device**.

## Storage and recovery contract

The installed Android core is an executable in the APK's extracted native-library directory, called
through bounded private stdin/stdout. It opens no HTTP listener. Its stdout is protocol transport,
never a logcat stream. Requests carry the selected dataset generation; requests from replaced views
cannot mutate the replacement. A persisted generation mismatch clears obsolete client caches before
their pending intents can be replayed. Device reminder delivery checks the same generation.

Canonical data lives under `<noBackupFilesDir>/mealshop-app-only`, independently of the encrypted
seven-day client cache. A safe OS file lock excludes concurrent owners. Stale store markers are
reclaimed only under that lifetime lock and only when no pending data transaction exists. Interrupted
or corrupt data exposes recovery instead of being silently replaced with an empty household.

Restore extracts into a private candidate generation, authenticates the complete encrypted stream,
opens every household through the domain loaders and shows a preview. Its confirmation is invalidated
by intervening mutation-capable requests. A synced atomic `active.json` selects the validated dataset;
old runtimes stop before obsolete generations are deleted. A recovery dataset remains untouched until
a verified replacement is confirmed. A sole empty interrupted bootstrap can retain its existing
generation; this never adopts a nonempty orphan as an empty database.

If the app reports recovery, open **More → Backup and restore**. Keep the current files until a usable
backup has been verified. An unrecognized root format is refused. This slice does not automatically
guess how to repair incomplete domain transactions or migrate local households into Nextcloud.

## Portable format and limits

Version 1 is a standard age passphrase-encrypted, uncompressed USTAR stream. It starts with an encrypted
`manifest.json` containing format/engine version, UTC creation timestamp, household/file counts and
canonical byte count, followed by regular `data/` entries. The age scrypt work factor is 17; decryption
allows at most 18. Passwords are neither stored in metadata nor written to logs. Unencrypted metadata
is not exported alongside the archive. The format is independent of the Android keystore.

The archive and extracted stream are bounded to 512 MiB, individual files to 64 MiB, and canonical
files to 20,000. Path names are bounded, relative and traversal-free. Symlinks, hard links, sparse files,
special entries, duplicates, unsupported versions and mismatched counts are rejected. The encrypted
reader is drained past tar's end marker to authenticate the final age chunk. No arbitrary destination
path is accepted from the Flutter channel. Temporary plaintext is confined to private candidate data.

## Build and verification

Use the pinned [mobile setup](mobile.md). Bootstrap installs Rust 1.100.0 and its three Android standard
targets through rustup without changing the default toolchain. Setup fetches the locked Cargo source
graph before the offline native licence verification; a fresh build does not rely on a warm workstation cache. `python3 scripts/mobile.py build`
builds and bundles the core for `arm64-v8a`, `armeabi-v7a` and `x86_64`. Android uses atomic same-directory
rename under exclusive ownership because its app `SELinux` policy rejects hard links. Other targets
retain the existing hard-link publication path and sync ordering.

`python3 scripts/native_core.py --check` rejects missing, modified or stale native binaries. The
Gradle pre-build gate performs this check for the selected ABIs. Each APK's build metadata includes
the actual packaged core checksum, public source fingerprint, Rust version and target. Native ELF
segments are checked for 16 KiB alignment. Locked dependency/runtime notices are bundled offline;
`python3 scripts/native_notices.py --write` updates the reviewed inventory after dependency changes.

On this NVIDIA workstation use `python3 scripts/mobile.py integration --local --host-gpu`;
the default software renderer is available for CI. The local-data AVD has 2 GiB of guest RAM.
Run `python3 scripts/mobile.py integration --local` for the actual core, Android document export/import,
bad-password/damage preservation, hard process death, APK update and restart. It uses a separate
`Mealshop_AppOnly_API36` AVD on console 5558 and the disposable `org.mealshop.pocket.verification`
package. That package is explicitly reset; the development package and daily AVD are retained. Only
synthetic documents and screenshots are read. Source tests additionally cover portable restore into a
different root, exclusive ownership, interrupted transactions, links/traversal and stale-generation
writes. [F37 evidence](../../specs/0037-app-only-backups/plan.md) distinguishes completed checks from
remaining delivery checks. No physical-handset or production Nextcloud installation is claimed.
