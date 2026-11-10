# Native Nextcloud Vue client

Status: Done
Priority: Must
Origin: Customer-approved Nextcloud scope separation, 2026-11-10
Milestone: M1

## User value
Use the implemented Mealshop workflows inside the real Nextcloud 35 host.

## Acceptance criteria
1. Vue 3 with compatible Nextcloud components respects host navigation, themes and accessibility; assets and typed API calls use embedded and `AppAPI` proxy paths.
2. Shared household ownership, invitations and editor/reader roles protect the actual implemented recipes, ingredients, profiles, planning and shopping workflows.
3. The Nextcloud client uses F21 locale/content/quantity contracts; Rust remains the calculation boundary.
4. Supported versions, authentication, CSRF/CSP, base paths and rootless container TCP/Unix transports have actual integration evidence.
5. Product UX is specified and reviewed before implementation. The framework-free `src/web/` remains a disposable test demo.

## Scope decision
The customer explicitly approved closing the shipped Nextcloud portion.
[F30](../0030-standalone-vue-client/specification.md) preserves the original six acceptance criteria verbatim,
including the unimplemented standalone shell and cross-client contracts. Flutter deployment remains open in F18/F29.
A complete `HaRP` deployment and arbitrary reverse-proxy topologies are not claimed as tested.

## Evidence and UX
[plan.md](plan.md), [validation.md](validation.md), [ux-notes.md](ux-notes.md),
[ADR 0012](../../docs/adr/0012-nextcloud-shared-households.md), [setup guide](../../docs/guides/nextcloud.md).
