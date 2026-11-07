# Server and delivery foundation

Status: Done
Priority: Must
Origin: Customer
Milestone: M0

## User value
Run and document a minimal server before implementing domain features.

## Acceptance criteria
1. Given the server starts with no database, GET /health returns HTTP 200 and exactly ok plus a newline; unknown routes return 404.
2. Given `MEALSHOP_LISTEN_ADDR` is set to a valid socket address, the server binds there; an invalid address or occupied port exits with an error.
3. Given a main commit, CI verifies formatting, clippy, tests, debug/release artifacts, coverage, declared MSRV, dependency licences/advisories, SPDX SBOM and binary-size metrics.
4. Given docs publication, requirements, specs, ADRs and Rust API documentation are browsable on Pages; Renovate uses the existing `RENOVATE_TOKEN`.
5. Given bootstrap delivery, source and AGPL text are on main and each workflow has been run and its outcome checked.

## Out of scope
Product endpoints, authentication, persistent data and Flutter.

## UX notes
No product UI in the bootstrap.

## Links
Requirement: [F01](../../docs/requirements.md). Verification: [bootstrap evidence](../../docs/bootstrap-verification.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).
