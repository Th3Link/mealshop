# ADR 0010: protected local household profiles

Status: Proposed implementation for the customer-authorized F09 slice

## Context
F03/F04 are accepted. F09 covers household members, hard dietary constraints and soft dislikes.
Recipe sharing/collections remains F05. Household health data needs explicit access controls
before profile storage; hosted identity integration remains separate future work.

## Decision
Provide an opt-in local household operator Bearer credential, distinct from public catalogue
and recipe routes. All profile reads/exports/writes/removals and dietary reviews require it.
Keep one household per dedicated directory with schema/revision validation, exclusive locking,
private file permissions and atomic snapshot replacement. Token stays in runtime configuration
and page memory, never stored with profiles. Replacement snapshots allow correction/deletion
without an immutable health-data history. No new crate is needed.

Keep allergy/intolerance notes, hard ingredient exclusions and soft dislikes separate. Group
review unions hard constraints, conservatively resolves known ingredient aliases and reports
unknowns. Without complete allergen mapping, every unblocked result requires human review;
no safety claim or automatic interpretation of free text is made.

## Alternatives
Unauthenticated profiles would expose household data through the existing demo. Reusing the
immutable recipe history would retain deleted health data. Building a hosted identity system
here would extend beyond this local slice. One operator credential is a bounded first access
model; it does not implement multiple household roles or federated authorization.

## Consequences
The operator credential grants access to all members. Rotation requires restart; previously
exported copies/backups cannot be revoked. Profile storage and recovery must remain separate
from recipe storage. See the household operator guide. Hosted authentication and richer
allergen data remain explicit future work; F09 is complete for the documented local access contract.

## Completion amendment
Structured hard allergen groups and positive source-backed classifications now augment free-text
notes; read-only credentials may export/evaluate but cannot modify/remove profiles. Unknown
coverage still requires review. These changes complete the local F09 acceptance contract.
