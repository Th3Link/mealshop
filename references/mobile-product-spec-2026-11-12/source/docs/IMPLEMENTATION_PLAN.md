# MealShop — Agent-ready implementation plan

Status: backlog proposal. **All tasks start as `not audited`; no claim of missing functionality is made.**

## Phase 0 — Repository audit and alignment (first task)
- [ ] Locate existing UI clients, Rust services, Nextcloud integration, schemas, API contracts, migrations and tests.
- [ ] Create `docs/CURRENT_STATE.md`: capability × status (implemented / partial / absent / unclear), with file paths and evidence.
- [ ] Identify reusable components and incompatibilities with `PRODUCT_VISION.md`, `UX_SPEC.md` and `DOMAIN_MODEL.md`.
- [ ] Document Nextcloud vs standalone auth/deployment/API differences.
- [ ] Recommend the smallest vertical slice; do not start a rewrite.

**Acceptance:** evidence-based current-state matrix, key risks, proposed first PR and test plan.

## Phase 1 — Mobile shell and household tour
- [ ] Implement/adapt five-tab mobile navigation and basic design tokens.
- [ ] Add household/person creation and editing against verified APIs.
- [ ] Build skippable, resumable profile tour with dietary presets + exceptions.
- [ ] Separate hard constraints from dislikes and preferences.
- [ ] Add weekly attendance and per-meal prep preferences; profile summary.

**Acceptance:** two people with different dietary rules can be created/edited; invalid combinations are explained; drafts survive interruption; accessible mobile layout.

## Phase 2 — Planning vertical slice
- [ ] Inspect/extend server plan proposal contract and constraint validation.
- [ ] Implement week/day plan and per-meal participant/serving edits.
- [ ] Add swap dish with conflict explanations and confirmation states.
- [ ] Add transparent shopping-diff preview.

**Acceptance:** server-generated proposal is editable without silent allergy violations; attendance and servings change required amounts; relevant tests pass.

## Phase 3 — Full shopping workflow
- [ ] Fast manual list entry, quantity/unit/category/store, check-off and sync.
- [ ] Preserve recipe/meal/manual contribution provenance.
- [ ] Support generic ingredients and optional specific barcode products.
- [ ] Define offline queue, idempotency and conflict behavior.

**Acceptance:** two devices can concurrently use one list; manual and checked items survive plan edits; quantities aggregate without losing sources.

## Phase 4 — Mealprep, pantry and nested recipes
- [ ] Add prepared-batch and allocation lifecycle, storage metadata.
- [ ] Add buy-vs-make choice and nested recipe expansion with cycle checks.
- [ ] Integrate prep tasks with planned meals and shopping.

**Acceptance:** Sunday six-portion batch can cover two later meals plus frozen leftovers; shopping reflects only missing ingredients.

## Phase 5 — Capture and import
- [ ] Camera OCR and human review of uncertain recipe fields.
- [ ] Share-sheet URL intake; inspect legally/technically supported source extraction.
- [ ] Barcode lookup and label OCR with explicit source/quality indicators.

**Acceptance:** import is editable, missing values remain unknown, failures recover gracefully; no unsupported social-media download assumptions.

## Phase 6 — Cooking assistant
- [ ] Step mode, independent timers, background-safe task persistence.
- [ ] Multi-day prep/fermentation reminders and notification permissions.

**Acceptance:** timer survives screen closure/restart per platform capabilities; prep tasks trigger at correct times and can be rescheduled.

## Phase 7 — Lightweight activity and future integrations
- [ ] Activity type, duration, optional intensity, history.
- [ ] Approximate kcal only when defensible, with uncertainty.
- [ ] Research smart-scale abstraction; defer hardware implementation.

**Acceptance:** 45-minute jog or 60-minute ride can be logged without personal weight data.

## Work item template
For each PR/agent task:
- User story and user-visible outcome
- Existing implementation evidence (paths, APIs)
- Scope / out of scope
- Data/API impact and migration strategy
- UI states (normal/empty/loading/error/offline)
- Privacy/security and accessibility checks
- Automated tests and manual acceptance steps
- Documentation/decision updates

## Recommended first agent prompt
> Read AGENTS.md and docs/PRODUCT_VISION.md, docs/UX_SPEC.md, docs/DOMAIN_MODEL.md and docs/IMPLEMENTATION_PLAN.md. Audit the current repository against these target specs. Write docs/CURRENT_STATE.md with evidence-based implemented/partial/missing status, list relevant APIs and existing tests, and propose one minimal end-to-end PR. Do not implement features or rewrite architecture yet. Ask before making irreversible schema or platform decisions.
