# Plan for product web clients

Status: In progress. The Nextcloud client is implemented; the standalone shell remains a separate slice.

- [x] Record the native Nextcloud UX, role model and acceptance flows in ux-notes.md.
- [x] Validate installed Nextcloud 35.0.2 and `AppAPI` 35.0.0 contracts; record ADR 0012.
- [x] Implement authenticated proxy routing, persistent shared households, explicit invitations and owner/editor/reader permissions.
- [x] Implement native Vue shells, recipes and exact quantities, catalogue, profiles, planning and shopping-list preview/application against existing domain endpoints.
- [x] Preserve original quantities and pinned source revisions; support exact server-generated g/kg display.
- [x] Add strict `TypeScript`, runtime API validation, keyed English/German UI and host theme support.
- [x] Exercise actual Nextcloud browser workflows, conflicts, revocation, responsive bounds and accessibility with Playwright.
- [x] Package the client and backend in a rootless-compatible container; verify isolated TCP and Unix-socket transports.
- [x] Complete the final combined verification and record evidence in validation.md.
- [x] Deploy the prepared container after explicit customer approval of the internal trusted domain; verify all browser workflows and persisted API responses across restart.
- [ ] Implement and verify the standalone shell in its own usable slice (AC 2).

See docs/guides/nextcloud.md for reproducible commands and remaining production deployment limits. Do not treat the product vision, standalone shell, account-lifecycle reconciliation or a complete `HaRP` deployment as verified functionality.
