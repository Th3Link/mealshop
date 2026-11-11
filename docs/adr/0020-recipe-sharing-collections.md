# ADR 0020: revision-pinned recipe collections and explicit sharing

Status: Accepted for implementation; customer selected accounts and households
Date: 2026-11-11

## Context
F05 requires private or explicitly shared recipes/collections, revocable server access and source-link
sharing for imported content. Existing ownership is household-based and recipe revisions are immutable.
Private audit findings must not be used as implementation fixtures or published acceptance evidence.

## Alternatives
Joining a household to read a recipe grants excessive access. Anonymous bearer links are not assumed.
Following a recipe's latest revision silently changes collections and may expose later edits.
Republishing cached imported content violates the established sharing restriction. A database adapter
or new identity system is unnecessary for this bounded file-store delivery.

## Decision
Keep current household ownership and roles. Store collection metadata separately from immutable recipe
records, with explicit recipe/revision entries, monotonic identities and optimistic revision checks.
Use private atomic pending-file writes under the existing single-process root lock; fail closed on
ambiguous storage errors or pending recovery. The collection API validates references to its own
household's saved recipes. Revision-pinned meal plans keep their existing behavior.

Add defaulted, omitted-for-owned recipe provenance. Once imported, a recipe retains the original
HTTPS source URL across revisions and restart. Future imports must set that provenance at creation;
F05 does not implement fetching/parsing external recipes (F16). A share projection branches on origin
before constructing the response: imported content becomes a source-link-only result. User-entered
authorship is a declaration, not independently verified rights ownership.

Implement personal account grants, addressed household invitations accepted by a destination owner,
and direct grants to other households owned by the source owner. Current authentication, enabled
state and CSRF remain mandatory. Recheck the current resource-specific grant for every shared output,
including collection entries, scale requests and images. Never grant access to the source household's
profiles, ingredients catalogue, plans, shopping or unrelated recipes. Do not retain recipient content
in browser storage. No public source fetches, link previews or third-party sharing provider are added.

## Consequences
Existing recipe files remain readable and byte-identical. Collection edits do not change recipes or
historical plans. Revocation governs subsequent server access, not copies already downloaded. Current
file limits and actionable conflict/recovery responses remain explicit. UI flows and the final role/
recipient matrix are recorded in the feature specification. Collection editors cannot add externally
visible content without owner authority once a collection has any outbound grant. Pending invitations
expose metadata only. Source/target owners may revoke; target-household members are authorized against
their current membership on every request.


## Implementation boundaries
`library.json` contains bounded household collection/grant metadata. Its optimistic household
revision protects outgoing writes; each grant has a separate version for recipient acceptance/removal.
Collection deletion removes its grants in the same atomic write. A pending or ambiguous write makes
the store read-only until the operator restores a verified document and restarts the single writer.

A recipe read resolves transient `sharing_origin` from its latest revision while retaining the requested
immutable recipe and ingredient snapshot. Correcting a formerly owned recipe to imported therefore
restricts all shared historical revisions without rewriting their files. Imported provenance cannot
be removed or its source replaced in a later revision. Sharing checks this effective origin before
constructing a summary, detail, scale or image response. Legacy owned files remain readable unchanged.

The native picker follows Nextcloud file-sharing visibility. The server independently verifies the
selected canonical account ID with the authenticated `AppAPI` users utility; lookup failure prevents
creating a grant. See the official [`AppAPI` users utility](https://docs.nextcloud.com/server/stable/developer_manual/exapp_development/tech_details/api/utils.html).
No remote directory response is retained or logged. The existing `AppAPI` route scopes are sufficient.
