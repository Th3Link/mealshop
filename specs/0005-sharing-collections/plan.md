# Plan for Recipe sharing and collections

## Affected modules
Existing immutable recipes, `AppAPI` household authorization and the native Vue client. Add only
feature-owned models/storage/routes needed for collections and sharing; no crates or placeholder adapters.

## Tasks
- [x] Inspect accepted requirements, current household ownership and immutable recipe boundaries.
- [x] Resolve the recipient product decision; complete the delivery contract and authorization matrix.
- [x] Preserve imported source provenance across saving, history loading and revisions, with regressions.
- [x] Implement revision-checked private collections and pinned recipe references.
- [x] Implement explicit recipient access and projection-safe imported sharing.
- [x] Integrate native collection/sharing management and incoming reading with typed EN/DE contracts.
- [x] Exercise roles, isolation, stale writes, revocation, imported redaction, images and restart.
- [x] Verify installed Nextcloud behavior, responsive accessibility and cleanup of owned fixtures.
- [x] Complete all repository/client/container gates; prepare the reviewable PR.

Delivery handoff requires the PR checks to pass at its exact head; this final external state is
recorded in the PR rather than recursively committing its own head hash.
- [x] Design and implement the behaviour for AC 1; add acceptance evidence.
- [x] Design and implement the behaviour for AC 2; add acceptance evidence.
- [x] Design and implement the behaviour for AC 3; add acceptance evidence.
- [x] Design and implement the behaviour for AC 4; add acceptance evidence.

## Validation
All acceptance fixtures are synthetic. No private ingredient audit findings are included in public
fixtures, documentation or PRs. Retained household directories are not read/copied for this task's
cleanup; only ledger-recorded synthetic households are archived with the single writer stopped.

Backend evidence: `cargo nextest run --workspace --locked` passed 109/109 tests. New regressions cover
pinned collections and restart; role/membership changes; stale and invalid writes; ambiguous persistence;
personal publication/revocation; accepted and direct household grants; canonical recipient lookup failure;
imported projection in summaries/details/scaling/images and retroactive restriction of older revisions.
The directory-unavailable fixture returns a deterministic failure rather than relying on TCP teardown.

Client evidence: `npm --prefix clients/nextcloud test` passed 23/23 tests, including strict source-only
contracts, unsafe URL rejection, pinned references/duplicates and recipient variants. Strict TypeScript
production build and formatting pass. No dependencies, crates or `AppAPI` scope changes were needed.

The 17 repository/client gates, strict rustdoc, rootless TCP/Unix container smoke tests and installed
browser acceptance evidence are recorded in [validation.md](validation.md). Final PR-head CI is checked
on GitHub before handoff; that status is not inferred from local checks.
