# Decisions for Interdata intake

- [ADR 0002](../../docs/adr/0002-quantities-and-nutrition.md)
- [ADR 0004](../../docs/adr/0004-licensing-hosting-and-sharing.md)
- [ADR 0008](../../docs/adr/0008-curated-ingredient-sources.md)
- [ADR 0040](../../docs/adr/0040-interdata-intake-service.md) — second workspace crate, file buffer, upload secret
- [ADR 0023](../../docs/adr/0023-reviewed-recipe-link-import.md)
- ADR 0039 (unweighed recipe quantities; arrives with F39, PR #88, not on main yet)

## Customer requirements (2026-11-28)
- Separate Rust HTTP service with a REST API for uploading recipes and ingredients.
- Uploads are buffered and checked before they are entered; checking is half-automatic with AI.
- Recipes hold no images, only any number of links into a CDN whose URL is not known yet.
- Recipes have tags (health-condition, diet/composition and nutrition tags).
- First home: `interdata.mealshop.luehr.dev`.

## Customer decisions (2026-11-28, second round)
- Condition tags without evidence, checked for plausibility (F43-CLAIMS).
- Open upload for registered users; registration separate and undecided; no authorship kept
  (F43-RIGHTS).
- Check escalation: rules, self-hosted micro model, frontier model with moderator approval
  (F43-AI).

## Proposals (not accepted)
- **Reuse the existing shapes.** Raw uploads use the `imports::Candidate` shape; accepted recipes
  use the existing recipe/line/quantity shape plus `tags` and `image_links`. Household servers
  can then consume published recipes without a translation layer.
- **Submission is immutable; checks and review are separate records.** Mirrors the suggestion
  inbox and the import receipt: the original stays intact, findings never overwrite it.
- **AI proposes, rules decide, a person accepts.** Models may suggest matches, conversions and
  tags, in the customer's escalation order (rules, self-hosted micro model, moderator-approved
  frontier model). Exact-match, dimension and provenance rules are deterministic code. Publication always
  needs a reviewer; whether clean submissions may ever auto-publish is a later decision.
- **Tags are a closed vocabulary with classes.** Composition and nutrition tags are derived and
  reproducible; condition tags are declared, plausibility-checked and labelled unverified (F43-CLAIMS). Free text tags would make
  filtering unreliable and health wording uncontrolled.
- **Image links are an allowlisted-origin list.** HTTPS only, configured CDN origin, never
  fetched by the service. Avoids SSRF and hotlinking arbitrary hosts.
- **No contributor accounts in this service.** Uploads carry no stored identity (F43-RIGHTS).
  Moderators still authenticate, with operator-issued tokens stored hashed. Uploads present one
  operator-configured secret until registration exists (customer decision F43-ADMISSION).

## Decisions required before later slices (need an ADR in `docs/adr/`)
- Persistence for review and publication (slice 3). Slice 1 uses a file buffer (ADR 0040); a
  multi-writer review queue suggests PostgreSQL, which adds a database crate.
- Which micro model is self-hosted and where, which frontier provider, and what data each stage
  receives (order decided in F43-AI).
- Deployment: container image, Helm values, DNS and TLS for the new host.
