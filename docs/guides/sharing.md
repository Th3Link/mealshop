# Recipe sharing and collections

## Use collections
Open Collections in a household, create a named collection, select recipes and choose their exact
revision. Arrange entries with the move buttons and save. Opening an entry reads that saved revision;
Open current revision is an explicit way to return to the current recipe. Later recipe edits never
advance a collection or a meal-plan reference automatically. Owners and editors curate private
collections; readers browse. When a collection has any outgoing grant or pending invitation, only
its owner may change it. Deleting a collection revokes all its grants together.

## Grant read access
The household owner opens Share on a recipe or collection. Choose a person with the native Nextcloud
picker, invite a household through its owner, or select another household you own. Household invitations
remain content-free until the contact accepts as destination owner. Acceptance explains that all
current and future members of that household gain read access while they remain members, and its
name becomes visible to the sender. No directory of unrelated household names is published.

Shared recipe grants pin the displayed revision. After editing a recipe, Share this revision explicitly
updates a recipient's published version. Collection grants follow the owner's explicit collection edits,
while each recipe entry remains pinned. The dialog lists read grants with revision/pending state,
copy-link and confirmed revoke actions. A copied link still requires its addressed recipient's normal
Nextcloud login; it is not a public bearer link.

## Read incoming content
Shared with me is available outside the source household. Recipients can read and scale owned recipes
and their authorized images. They cannot edit, grant access onward, or browse the source household's
profiles, catalogue, plans or shopping list. Personal recipients may remove their access. Only a
destination owner may remove a household-wide grant. A pending contact can decline the invitation.

Recipe from another source records an original HTTPS URL. Once saved, that provenance is retained.
Shared imported items contain only this URL, including in collection entries and previews: no cached
title, author, ingredients, instructions, image or nutrition is republished. Correcting an existing
recipe to imported also restricts already-shared older revisions. This feature does not fetch or
parse external recipes; automatic import is F16.

Each server read rechecks current grant/membership, including image and scaling requests. Revocation
removes subsequent server access. The client clears protected content after an access failure and
refreshes on visibility changes or Refresh access. Content is not stored in browser storage, and all
feature responses use `Cache-Control: no-store`. Revocation cannot remove a copy already downloaded.

## API contract
All routes below are authenticated `AppAPI` user routes under the existing `ExApp` proxy. The browser
uses generated Nextcloud URLs; no hostnames or bearer credentials are embedded in client code.

| Route | Methods | Contract |
|---|---|---|
| `/api/households/{home}/collections` | GET, POST | Household list/search; create with `expected_revision`, name, optional description, entries |
| `/api/households/{home}/collections/{id}` | GET, PUT, DELETE | Pinned content; update/delete with `expected_revision` |
| `/api/households/{home}/shares` | GET, POST | Owner-only grants; create with `expected_revision`, resource, recipient |
| `/api/households/{home}/shares/{id}` | PUT, DELETE | Owner-only explicit recipe publication or revocation with `expected_revision` |
| `/api/shared` | GET | Current recipient's metadata and projection-safe summaries; pending content is null |
| `/api/shared/{home}/{share}` | GET, DELETE | Authorized detail or recipient removal with `expected_version` |
| `/api/shared/{home}/{share}/accept` | PUT | Accept addressed invitation with `expected_version`, `household_id` owned by actor |
| `/api/shared/{home}/{share}/recipes/{recipe}` | GET | Current collection entry only, at its pinned revision |
| `/api/shared/{home}/{share}/image` | GET | Granted owned recipe image only |
| `/api/shared/{home}/{share}/recipes/{recipe}/image` | GET | Granted collection's owned entry image only |

A recipe reference is `{recipe_id, revision}`. Resource variants are `recipe` plus its reference or
`collection` plus `collection_id`. Recipient variants are `account`/`household_invitation` plus `user`,
or `household` plus `home_id`. Stale revisions/versions return 409 without overwriting data.
Missing/unauthorized incoming content returns 404. Private writes respect household roles; outbound
sharing requires owner. A selected account is independently verified against `AppAPI`'s users utility;
unknown IDs and failed lookups do not create grants. Imported detail is exactly
`{kind: "source_link", source_url: "https://…"}` inside its content envelope.

Lists use one-based `page` and `per_page` (maximum 50, default 20; outbound grants default 50).
Collection search uses `q` up to 200 bytes. Outgoing grants can be filtered by `kind` and `resource_id`.
Shared recipe detail accepts `servings` and `display_unit=g|kg`; calculations reuse exact recipe scaling.
Optional `revision` must equal the granted/entry revision. No mass/volume inference is introduced.

## Storage and recovery
`library.json` lives beside `membership.json` under the existing exclusive storage-root lock. Schema
version 1 permits up to 100 collections and 1,000 grants per household, 200 unique recipe entries per
collection, 1,000 library households, and a 2 MiB serialized document. Names are at most 200 bytes and
descriptions 2,000 bytes. Monotonic collection/share identities are never reused. Recipe content remains
in its existing immutable store; the library contains references and access metadata only.

Atomic private `.library.pending` writes sync file and parent directory before replacing the document.
Ambiguous write failures stop further writes; a pending marker fails startup. Stop the single writer,
retain the failed files privately, restore a verified complete `library.json` (or resolve the pending
transaction against it), then restart and verify access. Never delete a pending marker without checking
the committed document. Do not edit metadata while the service is running. Existing recovery guidance
for [recipes](recipes.md) and [Nextcloud](nextcloud.md) still applies.
