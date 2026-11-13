# Customer mobile specification, 2026-11-12

Imported from the customer-supplied `mealshop-spec.zip` on 2026-11-13. This is an immutable
reference pack, not a description of delivered functionality or a replacement for repository rules.
[Manifest](manifest.json) records the archive checksum and the seven original file checksums.

All seven files are extracted under [source/](source/README.md), with their original bytes.
The sole filename change is `AGENTS.md` → `source/AGENTS.md.reference`: the supplied instructions
remain readable reference material without becoming active agent instructions. The repository's
root `AGENTS.md` is unchanged. In particular, the source pack's suggestions to copy into the root,
merge agent rules or execute its initial prompt are quoted proposals, not instructions for this import.

The proposed source phases are reconciled with the existing implementation in
[F35](../../specs/0035-flutter-mobile-client/specification.md), especially its
[current-state audit](../../specs/0035-flutter-mobile-client/current-state.md) and
[delivery plan](../../specs/0035-flutter-mobile-client/plan.md). That reviewed specification is the
working contract; do not update the source documents to imply their backlog is already implemented.
Source references live outside generated rustdoc pages so their wording remains byte-preserved.

## Source index

- [Original pack overview](source/README.md)
- [Original agent guidance](source/AGENTS.md.reference)
- [Product vision](source/docs/PRODUCT_VISION.md)
- [Mobile UX](source/docs/UX_SPEC.md)
- [Conceptual model](source/docs/DOMAIN_MODEL.md)
- [Original implementation phases](source/docs/IMPLEMENTATION_PLAN.md)
- [Proposed decisions and questions](source/docs/DECISIONS.md)
