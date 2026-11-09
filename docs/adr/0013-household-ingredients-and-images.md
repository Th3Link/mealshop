# ADR 0013: household ingredient declarations and recipe images

Status: Accepted for implementation, 2026-11-09 (customer-requested F24)

## Context
The native client needs inline ingredient creation, archive, favourites, tags and composite identities, and optional recipe images. The bundled catalogue is curated primary data and must remain immutable. Saved recipe sources and hard dietary exclusions must survive subsequent edits.

## Alternatives
Editing the global bundled catalogue breaks isolation and provenance. Inventing a primary source or estimating a composite from a nearby food is unacceptable. A new database adapter is unnecessary for this bounded existing file-store deployment.

## Decision
Store a bounded, revisioned household ingredient overlay beside profiles under the existing exclusive household lock. Share that store with recipe creation and dietary review. Household declarations carry an explicit provenance kind; an absent source URL is permitted only for an explicitly declared, unknown-nutrition record. Do not fabricate source URLs, nutrient readings or licences. Keep English canonical fields and reviewed translation fields separate.

Store composition as ingredient membership, without implicit mass proportions. Validate references, duplicates and cycles. Expand both current and pinned composition during dietary review and retain their union of positive allergen declarations. Tags remain descriptive unless explicitly added to a person’s hard excluded tags; exact case-insensitive tag matches then block meals, including nested/pinned composition. Structured declared allergens and ingredient exclusions also block meals. Ordinary tags never establish safety. Preserve components in pinned recipe snapshots. Hold the shared ingredient guard through plan review and durable application, and generate the reviewed version from that same guarded state. A separate dietary revision ignores favourite/archive-only changes; legacy overlays without it conservatively use their existing global revision. Archiving affects new selection, not historic reads or constraints.

Recipe images are immutable private household assets, independent of recipe revisions. Accept bounded raster uploads only; validate signatures and dimensions, reject active formats, serve fixed MIME types with no-sniff and private cache policy. Recipe revisions retain optional image IDs; removing the current image does not destroy historic assets. Use existing authenticated household routes and filesystem primitives, without additional crates or external image requests.

## Consequences
Local and Nextcloud household APIs share the same overlay semantics. Existing catalogue and recipe JSON remain readable through defaulted additive fields. User-provided labels and allergen declarations are shown as declarations, not reviewed primary evidence. Composite nutrition remains explicitly unknown without exact composition evidence. Shared favourites are household annotations. Existing source records are archived only within that household.
