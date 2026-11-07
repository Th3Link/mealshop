# ADR 0008: curated ingredient sources and English-base identities

Status: Customer scope accepted; import/model details proposed
Date: 2026-11-07

## Context
The customer requested a broad kitchen catalogue and clarified that everything coded needs
an English base plus identifiers for language changes. German flour types and dairy grades
must be distinguished rather than approximated by generic US foods. BLS 4.0 provides native
English/German names and detailed component provenance; USDA complements it.

## Decision
- Canonical ingredient IDs remain stable English slugs; category IDs are English identifiers.
  `name`, `aliases` and `form` are the English base. Optional `translations.<language>` contains
  localized names/aliases/forms, keyed by ingredient identity and field. English remains the base,
  never an overriding entry in the translation map. Old schema-1 files default to English/other
  category with no translations, so existing operator fixtures remain loadable.
- Curate exact source-record selections in a separate manifest. Commit selected source facts,
  original descriptions, nutrient units, acquisition methods/references, source sheet/row or numeric
  JSON tokens and archive/workbook hashes. Regenerate the runtime catalogue deterministically.
- BLS source data retain CC-BY-4.0 attribution to Max Rubner-Institut, version 4.0 (2025), DOI
  10.25826/Data20251217-134202-0. USDA SR Legacy data retain CC0/public-domain status. Software
  stays AGPL-3.0-only; provider licences are not relabelled as AGPL.
- Runtime amounts support six decimal places. Imported source representations are rounded once
  to that precision using half-up normalization; original cell/token strings remain in snapshots.
  Missing, negative/unrepresentable and below-resolution positive values remain unknown, not zero.
- BLS `CHO` maps to `available_carbohydrate`; it does not populate USDA `carbohydrate` (by difference).
  BLS `FOLFD` and USDA nutrient 1187 map to `food_folate`; total folate/DFE stay separate. Units and
  basis are checked before importing. No energy, fibre or carbohydrate value is inferred from another.
- Unresolved product identities exist with no nutrient profile. Do not substitute related flour,
  a neighbouring fat grade, a brand formula or a generic whey reference for the requested variant.
- The demo chooses language by message IDs and translation fields; API numbers remain dot-decimal.
  Additive error/warning codes support localized presentation without matching English sentences.

## Alternatives
Using only USDA would miss German flour types and fat grades. Importing all foods without
review would add composite dishes and ambiguous mappings. Assigning plausible averages would
hide source gaps. Mixing available and by-difference carbohydrate would produce misleading totals.

## Consequences
The curated catalogue contains 313 items; 29 intentionally remain identity-only. The current
17-definition nutrient registry is a subset of the providers' data, not exhaustive coverage.
The file-based demo catalogue is close to its pre-existing 1 MiB validation bound; future wider
coverage belongs with explicit storage/loading work rather than an unreviewed bound increase.
Full versioned content translations and production language negotiation remain F21 work.

## Primary sources
- [BLS 4.0 download and licence](https://blsdb.de/download).
- [Max Rubner-Institut BLS overview](https://www.mri.bund.de/de/institute/ernaehrungsverhalten/forschungsbereiche/bundeslebensmittelschluessel-bls/).
- [USDA downloads](https://fdc.nal.usda.gov/download-datasets/) and [API/data licence](https://fdc.nal.usda.gov/api-guide/).
