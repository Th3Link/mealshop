# App-only operation and portable backups

Status: In progress
Priority: Must
Origin: Customer, 2026-11-15; demo request explicitly replaced by app-only operation with backup
Milestone: M1

## User value

Use the Android app with real, durable local data and no external server. Back up saved data and
restore it on the same or another device. This is a product operating mode, not seeded demo content.

## Acceptance criteria

1. First launch offers on-device operation alongside the existing Nextcloud connection. Choosing it
   requires no URL, account or service. The current bundled ingredient catalogue and existing saved
   household, shopping, recipe, people, planning, cooking, stock and brewing workflows remain usable.
2. Domain validation, exact quantities, nutrition provenance, dietary constraints and revision checks
   reuse the Rust core. The local core runs inside the app's installation through private standard I/O,
   without opening a network listener or emulating a remote Nextcloud service.
3. Saved local data survives activity recreation, normal restart, force-stop and APK update. Exactly
   one writer owns an app-only dataset. Interrupted transactions fail closed with recovery guidance;
   no live lock is removed and no corrupt dataset is silently replaced with an empty one.
4. The UI identifies on-device data and distinguishes it from remote synchronization. Cross-account
   invitations/sharing require connected mode and have an explicit unavailable state. Local data is
   never uploaded by choosing a server connection. Existing server authentication and data stay isolated.
5. A manual backup includes all saved local households and their canonical records/images, encrypted
   with a user passphrase in a documented portable format. It excludes server credentials, runtime
   secrets, device alarms and unsaved captures/drafts. Android's document picker chooses the destination;
   success is acknowledged only after writing and closing the selected document. Stored person
   activities require explicit read access; a complete backup refuses inaccessible records instead
   of omitting them or treating ownership as an automatic read grant.
6. Restore selects a backup through the document picker, verifies password, authenticity, version,
   size/path limits and domain integrity in a separate private dataset, and shows a preview before
   replacing active local data. Wrong passwords, damage, cancellation and unsupported versions preserve
   current data. Publication of a validated replacement is atomic and crash recoverable.
7. Backup and restore expose progress, cancellation where safe, actionable failures and keyed DE/EN
   messages. Passphrases and household data never enter logs or screenshots. Retained old datasets and
   temporary decrypted restore files are cleaned after successful publication/cancellation.
8. Native emulator tests prove offline creation/editing, process-death durability, backup/restore and
   failed-restore preservation using a dedicated verification package and synthetic data. Existing
   server-mode tests and repository gates remain green. Installable APKs accompany one reviewable PR.

## Scope and explicit limits

Android is the existing accepted mobile platform. The existing Nextcloud mode remains available.
Cloud providers can be chosen through Android's document UI; this slice does not add automatic `WebDAV`
synchronization, account federation, iOS delivery or implicit local-to-server migration. An app-only
installation has one device principal; it does not promise separate authentication for people using
the same unlocked app. Existing explicit person-activity grants remain enforced by the shared core.

## Links

[Plan](plan.md) · [UX notes](ux-notes.md) · [Decisions](adrs.md) ·
[F35](../0035-flutter-mobile-client/specification.md)
