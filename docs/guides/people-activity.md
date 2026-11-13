# People, dietary rules and private activities

A household person is distinct from a Nextcloud login account. Dietary profiles remain shared
household data: readers can inspect them, editors and owners can edit them. Activities use a separate
explicit per-person ACL. No diagnosis, body weight or medical goal is collected by this delivery.

## Rules and ingredient declarations

`GET/PUT /api/household/profiles` retains schema version 1 and its public revision. A member may carry
`rules: {version: 1, preset, exceptions, max_active_seconds, availability}` and
`avatar: {image_id: string | null}`. Presets are `unrestricted`, `vegetarian` and `vegan`. Exceptions
contain an exact `ingredient_id` and `household_produced_only` boolean. Maximum active time is an
optional integer of 1–604800 seconds. Availability windows contain Monday-zero weekdays (0–6) and
`breakfast`, `lunch`, `snack` or `dinner`; no windows means unrestricted attendance. Empty windows are
invalid. Existing clients omitting `rules` or `avatar` preserve the stored value. To clear an avatar,
send `{image_id: null}`; to clear restrictions, send an explicit unrestricted rules object.

Ingredient `organization.production` is an optional explicit `unknown`, `purchased` or
`household_produced` declaration. An omitted value preserves a saved declaration in legacy annotation
writes. Explicit `unknown` resets it. This declaration describes that ingredient identity throughout
the household; use distinct subtypes when keeping purchased and household-produced foods separate.
It does not create inventory or prove where a particular physical batch came from.

Exceptions relax only the preset. Allergies, sorbitol/intolerance restrictions, component/tag closure
and explicit exclusions still win. Missing diet/production data remains review-required. Organizational
parent groups do not override an explicitly selected subtype's dietary declaration; actual composition
is still evaluated. Current declarations and frozen recipe snapshots are reviewed conservatively.
Every canonical plan preview, apply and suggestion uses these rules. Attendance checks cover every
allocated eating slot, independently of a separate preparation date. Active-time limits use the
recipe's recorded active steps. No label is a guarantee of allergy safety.

## Explicit activity permissions

In an `ExApp`, prefix domain paths with `/api/households/{home}/domain`; the native client reaches
those through the configured Nextcloud installation's `index.php/apps/app_api/proxy/mealshop`.
Credentials are the current account's revocable app password, never the service secret.

- `GET /api/household/people/access` returns only each person's generation and capabilities.
- `GET /api/household/people/{person}/activity/access` exposes current account IDs and grants to
  an owner or an authorized reader, without entry contents/counts or their revision.
- `PUT .../activity/access` is owner-only. Supply `expected_profile_revision`, nullable
  `expected_generation`, `expected_access_revision` and `grants: {account: "read" | "edit"}`.
  Omit an account to revoke it. A newly added owner self-grant requires `confirm_self_access: true`.
- `GET .../activity?page=1&per_page=50` returns at most 50 dated entries, a total, capabilities and
  independent entry/access revisions. `GET .../activity/{entry}` reads one entry.
- `PUT .../activity/{entry}` creates/updates an entry with its explicit generation, access and entry
  revision guards. Entry IDs are `activity-` plus 48 lowercase hexadecimal digits, generated once by
  the client. Entries have an ISO date, kind (`walking`, `running`, `cycling`, `swimming`, `strength`,
  `other`), integer duration in seconds (1–604800), optional `easy`/`moderate`/`hard` intensity and
  optional bounded notes. No weight is needed. `energy` remains null.
- `GET .../activity/export` requires read access. Android offers a user-selected JSON document via
  the Storage Access Framework, without broad storage permission.
- `DELETE .../activity/{entry}` deletes one; `DELETE .../activity` deletes all. Both require edit
  access and the same three revision guards. An edit grant includes read, export and delete.

Owners have no implicit activity read/edit access. A household reader can receive explicit activity
edit access without gaining rights over recipes or dietary profiles. Leaving/rejoining a household
invalidates prior grants through membership epochs. Account removal never resurrects grants later.
Deleting/recreating a person invalidates old generations and grants. A household editor cannot delete
a person with protected activity state; an owner performs that cascade explicitly.

Activities and grants live in a private section of the atomically written, owner-only profiles file
(maximum 4 MiB; 1000 entries per person). Public profile reads, exports, planning and revisions do not
contain them. Entries remain until authorized deletion or person deletion. Exported copies/backups
are separate; deleting live records cannot erase those copies. An older server cannot parse newly
persisted rules/private data: use a matching complete backup or a reviewed forward recovery, never
strip the private section to downgrade. Keep this file and membership epochs together in backups.

## Android behaviour

More → People provides a five-step, resumable tour. A visible generic `Person N` label allows saving
without entering a personal name; a nickname can replace it. Dietary summaries explicitly identify
exceptions. Optional avatars reuse the existing bounded household raster upload store. Removing a
profile removes its reference; immutable uploads follow the existing image-store retention, not
activity deletion. No remote image fetch is introduced.

Only the edited person's draft, public revision and tour step are encrypted in the existing protected
record. Drafts are scoped by server/account/household/person, expire after seven days, and have a
25-draft/256-KiB aggregate limit. Other people's profiles are not copied into a tour draft. A resumed
stale draft requires comparison against current server fields before saving. Explicit discard clears
it; account removal or lost household access clears its scope.

Activities and incoming shared projections are online-only, held in memory. Sensitive reading refreshes
on foreground and periodically while visible; revoked access clears the view. Private activity forms
are hidden off foreground and disposed on denied access. A household/account switch disposes all open
routes. Export is an explicit copy to the destination the user chooses. No general offline write queue
is added for library/profile/activity edits.

Non-shopping writes use canonical revision guards. Recipe creation additionally supports optional
`POST /api/recipes?expected_count=N`, where N is the captured recipe-list length. The count check and
save hold the same store lock. Because recipes are not deleted, a lost-response retry cannot create a
second recipe, including after restart. Conflict requires checking the library, not silently rebasing.
Existing unguarded callers remain compatible. This is an online creation guard, not a replay receipt.
