# Native recipe collections and sharing

Use the existing Nextcloud shell, theme tokens, typography and compact record lists. Add a household
Collections destination and a Share action in recipe/collection details. A private collection starts
with a clear empty state and an Add recipe action; choose a recipe and explicit saved revision.
Show revision labels, ordered entries and straightforward remove actions. Retain drafts on conflict
and recover with reload; do not discard work after a failed save.

Keep sharing management separate from recipe editing. Show the exact revision and recipients before
applying a grant. The household owner controls access; reader/editor behavior must remain explicit.
The customer chose accounts and households. Household invitations address a contact, who accepts as
the owner of a destination household; show that all its current/future members gain read access.
Offer directly owned destination households without exposing other households' names. An incoming Shared with me destination
should work without household membership and must not expose source household navigation.

Imported entries show an Original recipe link rather than cached recipe content. Explain that the
source link is shared, with no external fetch, embedded image or preview. Empty/error/revoked states
must clear protected content. Use native dialogs, visible labels, keyboard focus restoration, busy
states and confirmation for removal/revocation. Check EN/DE, system light/dark and 320/390/1440 px
with scoped axe scans and screenshots of owned synthetic fixtures.

Taste is applied to alignment, spacing, hierarchy and visual consistency within the existing shell;
its marketing-page layouts/assets/gates do not replace Nextcloud components. UI/UX Pro Max's verified
confirmation/success-feedback guidance applies. Its optional form-library recommendation is unnecessary
for these bounded forms: keep the existing strict Zod contracts and native controls. Preserve dirty
drafts, exact quantities and permission states over generic styling recommendations.
