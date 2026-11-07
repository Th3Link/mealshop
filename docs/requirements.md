# Product requirements

Origin: customer vision, 2026-11-07. Mealshop is a working name.
The [feature index](../specs/README.md) maps F01–F20 one-to-one to acceptance contracts.

## Cross-cutting requirements
- N01: Rust/Axum/Tokio backend and Flutter client; standalone and free Nextcloud `ExApp` deployment.
- N02: Selectable Git/file, PostgreSQL, SQLite or MariaDB persistence, with portable versioned data.
- N03: Precise quantities, configurable presentation, reproducible nutrition and explicit unknowns.
- N04: Household isolation, dietary exclusions above recommendations, consented calendar/learning use.
- N05: AGPL-3.0-only complete software; paid official app, self-build allowed, free/paid hosted tiers.
- N06: Imported recipes share their source URL only, including via collections, MCP and federation.
- N07: Multi-day processes and reminders survive restart and explain delivery limitations.
- N08: Spec-driven development, ADRs and CI with Renovate, builds, docs/Pages, audit, SBOM, metrics and MSRV.

## Scope and unresolved decisions
M0 implements the bootstrap only. Nutrition data provider/licences, identity/authentication,
calendar provider, database migration/sync details, payment provider, attestation feasibility,
notification transport, federation protocol and concrete paid tiers remain open. No customer
price, health guarantee or provider commitment is inferred from the vision.

## Traceability
N01 → F01/F18; N02 → F18; N03 → F02/F03/F04; N04 → F09/F10/F11;
N05 → F19; N06 → F05/F16/F17/F20; N07 → F12/F13/F14; N08 → F01.
Each feature plan must link its tests to its numbered acceptance criteria before Done.
