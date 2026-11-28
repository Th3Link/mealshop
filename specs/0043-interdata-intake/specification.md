# Interdata: buffered recipe and ingredient intake service

Status: In progress
Priority: Must
Origin: Customer request, 2026-11-28
Milestone: M4 (proposed; the customer has not set one)

## Objective
A separate Rust HTTP service receives recipes and ingredients over a REST API, holds every upload
in a buffer, checks it half-automatically with AI assistance and only then makes it part of a
shared recipe and ingredient data set. It will be reachable at `interdata.mealshop.luehr.dev`
(the domain does not exist yet). It is not a household server: it stores no household, health or
planning data.

Customer wording (German, summarized): "we need a server for recipes and ingredients"; recipes
hold no pictures themselves but any number of links into a CDN (URL not known yet); uploads "are
not entered directly but buffered and checked first", "half-automatically with AI" for
plausibility, ingredient matching, unit conversion and nutrition completion; recipes carry tags
such as "especially good for fatty liver, Hashimoto, high blood pressure, diabetes, insulin
resistance" and "lactose-free, gluten-free, vegan, vegetarian, fish, beef, high-protein,
high-fibre, low carb, low fat".

## Delivered
Slice 1 (2026-11-28): acceptance criteria 1–4 and 13 as an undeployed service with a container
image definition and compose file; see the
[guide](../../docs/guides/interdata.md), [plan.md](plan.md) and ADR 0040. Criteria 5–12 are not
implemented: nothing is checked, reviewed or published yet.

## What exists today (reviewed 2026-11-28)
The suggestion inbox and unweighed quantities (F39, ADR 0039) are in PR #88 and not on `main`.

Server/`ExApp` (`src/recipes.rs`, `src/nutrition/`, `src/imports/`, `src/suggestions.rs`) and the
Nextcloud and Flutter clients use one recipe shape:

- Recipe: `title`, `language` (`en`/`de`), optional `author`, `servings`, optional `parts`,
  ingredient `lines`, `steps` (instruction, active/passive seconds), `origin`
  (`owned` / `captured` photo / `imported` with HTTPS `source_url`) and at most one private
  household `image_id`. Revisions are immutable and pin the catalogue entries they used.
- Line: `ingredient_id` (stable English slug), optional exact `grams`, optional `quantity`
  (decimal string + unit `g kg ml l tsp tbsp piece`), optional `conversion` evidence. Mass may
  be unknown (ADR 0039); nothing is converted from volume to mass implicitly.
- Ingredient: id, category, English name/aliases/form, `translations`, per-100-g `nutrients` with
  per-value uncertainty, a mandatory `source` (name, URL, record id, version, retrieval date,
  licence), `declared_allergens`, free descriptive `tags`, and an organization block with a
  `diet` of `unknown / vegan / vegetarian / meat_fish`.
- Unreviewed input already has a shape: `imports::Candidate` (source title, author, language,
  yield text, raw ingredient lines, raw steps, times) and the household suggestion inbox, which
  buffers candidates until a cook reviews them.

Gaps against this request: recipes have **no tags**; a recipe has **one private image**, not a
list of external links; there is **no shared, cross-household data set** and no service outside
a household boundary; AI checking is not implemented anywhere (F39 only records the intent).

## Behaviour
Criteria 1–4 and 13 are accepted and delivered with slice 1. Criteria 5–12 remain proposals
until their slice is Ready.

### Acceptance criteria
1. Given an upload presenting the configured upload secret (F43-ADMISSION) and, for a recipe,
   the CC0 dedication (F43-LICENCE), `POST` of a recipe or an ingredient is validated
   for size and shape, stored unchanged as a submission and answered with an id and status `received`.
   Nothing uploaded is visible in the published data set at that point.
2. Given a repeated upload with the same idempotency key and body, the same submission is
   returned; a different body under that key conflicts.
3. A recipe submission may be raw (`Candidate`-shaped source text) or structured (lines with
   ingredient references, quantities and units). The original upload is kept byte-for-byte next
   to every later normalized value.
4. A recipe carries zero or more image links. Each is an HTTPS URL on a configured CDN origin,
   with an order and optional alternative text. The service stores links only; it accepts no
   image bytes and does not fetch the links. While no CDN origin is configured, links are
   rejected with an actionable error.
5. A check run produces findings per submission, each with kind, severity, the evidence used and
   the stage and rule/model version that produced it: plausibility (amounts, servings, times), ingredient
   matching against the catalogue, unit normalization, and nutrition completeness. Findings are
   suggestions; they never change the submission.
   Checks escalate from cheap to expensive: (a) deterministic rules; (b) only what the rules
   could not settle goes to a self-hosted micro model; (c) only what that could not settle is
   queued for a frontier model, which runs solely after a moderator approves that escalation.
   Without approval the item stays open for manual review; no paid call happens on its own.
