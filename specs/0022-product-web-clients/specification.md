# Vue 3 product web clients

Status: Draft
Priority: Must
Origin: Customer (after PR #4 merge, 2026-11-07)
Milestone: M1

## User value
Use a proper web application inside Nextcloud or on a standalone Mealshop server.

## Acceptance criteria
1. The Nextcloud client uses Vue 3 and compatible Nextcloud components, respects host
   navigation/theme/accessibility, and loads assets/API calls correctly through AppAPI/proxy paths.
2. The standalone server client uses Vue 3 with its own navigation/authentication shell and
   connects to the same domain APIs. Shared code boundaries are chosen from real common needs.
3. Both clients use F21 language/content/number contracts from the outset; domain calculations
   stay on the Rust/API side and are not copied from the disposable demo.
4. Supported Nextcloud/AppAPI/Vue component versions, authentication, CSRF/CSP, base paths and
   deployment packaging are verified against actual installations before claiming compatibility.
5. The existing framework-free nutrition demo is explicitly a temporary test surface. Extending
   it does not establish product UX or replace either client. Design effort stays minimal there.
6. Product UX is specified/reviewed before implementation. If significant design work is needed,
   coordinate a dedicated UX/UI agent with the customer instead of expanding the demo design.

## Out of scope
Implementing either Vue client in the current localization/demo-classification change.
Flutter remains the separate app target in F18. No component library/build tool is selected yet.

## UX notes
To be defined before Ready with the customer or a customer-invoked UX/UI agent.

## Links
[ADR 0007](../../docs/adr/0007-localization-and-web-clients.md),
[F21](../0021-internationalization/specification.md), [F18](../0018-storage-deployment/specification.md).
