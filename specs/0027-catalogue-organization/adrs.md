# Explicit organizational mapping

Status: Accepted

Store a versioned mapping of exact existing IDs to bilingual groups alongside the curated catalogue.
Do not mutate frozen source records or infer a group's nutrition from a representative member. Runtime
clones only the structural record shape, clears quantitative/evidence/health fields and assigns explicit
organizational provenance with unknown nutrition. Group membership is independent of composition.
Keep previous flour group IDs so existing household annotations remain attached. User annotations can
still move or ungroup an ingredient; startup never rewrites household overlays or historical snapshots.
Malformed mappings fail startup. Offline checks require coverage of every bundled ID exactly once.
