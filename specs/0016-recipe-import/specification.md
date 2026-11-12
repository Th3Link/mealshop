# Customer-operated recipe link import

Status: In progress
Priority: Should
Origin: Customer
Milestone: M3

## User value
Keep recipes from source websites in a private recipe library.

## Acceptance criteria
1. Given a customer-supplied URL (e.g. Chefkoch, Instagram, Einfach Backen), an available importer extracts a preview or clearly reports unsupported/blocked content.
2. Given an import preview, the customer reviews ingredient mapping, units, steps and provenance before saving; missing fields remain marked unknown.
3. Given any imported recipe, its original URL and imported-content status persist through revisions, exports and collections; sharing is restricted to the source link.
4. Given an untrusted URL, fetching rejects private/local addresses, unsafe redirects and excessive responses and does not execute supplied scripts.
5. Given source restrictions or required login, import does not bypass access controls; manual private entry remains available.

## Out of scope
Guaranteed support for every website or publication rights obtained merely by importing.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F16](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).

## Nextcloud delivery contract
F16 proceeds under the customer's standing autonomous implementation instruction after merged F17.
The implementation choices below narrow the public fetch surface while delivering all five criteria.

- Existing owner/editor household authorization protects fetch and save; readers cannot initiate either.
  Require explicit outbound consent for each URL fetch and retain original requested HTTPS URL across
  redirects. Manual entry issues no remote request. No login cookies, account headers, referrers,
  system proxies, script/browser execution or automatic asset/context retrieval.
- Public HTTPS on port 443 only. Resolve all addresses (maximum 32), reject mixed/public-private sets,
  special-use, local, mapped/transitional and reserved ranges. Pin accepted addresses in the transport
  and verify the peer. Validate each of at most three redirect targets before contacting it; reject
  robots redirects rather than relaxing policy. Respect robots disallow and explicit paywall metadata.
- Bound total fetch to 15 seconds, page to 2 MiB, robots to 512 KiB and extracted preview/source to
  96 KiB; accept UTF-8 HTML/JSON-LD (including text/plain carriers) only. At most four simultaneous fetches per process, one per actor
  and a ten-second actor cooldown in the existing single-writer deployment. No background crawling.
- Extract up to 16 Schema.org Recipe JSON-LD candidates with original text, author/language, yield,
  ingredients, steps/sections and optional source durations. Unsupported/login/policy/large/timeout
  states are actionable and always offer manual private entry. No universal website support claim.
- Review in the existing compact editor: explicit ingredient IDs, exact quantities and source text per
  line; inline ingredient creation and normal composition editing remain available. Unrecognized units,
  ranges, fractions and missing fields stay unknown until explicitly entered. Suggestions never use
  fuzzy food matching, volume-to-mass assumptions, source health claims or automatic translation.
- Required canonical values must be supplied; explicit confirmation reviews mapping, servings, steps,
  source differences and entered time estimates. Original unknown fields remain unknown in retained
  provenance. External images are not fetched/hotlinked; normal user image upload remains available.
- Actor-bound previews expire after 30 monotonic minutes, with at most 16 per household and four per
  account and cleanup within a further 30 seconds. Draft/permission/error/expiry states retain local
  input until an explicit discard/reload. Confirmed retry uses the same immutable request body/key.
- Persist selected bounded source fields, importer version, original/final URL, retrieval time and
  authenticated creation actor/time in the first immutable record, alongside its idempotency receipt.
  Same request survives retry/concurrency/restart without duplication; changed body/key reuse conflicts.
  Never store whole HTML. Existing origin rules prevent source removal/change; private reads expose
  original source fields, while sharing/collections/MCP expose only the retained source URL.

## Acceptance evidence
Map parser/transport/permission/review/atomic-retry regressions, native browser checks and installed
restart to AC 1–5 in validation.md. Complete all required gates and one reviewable PR against main.
Only ledger-owned synthetic households may be inspected/archived; preserve all retained data.
