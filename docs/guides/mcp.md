# MCP recipe tools

F17 supplies six household-scoped tools in the existing Nextcloud installation. It does not
connect a model or send data until you configure and invoke your own MCP client. Access defaults
to Disabled for every account. The local protected demo exposes the same contracts with its
existing bearer token; this guide describes the delivered Nextcloud connection.

## Connect a client

1. Open a household's **Assistant tools** view. The owner selects **Read and preview** or
   **Read, preview and create** for each current account and saves. A reader's role always caps
   access at read/preview, even if an older creation grant remains after a role change.
2. Copy the exact **Household MCP endpoint** from that view. It uses Nextcloud's configured
   installation path and proxy; do not substitute the container address or service secret.
3. Create an app password in that account's Nextcloud security settings. App passwords retain
   the account's ordinary Nextcloud permissions; they are **not** household-scoped OAuth tokens.
   For a client outside your trust, use a separate Nextcloud account with only the household role
   and MCP grant it needs. The UI never asks for or stores passwords.
4. Place this checkout's `scripts/mcp_stdio.py` on the MCP host. Python 3 uses only its standard
   library. Configure the host's stdio command as `python3` and its sole argument as the absolute
   path to that script. Supply these values through the host's protected environment:

   ```text
   MEALSHOP_MCP_URL=<exact copied endpoint>
   MEALSHOP_MCP_USER=<Nextcloud account ID>
   MEALSHOP_MCP_APP_PASSWORD=<app password from protected storage>
   ```

   Do not put the password in command arguments, a checked-in configuration, shell history or
   browser storage. The native view provides a credential-free environment example.
5. Allow read operations as needed. Configure your host to show the validated preview and request
   explicit confirmation before `recipe_create`. A recipe's text is untrusted content and cannot
   grant tools, change accounts or authorize a write.

The bridge requires HTTPS, except literal loopback/localhost HTTP for local testing. It refuses
redirects and URL credentials/query/fragment, uses the existing Nextcloud authentication and write
marker, and emits only newline-delimited JSON-RPC on stdout. Errors omit upstream HTML and credentials.
Requests are limited to 128 KiB, responses to 4 MiB and network waits to 30 seconds. A lost response
can be ambiguous: preserve the original confirmed write's preview ID and key when retrying.
No remote fetch, SSE subscription, sampling, tasks or OAuth service is advertised. A generic hosted
HTTP client that requires OAuth discovery is outside this authentication contract; use the supplied
stdio bridge. Protocol support is MCP 2026-07-28 and legacy 2025-11-25, verified with the official SDK.

## Tool flow

| Tool | Input | Result / effect |
| --- | --- | --- |
| `ingredient_search` | Optional localized `query`, `page` | Up to 20 declared catalogue/household ingredients and stable IDs. |
| `ingredient_get` | Ingredient `id` | Original declarations, sources, available conversions and unknowns. |
| `recipe_search` | Optional `query`, `page` | Up to 20 owned title summaries or imported source links. |
| `recipe_get` | Recipe `id`, optional pinned `revision` | Owned recipe and saved nutrition inputs, or original source URL only. |
| `recipe_preview_create` | Structured `recipe` matching discovered schema | Canonical validation, normalized exact values, preview ID and expiry; saves nothing. |
| `recipe_create` | `preview_id`, opaque `idempotency_key`, `confirmed: true` | First immutable recipe revision and replay status; creation scope required. |

The tool list contains machine-readable schemas. Use stable ingredient IDs and **decimal strings**,
for example a line with `ingredient_id: "oats"` and `quantity: { amount: "0.100500", unit: "kg" }`.
Canonical validation preserves the original quantity and normalizes it to `100.500000` grams.
A floating-point number, unknown ingredient, invalid image reference, invalid recipe composition or
implicit volume/mass conversion fails with a structured tool error, as in the regular recipe API.
Unknown nutrition/allergens remain unknown. Tools never claim dietary safety.

Imported recipes expose **only their retained original HTTPS source URL**, including preview,
search and older revisions subsequently marked imported. No imported title, ingredients, instructions,
image or nutrition is exported. Search matches imported source URLs rather than their private titles.
Profiles, plans, shopping, history, preference learning and other households are unavailable.

## Confirmation, expiry and retries

A preview is bound to the current account, scope-settings revision and ingredient revision. It expires
after ten monotonic minutes; a bounded household cache holds at most 16 previews and 4 per account.
Expired bodies are removed on access and by a household cleanup task within a further 30 seconds.
Previews never reach disk. A restart requires a new preview for an **uncommitted** write. Changing
scope settings or ingredient declarations invalidates a pending creation; obtain and review a new
preview. Saving identical grants preserves a valid draft. Household membership is checked on every call.

Before creating, the host must let the user inspect and deny the proposed recipe. `confirmed: true`
is the client's attestation, not cryptographic evidence of a physical human review. Generate and retain
one opaque ASCII key (16–128 characters: letters, digits, `_`, `-`, `.`) for the confirmed insertion.

The first immutable recipe file atomically includes the authenticated actor, timestamp, preview ID and
key. Two concurrent requests or a retry after timeout, expiry or server restart return the **same first
revision**, without another recipe or audit row. A different preview under that actor/key returns a
conflict. Another account cannot use your preview. Scope/role revocation applies even to retries.
Later edits keep the receipt in revision one; subsequent imported restrictions still apply to replays.
The native activity list displays actor/time and pinned revision links, never keys or credentials.

## Revoke and recover

Set an account to Disabled and save to stop subsequent tool calls. Reduce its household role or remove
membership when appropriate. Revoke an app password in Nextcloud to invalidate that credential across
its underlying account authority. A request already admitted before revocation may finish; revocation
is enforced on subsequent calls. There is no background assistant or speculative execution.

Settings are private `household/mcp.json` (schema 1, mode 0600), with optimistic revision checks and
synced atomic rename. A settings conflict retains the browser draft until explicit reload. An interrupted
`.mcp.pending`, invalid settings or an ambiguous write fails closed. Stop **all** writers, inspect only
the affected household's verified settings and recover deliberately; never silently reset grants.
Recipe publication/recovery uses the [existing immutable recipe procedure](recipes.md). Do not delete a
successful first revision or its receipt to retry. Backups and restoration must preserve the complete
first revision. Older servers do not understand creation receipts: do not downgrade a household with
MCP-created recipes without an explicit format migration preserving audit/idempotency information.

## Verification

The [F17 evidence](../../specs/0017-mcp/validation.md) records real Nextcloud, browser, installed restart
and container checks. Reproduce isolated wire compatibility with:

```sh
cargo build --locked
npm --prefix clients/nextcloud ci --ignore-scripts --no-audit --no-fund
node scripts/generate_mcp_schema.mjs --check
python3 -m unittest discover -s tests -p 'test_mcp_stdio.py'
node scripts/test_mcp_client.mjs target/debug/mealshop-server
```

The official MCP SDK is pinned as a development dependency only; the Rust runtime and Python bridge
add no dependency. CI exercises modern direct HTTP and modern/legacy stdio with isolated synthetic data.
Real-instance Playwright checks require the existing test credentials in protected environment variables
and create only ledger-recorded synthetic households. Never use retained customer households as fixtures.
