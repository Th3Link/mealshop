# Starter ingredient catalogue

`catalogue.json` is the reviewed, read-only starter dataset, schema version 1. It is loaded
once at startup and can be replaced with `MEALSHOP_CATALOGUE_PATH`. It is not a household
database, an allergen database or a complete food composition database.

## Provenance and licensing
- `sources/usda-oats.json`: original USDA API response for FDC 169705, retrieved 2026-11-07.
- `sources/usda-chia.json`: original USDA API response for FDC 170554, retrieved 2026-11-07.
- Both are SR Legacy records, published 2019-04-01. Oats refers to dry uncooked grain, not
  every branded rolled-oat product; chia refers to dried seeds. Select matching forms.
- [USDA API guide](https://fdc.nal.usda.gov/api-guide/): data are public domain/CC0-1.0.
  Original numerical facts remain CC0; the software licence does not change those rights.
  Recipe calculations run locally against these frozen records, with no USDA API call/key.
- The catalogue retains 13 supported nutrient amounts per source in their original units.
  Decimal spelling is normalized by removing insignificant zeros; no value is rounded or inferred.
  `python3 scripts/verify_catalogue.py` checks the 26 values against the frozen responses.
- Psyllium husks is an identity-only project record. No verified product-specific nutrient
  source was selected, so every amount remains null. Its note cites a guideline, not composition.
- Project-authored identity metadata, educational summaries and code are AGPL-3.0-only.
  External publications are linked, not redistributed as full text.

## Interpretation
Every value is per 100 g edible portion of the stated form. Source uncertainty includes
reported sample counts/ranges where available; a sample range is not a confidence interval.
USDA carbohydrate here is calculated by difference and includes dietary fibre. This must
not be mixed with EU-style available-carbohydrate values under the same nutrient ID.

Beta-glucan and soluble fibre are not measured in the selected nutrient records. Their
amounts stay unknown, even though related educational notes exist. No ingredient serving
is declared to satisfy a health-claim condition. The oats note links Health Canada's claim
assessment; the chia note links EFSA's composition/safety opinion; the psyllium note links
the AGA/ACG guideline. Conditions and limitations remain visible with each summary.

## Editing a local catalogue
Copy `catalogue.json` and edit whole ingredient records with reviewed sources. Keep stable
IDs and source record/version/licence/retrieval fields. Decimal amounts must be JSON strings;
null/omitted nutrients mean unknown. A source zero is a real zero. The nutrient vocabulary,
units, input limits and API examples are documented in [the nutrition guide](../docs/guides/nutrition.md).
No metadata or quantities are rewritten during calculations. Restart to load a changed file;
invalid catalogues fail startup instead of silently falling back to the bundled data.
