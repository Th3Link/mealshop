# Working on Mealshop

## Language and scope
- Repository content, code, commits and PRs are English. Talk to the customer in German (du).
- Read README.md, docs/requirements.md and the affected spec before changing behaviour.
- Product vision is not implemented functionality. Keep status and acceptance evidence honest.
- This initial bootstrap is explicitly authorized to push to main. Subsequent work uses
  story/<slug>, fix/<slug> or chore/<slug> and reviewable PRs; never force-push shared branches.

## Spec-driven workflow
One folder per feature: specs/NNNN-slug/{specification.md,adrs.md,plan.md}; add ux-notes.md
before a UI feature becomes Ready. specification.md is the acceptance contract, plan.md
tracks implementation, specs/README.md is the index. Use Draft → Ready → In progress → Done.
A Draft is not an authorization to implement it. Keep criteria, implementation and tests
traceable. Complete a usable slice per PR and update docs and status in the same change.
Record cross-cutting decisions in docs/adr/NNNN-slug.md using context, alternatives,
decision, consequences and status. Distinguish customer requirements from proposals.
Ask only for unresolved product decisions that block the current slice; record safe proposals.
Do not introduce agents, extra crates, abstractions or features just to fill the architecture.

## Engineering
- Rust + Tokio + Axum backend; Flutter client is planned. Keep domain logic independent of I/O.
- Add crates/adapters only when an accepted feature needs them, with an ADR.
- Forbid unsafe code. No unwrap/expect outside tests. Return actionable errors.
- Quantities require dimensions, units, precision and provenance; no unqualified floating point
  or implicit ml-to-g conversion. Preserve original input alongside normalized values.
- Allergies are hard constraints; preference learning cannot override them. Unknown data stays unknown.
- Never log secrets or household health data. Imported content keeps its source and sharing rules.
- No invented nutrition/health claims; evidence and uncertainty belong in the model and UI.
- Do not weaken failing checks, ignore advisories or loosen licences just to make CI green.

## Gate before push
```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo build --release --locked
cargo deny check licenses
cargo audit
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
```
CI additionally checks MSRV, coverage, SBOM and binary metrics. Run affected checks during work,
then the full applicable gate once before pushing. Report exact checks and unresolved failures.
See docs/guides/development.md and docs/adr/index.md. Source conventions adapted from Curvyo;
see docs/provenance.md. Customer instructions take precedence over this file.
