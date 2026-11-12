# Native assistant connection and recipe receipts

Use a household Assistant tools view with the existing Nextcloud page header and ordinary labelled
controls. Show the current user's effective scope first, then the copyable exact endpoint and a
credential-free stdio example. Explain that recipe content is sent to the configured assistant,
imported recipes remain source-only, and creation needs a preview plus explicit host confirmation.
Never display/store app passwords or the `ExApp` service secret.

An owner sees current member IDs/roles and a Disabled/Read/Read and create selector per account.
Reader rows cannot select creation. Save/reload use a revision and retain conflicting drafts; changes
are protected when navigating away. Non-owners see their own scope, not editable owner controls.
Audit rows show actor/time and pinned recipe references, without idempotency keys or recipe contents.
Removed/unavailable references and loading/empty/error/revocation states stay explicit.

Use keyed EN/DE messages, native Vue controls, theme tokens and focus feedback. Taste governs
alignment/spacing and UI/UX Pro Max guides setup clarity; do not add marketing assets, palettes,
fonts or a new design system. Verify keyboard, 1440/390/320 px, light/dark, axe and real Nextcloud.
