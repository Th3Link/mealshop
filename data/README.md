# English-base kitchen ingredient catalogue

`catalogue.json` contains 313 curated ingredient identities: 284 source-backed profiles and
29 identity-only records. It is immutable reference data loaded at startup, not a household
database or an allergen-safety database. All base names/aliases/forms are English; German
names/aliases/forms live under `translations.de`. Stable ingredient IDs plus field names form
the content-translation identifiers. Unit/quantity meanings never depend on display language.

## Sources, attribution and licences

### BLS 4.0
Max Rubner-Institut (2025): Bundeslebensmittelschlüssel (BLS), Version 4.0 — Deutsche
Nährstoffdatenbank. Karlsruhe. DOI: [10.25826/Data20251217-134202-0](https://doi.org/10.25826/Data20251217-134202-0).
Source: [official download](https://blsdb.de/download), retrieved 2026-11-07.
Original BLS data are licensed [CC-BY-4.0](https://creativecommons.org/licenses/by/4.0/).
No endorsement by the publisher is implied.

`data/sources/bls-4.0-selected.json` retains 248 selected source rows with original German/English
names, numeric cell strings, units, acquisition methods, references and workbook sheet/row.
Archive/workbook SHA-256 hashes identify the exact downloaded source. Adaptations: reviewed
food selection, English-base structure, German translation fields/aliases, a subset of nutrient
components and documented six-decimal normalization. Source numeric values are not universal
brand/package values. The provider's source-data licence remains intact in API provenance.

### USDA
[USDA FoodData Central](https://fdc.nal.usda.gov/api-guide/) data are public domain/CC0-1.0.
The original oats/chia API responses remain in `sources/usda-oats.json`/`usda-chia.json`.
`sources/usda-selected.json` holds 34 more reviewed SR Legacy records from the official April
2018 archive (records published 2019-04-01), with archive hash and original numeric token spellings
preserved as strings in the extracted snapshot. Source units, versions and record IDs are retained.
Herb states and formulated reference protein powders are named explicitly; a USDA reference
whey formulation is not treated as a universal pure-isolate or manufacturer specification.

### Project records
Psyllium husks and 28 further product/source-specific identities have no verified matching
composition selected. Their numeric values remain absent/null. This includes particular pulse
pastas, rice paper, erythritol, double-cream quark and various protein products. They are searchable
and selectable, but do not inherit nutrients from flour, a nearby fat grade or another product.
Project software and project-authored curation/evidence text remain AGPL-3.0-only. External data
retain their stated licences; third-party publications are linked, not redistributed as full text.

## Nutrient definitions and precision
All amounts refer to 100 g edible portion of the named state. Raw/cooked, fresh/dried,
fortified/unfortified, lactose-free and fat-in-dry-matter variants retain separate identities.
A cheese/quark fat-in-dry-matter grade does not mean that many grams of fat per 100 g food.
Milk is still measured in grams; no ml-to-g conversion or density assumption is introduced.

The current vocabulary has 18 nutrient definitions plus two legacy constituent identifiers. BLS available carbohydrate (`CHO`) excludes fibre and includes
polyols; USDA by-difference carbohydrate includes fibre. They remain separate IDs, not summed or
converted. BLS `FOLFD` and USDA 1187 are food folate, not total folate or DFE. Soluble fibre is used
only where reported; beta-glucan is never extrapolated from total/soluble fibre. Source-specific
energy is retained, not recomputed from macros. Mixed definitions may yield partial totals,
explained by a stable warning code. Source uncertainty/acquisition notes remain visible.

The runtime amount format supports six fractional places. Import normalizes raw numeric
representations (including XLSX binary-decimal artifacts) to that scale using half-up rounding.
Original strings remain in the frozen snapshot; verification checks a maximum 0.0000005 difference
in the nutrient's unit. Missing/unusable and below-resolution positive amounts stay unknown;
they never become an asserted zero. This is an import representation policy, not source accuracy.
Recipe/portion aggregation still uses exact checked integers through final two-place display.

## Reproduce or extend the catalogue
`catalogue-base.json` preserves the three initially reviewed records; `catalogue-selection.json`
is the explicit identity/source manifest. The generated runtime file must not be hand-edited.
All normal generation/verification is offline:

```sh
python3 scripts/import_catalogue.py
python3 scripts/import_catalogue.py --check
python3 scripts/verify_catalogue.py
```

To refresh selected snapshots after reviewing source/version/licence and adding exact manifest
mappings, download the named archives from their official pages, then run:

```sh
python3 scripts/extract_bls_snapshot.py /absolute/path/BLS_4_0_2025_DE.zip
python3 scripts/extract_usda_snapshot.py /absolute/path/FoodData_Central_sr_legacy_food_json_2018-04.zip
python3 scripts/import_catalogue.py
python3 scripts/verify_catalogue.py
```

The extractors use Python's standard ZIP/XML/JSON readers and preserve original source tokens;
they do not modify the source workbooks or fetch the network. Check the source hashes, exact
identity/state, component definitions and aliases in the diff. Source verification and generated
catalogue checks run in CI. There are 3,778 mapped nutrient values in the current snapshot.

An operator's custom JSON file may still replace the catalogue via `MEALSHOP_CATALOGUE_PATH`.
The F34-enriched loader has a bounded 2 MiB/1000-record envelope; wider persistent
catalogues still require F18 loading/storage work. See the
[nutrition demo guide](../docs/guides/nutrition.md) and [F23](../specs/0023-kitchen-ingredient-catalogue/specification.md).

## Curated conversion reference portions
`conversions.json` selects 43 source-specific USDA SR Legacy household portions (CC0-1.0)
from the committed `usda-selected.json` snapshot. It retains portion IDs, original modifiers,
amounts, gram weights and exact food/state identities. Spoon masses are direct USDA household
portion estimates, not inferred g/ml densities or universal regional spoon volumes. Block,
packet and cake sizes are explicit in the source label. No transfer to a nearby food is made.
Run `python3 scripts/verify_conversions.py` to independently check every selected factor.
USDA portion semantics: https://fdc.nal.usda.gov/portal-data/external/dataDictionary
The API `/api/conversions` exposes the list; the demo offers explicit selection and saved
revisions retain the full evidence even if the reference list later changes. Unknown densities
remain unknown. Cross-device preference synchronization is deferred at customer request.

## Positive allergen identity classifications
`allergens.json` defines 14 stable group IDs and 106 explicitly selected positive food-identity
classifications, tied to exact catalogue food records/forms. Groups and food examples are based
on the [Food Standards Agency reference](https://www.food.gov.uk/sites/default/files/media/document/14-Allergens%2016Nov21.pdf).
Selections are Mealshop curation, not FSA analysis of our catalogue or individual retail products.
Only positive associations are recorded; omissions never assert absence. Cross-contact, refined
product exceptions, thresholds and complete brand composition are not inferred. Source coverage
can be expanded without relaxing the review-required behaviour. Run
`python3 scripts/verify_allergens.py` to validate registry/selection identity and provenance fields.

## Organizational groups

`ingredient-groups.json` is project-authored discovery metadata mapping all bundled IDs to 73 bilingual
groups. It supplies no nutrient, allergy or health data and never replaces frozen source records.
`python3 scripts/verify_catalogue.py` checks complete and unique coverage alongside the original
source verifier. `intolerance-triggers.json` is a separate explicit-declaration vocabulary; it is not
part of the regulated fourteen-allergen positive-identity mapping.

## Supplemental nutrition and comparison evidence v1

F34 explicitly extends the earlier exact-only policy **for separate labelled evidence**. The
original `catalogue.json`, source snapshots and 29 identity-only nutrient maps remain unchanged.
`nutrition-estimates-v1.json` is a deterministic supplement matched by ingredient ID, original
publisher/record/version and food state. It provides complete core energy/protein/fat/carbohydrate
coverage for 313 foods and member-derived spans for 73 groups. Original readings always win.
Same-record BLS/USDA supplements add reported sugars, saturated fat and salt where actually stated.
All other envelopes remain estimates, including degenerate zero-width comparisons.

`data/sources/nutrition-evidence-v1.json` freezes selected raw decimal tokens, units, acquisition
metadata and explicit comparison selections. BLS/USDA attribution and archive hashes above apply.
Manufacturer facts retain publisher, product/URL, retrieval and explicit unknown data-licence status;
no product photographs, health claims or full copyrighted pages are redistributed. Public labels are
rounded observations, not measurements of an unidentified household product. Numeric `< x` labels
are conservatively enclosed by 0…x, never silently replaced by zero. An unspecified detection limit
remains unknown. Microgram, carbohydrate and folate definitions remain distinct.

Min/max spans describe selected comparable observations, **not guaranteed bounds or statistical
confidence intervals**. There is no universal percentage. Matching rationale names product/purity,
flavour, processing and state differences. Examples: dry pulse pasta versus dry pulse flour,
partially defatted hemp powders, two per-100-g psyllium declarations. Only supported fields are
filled; no beta-glucan, soluble fibre, allergy or diet inference is made. KoRo rice protein energy
was excluded because of conflicting page/specification declarations and macro/energy consistency;
per-100-ml pudding tables were excluded without density evidence.

Group ranges use explicit `ingredient-groups.json` membership only, retaining observed extrema,
never a guessed typical subtype. A group with incompatible supplied member records receives no
bundled group fallback. Own household ranges and group inheritance preserve explicit evidence.
New recipe revisions pin effective profiles. Legacy records receive immutable-v1 supplemental
analysis in memory, with their original persisted files/history unchanged. A future evidence release
must use a new versioned registry; never revise published v1 in place. Back up before installation:
older executables may reject the new optional persisted fields.

The enriched catalogue exceeds the previous 1 MiB envelope. Its bounded JSON loader now permits
2 MiB/1000 records; the independent **2 MiB recipe record publication/restart guard** remains.
Recipe calculation uses checked integer arithmetic through serving scaling, rounding lower bounds
down and upper bounds up at the final two-place display. Missing unbounded inputs produce an
explicit covered subtotal, never an apparently complete total. Exact-only legacy total fields stay
nullable; new tagged `range` fields carry complete or partial intervals for updated clients.

Offline generation and independent verification (also in CI):

```sh
python3 scripts/import_nutrition_estimates.py
python3 scripts/import_nutrition_estimates.py --check
python3 scripts/verify_nutrition_estimates.py
python3 scripts/verify_nutrition_archives.py --bls /path/BLS.zip --usda /path/USDA.zip
```

The optional archive check verifies archive/workbook hashes and original selected BLS/USDA tokens,
units and acquisition metadata directly, without network access. Manufacturer facts are factual
manual selections from the linked primary declarations; package updates require a new reviewed
release. No household health data are included in these public evidence files.
