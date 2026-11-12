# ADR 0022: scoped MCP access and atomic recipe creation receipts

Status: Accepted for implementation under F17
Date: 2026-11-12

## Context
F17 needs authenticated recipe tools without bypassing household roles, exact quantities or imported
source-only sharing. Nextcloud already supplies authenticated account identity through `AppAPI`.
Recipe publication is one append-only, private, synced file transaction; a second idempotency journal
would create a crash window after recipe publication and before receipt persistence.

## Alternatives
Exposing the service secret would grant all households. New OAuth infrastructure is not justified
by the existing deployment. Client-declared read-only flags cannot protect backend tool calls.
Storing a separate successful-operation record after saving a recipe duplicates on ambiguous retries.
Reimplementing recipe validation in MCP would drift from the regular API.

## Decision
Reuse the Nextcloud authenticated proxy; owner-managed account grants default to Disabled and intersect
current membership/role on every call. Publish only recipe/ingredient tools. Use bounded stateless
JSON-RPC with 2026-07-28 and 2025-11-25 compatibility; reject Origin-bearing RPC requests and keep native
configuration on existing protected REST. Supply a standard-library stdio bridge using environment
credentials; prohibit redirects and nonlocal plaintext transport. App passwords retain normal account
powers, so a dedicated least-privilege account is required for clients outside the owner's trust.
This is existing Nextcloud authentication, not a new standalone auth provider or an OAuth claim.

Use canonical validation and a short-lived actor/scope/catalogue-bound preview. Explicitly confirmed
creation atomically adds an audit/idempotency receipt to the first recipe revision. The existing store
lock serializes duplicate keys; restart loads the immutable receipt. There is no second commit to lose.
A receipt is private persistence metadata; public API/tool projections expose actor/time, not keys.
Imported effective provenance controls every MCP projection, including historical revisions and previews.

## Consequences
No new crate, remote fetch, model provider, generic persistence adapter or global permission is needed.
The official MCP client is a pinned development-only dependency for reproducible wire interoperability tests.
Existing files without receipts remain compatible and are not rewritten. Failed/ambiguous publications
retain existing fail-closed recovery. Preview bodies live only in bounded memory and expire; clients
retry completed writes using the original key and preview ID, even after preview expiry or restart.
Remote hosted OAuth, per-client delegation and non-Nextcloud shells remain separate accepted features.

## Primary sources
- [Current MCP transport](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/streamable-http): stateless requests, mirrored headers and Origin validation.
- [MCP tools](https://modelcontextprotocol.io/specification/2026-07-28/server/tools): schemas, structured errors, annotations and confirmation.
- [MCP discovery](https://modelcontextprotocol.io/specification/2026-07-28/server/discover) and [version compatibility](https://modelcontextprotocol.io/specification/2026-07-28/basic/lifecycle).
- [MCP stdio](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio) and [authorization](https://modelcontextprotocol.io/specification/2026-07-28/basic/authorization): environment credentials for stdio; no false OAuth interoperability claim.
- [Nextcloud ExApp authentication](https://docs.nextcloud.com/server/stable/developer_manual/exapp_development/tech_details/Authentication.html); installed `AppAPI` proxy source and existing role checks are authoritative for this deployment.
