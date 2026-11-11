# Recipe sharing and collections

Status: Done
Priority: Must
Origin: Customer
Milestone: M2

## User value
Share owned recipes and organize collections on a server.

## Acceptance criteria
1. Given an owned recipe or collection, its owner can save it on the server and select private or explicitly shared access.
2. Given a recipient without access, recipe content is withheld; revoking a share prevents subsequent server access.
3. Given an imported recipe, sharing emits only its original source URL and never republishes cached ingredients/instructions.
4. Given a recipe revision, collections and meal plans retain a defined revision reference rather than changing historical cooking data silently.

## Out of scope
Payments, federation and claims that downloaded copies can be revoked. Standalone/Flutter client
integration remains in F30/F18; automatic source fetching/import is F16.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Nextcloud delivery contract
The customer requested F05 after the merged F34 delivery. Existing recipes belong to households;
private means access stays within the current household roles, not a new individual ownership model.
The customer explicitly selected **named Nextcloud accounts and whole households**. Public/anonymous
links and write delegation are not part of this delivery. The contract passed Ready after the
recipient decision and is implemented as the Nextcloud delivery below.

- Collections have names, optional descriptions and ordered, explicit `{recipe_id, revision}` entries
  from the owning household. Source edits never advance those references automatically. Adding,
  replacing or removing an entry is a revision-checked collection edit.
- A recipe's explicit imported origin retains its original HTTPS source URL across revisions.
  Existing records without origin retain their original representation and remain readable.
  An imported origin cannot be silently converted to owned or replaced by another URL.
- Sharing must project imported recipes to their source URL before serializing any response,
  including collection entries, list/search results, previews, scaling and images. No server-side
  source fetching, cached imported title, instructions, ingredients, image or nutrition is emitted.
- Recheck current authorization for every detail, collection entry and image request. Revocation
  removes subsequent server access; neither browser caching nor copied downloads are claimed revocable.
- Preserve household isolation, frozen ingredient evidence, exact quantities, existing meal-plan
  references and `AppAPI` authentication/CSRF. Use additive file-backed models; no dependency is needed.

### Access and recipient matrix
- Household owners manage outbound grants. Owners/editors curate private collections; readers can
  browse them. Only owners may change a collection's entries while that collection has active or
  pending external grants. Readers/editors cannot publish additional content to its recipients.
- A named-account grant gives that account read-only access immediately. A household grant addresses
  an invitation to a selected account; that contact accepts as the owner of a target household.
  All current/future members of that target household may read while they remain members. The source
  owner can also grant directly to another household they own. Reject a grant back to its source home.
- Pending household invitations expose invitation metadata only, not recipe content. Acceptance
  explains the household-wide access. Do not publish a directory of other households to select from.
- A recipient can decline a pending invitation or remove their personal incoming grant. A target
  household's owner can remove its incoming grant; its editors/readers cannot revoke for everyone.
- Revocation and collection deletion remove grants atomically. Every incoming read uses current
  membership/recipient state. No permissions to edit, forward grants or read unrelated household
  data are conveyed. Anonymous users and unrelated accounts receive no shared content.
- Collections remain ordered sets of recipe/revision references. Shared collection edits are explicit
  owner actions; recipe edits never advance published references. Shared recipe details remain pinned.
  Existing meal plans retain their prior pinned revisions and are not silently changed by collection edits.

### Native workflows
Collections use a household navigation entry, native lists, search/pagination, a compact editor with
recipe/revision selection and explicit save/remove. Recipe and collection details expose Share to the
owner. A native dialog shows the frozen revision, current grants, recipient type and Nextcloud person
picker; revocation requires confirmation. Incoming content appears in Shared with me, independently
of household membership, with pinned recipe reading/scaling and access-checked images. Imported items
provide only their source link. Include loading, empty, conflict, error and revoked states; clear
protected content after an access failure and do not persist it in browser storage.

Implementation commands: `cargo test --locked --test recipes`; `cargo nextest run --workspace --locked`;
`cargo clippy --workspace --all-targets --locked -- -D warnings`; `npm --prefix clients/nextcloud test`;
`npm --prefix clients/nextcloud run build`; all repository gates in AGENTS.md.

Model/API code belongs in `src/`, native views/contracts in `clients/nextcloud/src/`, and acceptance
coverage in `tests/` and `clients/nextcloud/e2e/`. Follow existing typed errors, exact decimal strings,
keyed EN/DE messages and native Nextcloud controls. Never publish private source-audit findings,
relax authorization or replace frozen recipe history. Test missing/stale/unauthorized requests,
imported output redaction and restart as observable behavior, using synthetic fixtures only.

## Links
Requirement: [F05](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).


## Acceptance evidence
The Nextcloud delivery meets AC 1–4. See [validation.md](validation.md) for the acceptance trace,
109 backend tests, 23 client tests, installed browser scenarios and container/privacy boundaries.
Final candidate and exact-head CI results are retained in the reviewable pull request.
