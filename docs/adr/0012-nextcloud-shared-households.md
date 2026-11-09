# ADR 0012: native Nextcloud client and shared household boundary

Status: Accepted, 2026-11-09; verified native client and test-container integration (see F22 validation).

## Context
Before this change, the repository served a loopback-only single-household Rust API and a disposable
framework-free demo. Customer requests a complete native Nextcloud 35 client and chose shared
households with explicit invitations and read/write roles. The supplied test installation is
Nextcloud 35.0.2 with `AppAPI` 35.0.0; no `ExApp` or deployment daemon was registered initially.

## Alternatives
Publishing the existing server exposes unowned recipes and operator-token health APIs.
Per-user directories cannot represent shared households. A shared instance-wide directory has
no household boundary. A new general repository framework or authentication service is unnecessary.

## Decision
Add an opt-in `AppAPI` server mode with authenticated lifecycle, registered top menu/assets,
allowlisted proxy routes and server-side household membership. Keep the local mode intact.
Authenticate the signed `AppAPI` user/secret pair with constant-time comparison; never trust a
browser-supplied user ID or expose the `ExApp` secret/operator credentials to the browser.
Store membership separately from domain data. Each household has dedicated existing recipe,
profile and planning stores, preserving revision and nutrition provenance rules. Check current
roles before forwarding any request; readers cannot write even when a domain route lacks its
own local role check. Invitations are addressed, explicit and revocable.

Use Vue 3, `@nextcloud/vue` 9.14.1 and a single Vite IIFE plus CSS for `AppAPI`'s actual script loader.
Use `@nextcloud/router` to generate the embedded history base and PHP proxy URLs, and Nextcloud
axios for session/CSRF behavior. No origin hardcoding, CSP relaxation or CORS workaround.
Schema-validate responses at the `TypeScript` boundary. Keep decimal amounts as strings.

Additional Rust dependencies must serve this accepted boundary: base64 for `AppAPI` credentials,
subtle for constant-time comparison, reqwest for authenticated Nextcloud OCS registration, and
tower util for dispatching already authorized requests into existing domain routers. No new crate
within the workspace and no new storage adapter are introduced.

## Consequences
Nextcloud identity supplies authentication; household membership supplies authorization.
Domain data is not migrated from arbitrary local directories. Unknown data remains unknown.
The local test daemon is a development deployment, not evidence for `HaRP`/container packaging.
F22/F18 stay In progress until their complete acceptance criteria have actual evidence.

## Dependency and packaging evidence
Nextcloud Vue 9.14.1 depends on Vue Router 5.4; using Router 4 alongside it breaks router-aware
native buttons. The client uses one Router 5 instance. `AppAPI` 35 appends `.js`/`.css` to its
registered asset paths; registration therefore uses `js/mealshop` and `css/mealshop`.
The Vite IIFE defines Nextcloud Vue's `appName`, `appVersion` and production environment.
The FRP entrypoint is copied from the installed Nextcloud exapp-development reference with its
copyright and licence intact; `appinfo/info.xml` uses a local-only image registry, not a published
release. TCP and `HaRP` Unix-socket transports use the same authenticated routes.

Rust's TLS chain introduces ISC (`ring`, `rustls-webpki`, `untrusted`) and the bundled trust-store
CDLA-Permissive-2.0 (`webpki-roots`) licences. deny.toml grants only exact-version exceptions for
these packages, retaining the existing default policy. `RustSec` findings in quinn-proto and
webpki-roots were resolved with patched versions, without advisory ignores. A minimatch override
addresses the npm advisory in the Nextcloud component dependency graph; no component downgrade.

Primary integration reference: [Nextcloud 35 `ExApp` overview](https://docs.nextcloud.com/server/stable/developer_manual/exapp_development/development_overview/ExAppOverview.html).
The installed `AppAPI` 35.0.0 PHP controller and asset services were also read and tested directly.

## Scope follow-up (2026-11-10)
The customer approved closing the delivered Nextcloud portion of F21/F22. Their complete original
acceptance criteria are retained verbatim in Draft F29/F30. This supersedes the F22 status statement
above without claiming standalone/Flutter delivery; F18 remains In progress.
