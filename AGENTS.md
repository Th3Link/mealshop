# Working on Mealshop

## Language and scope
- Repository content, code, commits and PRs are English. Talk to the customer in German (du).
- Read README.md, docs/requirements.md and the affected spec before changing behaviour.
- Product vision is not implemented functionality. Keep status and acceptance evidence honest.
- This initial bootstrap is explicitly authorized to push to main. Subsequent work uses
  story/<slug>, fix/<slug> or chore/<slug> and reviewable PRs; never force-push shared branches.
- **Final reply to the customer: exactly three short lines.**
  1. 🟢/🟡/🔴 what changed and its status. 🟢 only with the exact check that was run; 🟡 not
     tested; 🔴 blocked or a guess.
  2. What the customer has to do ("nichts" if nothing).
  3. What is still left to do ("nichts" if nothing).
  Details go into PR descriptions, specs and plans, not into chat.
- **Questions** only when a decision is the customer's. Format: own block
  `Frage <n> (<KURZ-ID>): Titel`, options A/B/C, recommendation, default if unanswered.
  
## Spec-driven workflow
One folder per feature: specs/NNNN-slug/{specification.md,adrs.md,plan.md}; add ux-notes.md
before a UI feature becomes Ready. specification.md is the acceptance contract, plan.md
tracks implementation, specs/README.md is the index. Use Draft → Ready → In progress → Done.
Complete a usable slice per PR and update docs and status in the same change.
Record cross-cutting decisions in docs/adr/NNNN-slug.md using context, alternatives,
decision, consequences and status. Distinguish customer requirements from proposals.
Ask only for unresolved product decisions that block the current slice; record safe proposals.
Do not introduce agents, extra crates, abstractions or features just to fill the architecture.

## UI and localization
- `src/web/` is a disposable test demo. Keep it pragmatic; do not expand product design there.
- Product web clients target Vue 3: a Nextcloud-integrated shell and a standalone server shell
  (F22 / ADR 0007). Flutter remains the app target. Do not start those clients without their slice.
- English is the base for coded messages and canonical content, with stable translation identifiers.
- Use keyed UI messages and language-independent ingredient IDs; keep localized names/aliases
  in translation fields. Localized display/input must preserve exact quantities. Do not silently replace original source content with unreviewed translations.
- Significant product UX/UI work needs its own specification/review; involve a dedicated agent
  when the customer requests it, rather than proactively spawning one for demo polish.

## Engineering
- Add crates/adapters only when an accepted feature needs them, with an ADR.
- Forbid unsafe code. No unwrap/expect outside tests. Return actionable errors.
- Quantities require dimensions, units, precision and provenance; no unqualified floating point
  or implicit ml-to-g conversion. Preserve original input alongside normalized values.
- Allergies are hard constraints; preference learning cannot override them. Unknown data stays unknown.
- Never log secrets or household health data. Imported content keeps its source and sharing rules.
- No invented nutrition/health claims; evidence and uncertainty belong in the model and UI.

- Curate ingredients from exact primary records with retained source/version/licence. Never populate
  unresolved product profiles from a nearby food. Run the offline catalogue import/verifier checks.

## Gate before push
```sh
python3 scripts/import_catalogue.py --check
python3 scripts/verify_catalogue.py
node --test tests/i18n.test.mjs
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo deny check licenses
cargo audit
```