6. Ingredient matching follows the existing rule: automatic only for exactly one exact name or
   alias match; otherwise ranked candidates for the reviewer. A nearby food is never substituted
   and never lends its nutrition profile.
7. Unit handling is exact: same-dimension conversion only, decimals of at most six fractional
   digits, no implicit volume-to-mass. A mass for a volume or piece needs retained evidence or
   stays unknown.
8. Nutrition is calculated from matched catalogue entries only. Missing inputs yield an explicit
   incomplete result, never zero and never a model's guess presented as a value.
9. Given a reviewer, a submission can be accepted (optionally with corrections), rejected with a
   reason or returned for changes. Only acceptance creates an immutable published revision that
   records the submission, the reviewer, the applied findings and the catalogue version.
10. Tags come from a closed, versioned vocabulary with stable English ids and separate
    translations. Three classes are distinguished:
    - *composition* (vegan, vegetarian, fish, beef, lactose-free, gluten-free): derived from the
      matched ingredients; unknown ingredients make the tag unknown, not true. A "free from" tag
      is information, never an allergy clearance.
    - *nutrition* (high-protein, high-fibre, low fat, low carb): derived from the calculated
      nutrition against a threshold that names its primary source; unavailable when nutrition is
      incomplete.
    - *condition* (fatty liver, Hashimoto, high blood pressure, diabetes, insulin resistance):
      may be set by a contributor or suggested by a check without filed evidence (customer
      decision F43-CLAIMS). The plausibility check reviews them against ingredients and
      nutrition and flags contradictions. The tag records who or what set it and is published
      as an unverified suitability hint, not as a medical or evidence-backed statement.
11. An ingredient submission needs the source fields the catalogue already requires. Without a
    primary record it can only become an identity with unknown nutrition.
12. Published recipes and ingredients are readable through versioned `GET` endpoints with
    filtering by tag and language. Imported recipes expose their source URL only (N06).
    No contributor identity is stored with a submission or a published recipe; an optional
    `author` text in the recipe itself is content, not an account.
13. Uploads, URLs and text are hostile input: bounded bodies, rate limits that work without
    accounts (slice 1: one global per-minute ceiling and a buffer capacity), no
    server-side fetch of submitted URLs, no secrets or submission content in logs.

### Out of scope
Household data, meal plans, payments (F19), server-to-server federation (F20), hosting the CDN
or image processing, and any Mealshop client UI for browsing the data set. Wiring household
servers to consume the data set is a later slice.

## Customer decisions (2026-11-28)
- **F43-CLAIMS.** Condition tags need no evidence record; anyone uploading and the checks may
  set them, and the plausibility check reviews them. Consequence recorded here: they are shown
  as unverified hints and carry their origin, so the project rule against invented health claims
  is met by labelling rather than by evidence.
- **F43-RIGHTS.** Uploading is open to registered users. Registration is independent of this
  service and not designed yet; it should be as light as possible, without much login. Who
  uploaded which recipe is not remembered.
- **F43-AI.** Cheap before expensive: rules first, then a self-hosted micro model, then a
  frontier model only with a moderator's approval, to avoid unnecessary cost.
- **F43-ADMISSION.** Until registration exists, uploads are admitted by one operator-configured
  upload secret. It is compared in constant time, never logged and identifies nobody; it is
  replaced once registration is decided.

- **F43-LICENCE.** Recipes, their tags and everything the checks normalize from them are
  published under CC0 1.0. A recipe upload must state `"license": "CC0-1.0"`; that is the
  uploader's confirmation that they may dedicate it, and it is stored with the upload. Someone
  else's recipe is uploaded in the uploader's own words with its `source_url` (N06). Ingredient
  records keep the licence of their source record and are never relabelled. Image files on the
  CDN are not covered.
- **F43-DOCKER.** A container image definition and a compose file are part of slice 1 so that
  the customer can deploy the service.

## Open customer decisions
None block slice 2. Registration (F43-RIGHTS) and the model hosts (F43-AI) are still undecided.

## UX notes
The reviewer workflow (queue, finding display, corrections) needs `ux-notes.md` before Ready if
it gets a UI; the first slice can be API-only.

## Links
Requirement: [F43](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions:
[adrs.md](adrs.md). Related: F02/F23 (catalogue), F04 (recipes), F16 (import), F19, F20, F39.
