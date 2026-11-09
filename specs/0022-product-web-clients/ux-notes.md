# Nextcloud product experience

Status: Ready for implementation, customer authorized 2026-11-09.

## Design contract
Nextcloud owns the header, navigation chrome, typography, theme and focus conventions.
Use `@nextcloud/vue` 9 (Vue 3), Composition API and strict `TypeScript`. No global reset,
web fonts, decorative cards, hero layouts, gradients or alternative component system.
Taste contributes alignment, rhythm and screenshot review only: its website generators,
hero/motion requirements and marketing gates conflict with the customer's native-app brief.
UI UX Pro Max contributes search, progressive disclosure, keyboard/focus and error recovery.
Awesome Design was not found in the installed skill directories; no capability is assumed.

## Information architecture
- Household context first: create a household or accept an addressed invitation.
- Recipes: list, create, read, revise and exact serving preview using existing domain APIs.
- Meal plan: attendance, recipe revisions, time constraints, suggestions, review, shopping preview,
  apply and removal. Calendar snapshots require explicit consent; no live calendar is implied.
- Ingredients: searchable catalogue, translated identity, exact nutrient values, provenance,
  original-language fallback and explicit unknowns.
- Household: dietary profiles and membership are distinct sections. Hard exclusions and soft
  dislikes never share a control. Health-data visibility is explained before accepting access.

## Membership contract
Customer chose shared households with invitations and read/write roles. Proposed concrete
roles: owner manages membership, editor modifies domain data, reader reads/exports and reviews.
Invitations target an exact Nextcloud user ID and require explicit acceptance. No automatic
instance-wide household. Server checks current membership on every request. Unknown account
names do not expose the Nextcloud directory. Owner cannot be removed without transferring ownership.

## Interaction contract
One page title and clear primary action. Native navigation, dialogs, fields, notices and empty
states. Persistent labels, inline errors, retained input on failure, conflict recovery without
silent overwrite. Loading and retry paths on every remote screen; no fake success or demo data.
Lists lead to URL-addressable detail pages. Back returns to the list and keeps its search.
At narrow widths navigation collapses using native components, forms stack, details wrap and
nutrient tables retain readable headers. Native theme variables only; inherited reduced motion.
Keyboard focus moves to the new page heading; dialog focus trapping belongs to `NcDialog`.

## Verification
Real host: customer-provided Nextcloud 35.0.2 / `AppAPI` 35.0.0 at loopback port 8080.
Verify session/CSRF, proxy paths, refresh/deep links, two users, reader denial, invitation/revocation,
revision conflicts, unknown nutrition, light/dark, 320/768/1024/1440 widths and axe checks.
A second visual pass may refine spacing and copy, but cannot stand in for missing workflows.
