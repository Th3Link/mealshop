# ADR 0023: safe public recipe retrieval and explicit private review

Status: Accepted for implementation under F16
Date: 2026-11-12

## Context
F16 adds customer-operated public URL import to the existing Nextcloud recipe editor. URLs, DNS,
HTML and structured source content are hostile inputs. Import must not reach internal services,
execute scripts, reuse credentials, infer ingredient identities or remove imported sharing rules.

## Alternatives
A hosted scraper/model provider adds a new privacy boundary and is unnecessary. Headless browsing
would execute supplied scripts and encourage login bypass. Regular expressions are not an HTML
parser. A separate successful-import journal can drift from recipe publication after a crash.

## Decision
Use the existing reqwest transport with a deliberately narrow public HTTPS policy, DNS validation
and address pinning, peer verification, no system proxies/cookies/credentials, manual validated
redirects, bounded bodies and one total deadline. Bounded CPU slots retain their permit inside parser tasks even after cancellation. Respect robots policy and explicit paywall flags;
treat missing/unavailable robots resources under RFC 9309 conservatively (auth/rate-limit/policy
errors remain blocked); report unsupported/login/blocked sources without bypass. Source restrictions do not imply that a
public page grants publication rights. Manual private entry remains available without any fetch.

Use `html5ever` 0.35.0 directly for a non-executing HTML5 token stream and `robotstxt` 0.3.0 for
robots matching. A newer scraper failed Rust 1.85; its compatible version pulled in unnecessary CSS
selectors, an unmaintained hash dependency and licences outside the existing policy. A token stream
needs no DOM/selectors, uses bounded collected text, and passes the unchanged licence/advisory gates.
These parsers are required by this feature. No new service, model, auth provider or browser runtime.

Extract bounded Schema.org Recipe JSON-LD candidates, including arrays/graphs and nested steps.
Keep source text/language and optional yield/durations; missing values stay explicitly absent.
Never fetch JSON-LD contexts, images, scripts or linked instructions. Source nutrition, allergy and
diet labels never populate the ingredient catalogue or assert safety. Ingredient mapping is explicit;
only unambiguous metric amounts may be suggested, with source text retained.

Use the native existing editor for review. Required canonical quantities/servings must be supplied;
step times absent from the source remain labelled as such and any entered times (including zero)
are explicit user declarations under the review confirmation. Do not allocate total cooking time
across steps or silently translate. Preserve unknown original fields independently of reviewed values.

A bounded actor-bound preview lives in memory for 30 minutes. Final save checks current write role,
canonical recipe/images/ingredients and explicit review. Atomically attach an immutable import receipt
and selected original source snapshot to the first recipe revision using the existing publication
transaction. Repeated identical actor/key/preview/candidate/body returns that revision across restart;
changed payload under the same key conflicts. Ordinary edits retain original URL/status and expose
the first source snapshot only to the owning household; shared and MCP projections stay source-only.

## Consequences
No HTML or complete page is persisted; only selected bounded source fields and reviewed recipe data.
No secrets or recipe contents enter logs. Existing files remain readable and unchanged. An older
server cannot read a new import receipt; downgrade requires an explicit format-preserving migration.
Tests use synthetic sources/households, never retained private directories. New API transport seams
exist only to test DNS/redirect/body policy deterministically, not to provide a production bypass.

## Primary sources
- [Schema.org Recipe](https://schema.org/Recipe): ingredient text, yield and nested instructions.
- [RFC 9309](https://www.rfc-editor.org/rfc/rfc9309.html): robots policy and unavailable resources.
- [reqwest 0.12.28](https://docs.rs/reqwest/0.12.28/reqwest/struct.ClientBuilder.html): pinned resolution,
  redirect policy, HTTPS-only requests and disabling environment proxies.
- [IANA IPv4 special registry](https://www.iana.org/assignments/iana-ipv4-special-registry/) and
  [IPv6 special registry](https://www.iana.org/assignments/iana-ipv6-special-registry/): conservative
  globally reachable address policy, excluding transitional and special-use ranges.
- [html5ever](https://docs.rs/html5ever/0.35.0/html5ever/tokenizer/) and [robotstxt](https://docs.rs/robotstxt/): parser APIs.
