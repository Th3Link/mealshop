# App-only and backup UX

Keep the existing native Material components, type scale, themes and five destinations. Apply
`ui-ux-pro-max` to task clarity, touch targets, text scaling, visible progress and recovery; do not
introduce a new visual system for the mode selector.

- Connection entry offers “Use on this device” and the existing Nextcloud connection. The local path
  opens the real household creation/selection flow. No sample household or pretend account is seeded.
- Home/More identifies “On this device”; local saves are not labelled as server synchronization.
  Server-only sharing explains that it requires a connection. An explicit exit returns to connection
  choice while retaining saved app-only datasets; ordinary Nextcloud sign-out semantics remain intact.
- More contains a local-data section with last completed backup, Back up and Restore. Explain that
  backups contain saved records and images and that the passphrase is required to restore them.
- Export asks for a passphrase and exact confirmation, then uses the system destination picker. Allow
  paste/password managers, provide an accessible visibility toggle, and clear fields after completion.
- Restore selects a file and requests its passphrase, validates privately, then previews saved-data
  counts/date/version. Replacement needs explicit confirmation. Errors and cancellation keep current
  local data and drafts. Restore completion clears obsolete local alarm enrollment and stale caches;
  saved cooking deadlines are not reset.
- Long operations show text and progress without blocking the UI thread. Do not label a selected
  destination as a completed backup before the write/close succeeds. A cloud provider may upload later.
- Check DE/EN, light/dark, 320dp at 200% text scale, keyboard/Back, screen-reader names and 48dp targets
  on new screens. Screenshots contain synthetic fixtures only, with no visible passphrases.
