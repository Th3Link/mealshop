# Reviewed private recipe import

F16 adds **Import from link** beside Create recipe in a Nextcloud household. Only owners and editors
can fetch or save an import. Readers retain ordinary recipe access but cannot initiate these operations.
A model/provider, subscription, browser extension or source-site account is not required.

## Fetch, review, save

1. Enter the original HTTPS recipe URL. Explicitly allow this server to contact the source and
   validated redirects: the website receives the URL and the server's IP. Nextcloud credentials,
   cookies and referrers are never forwarded. No source scripts, images or JSON-LD contexts load.
2. Fetch the preview and choose a recipe if the page supplies several. Supported public Schema.org
   Recipe JSON-LD can appear in HTML, a JSON document, arrays or graphs. Nested instruction sections
   keep their headings. This is structured extraction, not guaranteed support for every website.
3. Continue into the normal compact editor. Original ingredient text sits beside its mapping. Select
   actual catalogue/household ingredient IDs; missing ingredients can be created inline. Suggestions
   accept only unambiguous metric prefixes with an appropriate decimal convention. Fractions, ranges,
   unrecognized units and ambiguous separators require explicit input; no nearby food is substituted.
4. Enter missing canonical values, including positive servings and exact quantities. Volume/count
   conversion still needs explicit matching evidence. Total source preparation/cooking durations are
   retained as supplied and never distributed over steps. All entered step times, including zero,
   are your explicit declarations under the review confirmation; absent original values stay unknown.
   Check the reviewed recipe's language; no text is silently translated.
5. Review ingredient mappings, amounts, servings, language, steps and time estimates, then confirm
   and save. Subsequent changes clear confirmation. The original URL is fixed. Original source fields
   remain separate from your reviewed values, including after later edits and historical reads.

Imported nutrition, allergy and diet claims never populate the ingredient catalogue or establish
safety. Nutrition uses the reviewed IDs/quantities and existing source snapshots/ranges. External images
are not fetched or hotlinked; the normal explicit image-upload control remains available.

## Unsupported or blocked source

Login/paywall/robots restrictions, unsafe targets, encoding/type/size limits, timeouts and missing
structured recipes have explicit errors. Use **Enter privately without fetching** to open the same
editor with the original URL and unknown original fields. This path makes **no remote request**.
It does not bypass source access controls or imply publication rights. Fill your private copy normally;
sharing, collections and MCP still expose only its source link. Chefkoch, Instagram and Einfach Backen
are examples of customer URLs, not a promise that every page permits automated retrieval.

## Drafts, ambiguous responses and retries

Previews are actor/household-bound, memory-only and valid for 30 monotonic minutes. At most 16 previews
per household and four per account are retained; a cleanup task removes expired data within another
30 seconds. No full HTML document reaches persistence or browser storage. Required fields cannot save
as unknown; the source snapshot still records which original fields were absent.

Once submitted, an ambiguous request is frozen. **Retry confirmed import** sends the same body,
preview/candidate and operation key, even if the first response was lost. Publication atomically places
its authenticated actor/time, immutable source fields and idempotency receipt in revision one. Parallel
or restarted identical retries return that same first revision; changed payload/key reuse conflicts.
A definite validation failure unlocks the retained draft for corrections and a new confirmation.

Leaving, replacing or reloading unsaved work requires explicit discard. An expired or restarted
uncommitted preview must be fetched/reviewed again. If you leave an ambiguous submission, check the
recipe list before beginning a new import; no key/source content is persisted in browser storage.
Revoked membership or a reader role blocks subsequent saves, including retries. Requests already
admitted can finish; the normal Nextcloud account/household boundaries remain authoritative.

## Public fetch boundary

- Public-domain HTTPS on port 443 only, without embedded credentials. IP literals, local suffixes,
  private/local/reserved, mixed DNS sets, IPv4-mapped/transitional IPv6 and special-use ranges are
  rejected. A conservative IANA-based policy allows ordinary IPv4 and IPv6 global-unicast addresses.
- Resolve at most 32 addresses, reject the whole set if any is unsafe, pin accepted addresses and
  verify the connected peer. Recheck every target before each hop. Environment proxies are disabled.
- At most three recipe redirects; unsafe targets are rejected. Robots redirects/policy/auth/rate-limit
  failures block the fetch. Missing/unavailable robots resources use the documented RFC-9309 subset;
  applicable allow/disallow rules and explicit `isAccessibleForFree: false` metadata are respected.
- A 15-second total network deadline, 3-second connect and 8-second individual request bounds. Page
  bodies are limited to 2 MiB, robots to 512 KiB and extracted preview/source fields to 96 KiB.
  UTF-8 HTML/JSON-LD is accepted, including plain-text carriers; compressed/unsupported encodings fail.
- Four concurrent fetches per process and four bounded parser workers. One active fetch and a
  ten-second cooldown per household/account pair; there is no background crawling or automatic retry.
  Parser permits remain held by worker tasks even if an HTTP caller disconnects or times out.

HTML5 tokenization and the established robots matcher add no script/browser execution. The unchanged
licence/advisory policy and actual Rust-1.85 compatibility check are verified; no advisory exception was introduced.

## API and recovery

All paths below are relative to the existing authenticated household domain; normal JSON, request
marker, enabled-state and current membership checks apply. No new hostnames or auth flow are required.

| Request | Body / response |
| --- | --- |
| `POST /api/household/imports/preview` | `{ mode: "url", url, consent: true }` or `{ mode: "manual", url }`; returns bounded candidates, requested/final URL, method, retrieval time, preview ID and expiry. |
| `POST /api/household/imports` | `{ preview_id, candidate, idempotency_key, reviewed: true, recipe }`; zero-based candidate, opaque ASCII key of 16–128 characters, canonical reviewed recipe. Returns the normal saved recipe projection. |
| `GET /api/recipes/{id}` | Owning-household response includes `import_source` separately from `recipe`; receipt keys/preview IDs are omitted. Historical reads retain that first original snapshot. |

The private first recipe record gains optional `import_receipt`; old records are unchanged/readable.
It is valid only in revision one, must match the imported original URL and uses the existing private,
synced append-only publication. Later record files do not duplicate it. Damaged, duplicate or misplaced
receipts fail closed on startup. Preserve the complete first revision in backup/restore. Do not remove
its receipt to retry or downgrade to a server that rejects this format; use an explicit migration that
preserves provenance/idempotency. Follow [recipe storage recovery](recipes.md) for pending writes.

[Acceptance evidence](../../specs/0016-recipe-import/validation.md) records real Nextcloud/proxy/browser,
source, container, restart and synthetic-only cleanup checks. Deterministic transport tests inject DNS
and HTTP replies into a private boundary; production has no test-network bypass. The public synthetic
fixture is versioned in `tests/fixtures/import-recipe.html`. Browser tests use an immutable fixture URL
before merge (`MEALSHOP_IMPORT_FIXTURE_URL`) and main afterward; they never inspect retained households.
