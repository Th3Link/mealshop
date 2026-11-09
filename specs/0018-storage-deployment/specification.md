# Storage adapters, Nextcloud `ExApp` and Flutter

Status: In progress
Priority: Must
Origin: Customer
Milestone: M1

## User value
Use an own server or the hosted service without locking household data to one backend.

## Acceptance criteria
1. Given a deployment, one selected backend persists equivalent versioned records using Git-backed files, SQLite, PostgreSQL or MariaDB.
2. Given a backend migration, an export/import round trip preserves identities, quantities, permissions, provenance and relationships and is verified before switching.
3. Given a Nextcloud `ExApp` installation, the Axum service integrates with supported `AppAPI` lifecycle/authentication; installation remains free.
4. Given a Flutter app, users can configure an own server or hosted endpoint and understand connection/authentication failures.
5. Given concurrent edits or a Git conflict, data is not silently overwritten; recovery and backup/restore behaviour is documented per adapter.

6. Nextcloud and standalone product web shells follow F22 (Vue 3); the current nutrition
   browser demo does not satisfy product integration. Localization follows F21; Flutter remains distinct.

## Current slice
F04 starts one local file store with immutable schema-1 revisions and pinned catalogue inputs.
It is not Git-backed and does not complete AC 1. Dedicated-directory exclusive locking,
expected-revision writes and fail-closed recovery address a bounded local part of AC 5.
See [ADR 0009](../../docs/adr/0009-local-recipe-revisions.md) and the
[operator guide](../../docs/guides/recipes.md). SQL/Git adapters, migration and Flutter remain unimplemented. The customer-authorized
Nextcloud slice now adds authenticated `AppAPI` lifecycle/transport, shared-household roles and
private file stores; real-host evidence and the pending container rollout permission are
tracked in [F22](../0022-product-web-clients/plan.md) and the [Nextcloud guide](../../docs/guides/nextcloud.md).

## Out of scope
All adapters in the bootstrap, an unvalidated common SQL dialect or a prematurely selected sync algorithm.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F18](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).
