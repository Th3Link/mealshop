# Standalone Vue client and cross-client integration

Status: Draft
Origin: Customer-approved separation of remaining scope, 2026-11-10

The original acceptance contract is retained below without deleting requirements. Already verified
Nextcloud behaviour remains the baseline; the standalone/Flutter and outstanding content contracts
require their own implementation and evidence. No implementation of those clients is claimed here.

## Acceptance criteria
1. The Nextcloud client uses Vue 3 and compatible Nextcloud components, respects host
   navigation/theme/accessibility, and loads assets/API calls correctly through `AppAPI`/proxy paths.
2. The standalone server client uses Vue 3 with its own navigation/authentication shell and
   connects to the same domain APIs. Shared code boundaries are chosen from real common needs.
3. Both clients use F21 language/content/number contracts from the outset; domain calculations
   stay on the Rust/API side and are not copied from the disposable demo.
4. Supported Nextcloud/`AppAPI`/Vue component versions, authentication, CSRF/CSP, base paths and
   deployment packaging are verified against actual installations before claiming compatibility.
5. The existing framework-free nutrition demo is explicitly a temporary test surface. Extending
   it does not establish product UX or replace either client. Design effort stays minimal there.
6. Product UX is specified/reviewed before implementation. If significant design work is needed,
   coordinate a dedicated UX/UI agent with the customer instead of expanding the demo design.

