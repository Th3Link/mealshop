# F02: ingredient catalogue and nutrition

Status: accepted through merged PR #4. This is a disposable test demo with
313 ingredients and 17 nutrient/component definitions, not a production family planner.

## Try the browser demo
```sh
cargo run --locked
```
Open [the local workbench](http://127.0.0.1:3000/). The initial input is 100 g oats and 25 g
chia for two servings. Choose **Calculate nutrition**: energy is 510.50 kcal for the recipe,
255.25 kcal per serving; protein is 21.03 g for the recipe and 10.51 g per serving.

Change chia to psyllium husks. Complete totals become unknown: the known energy subtotal
remains 389.00 kcal for the recipe and 194.50 kcal per serving, explicitly labelled partial.
Change preparation to **Will be cooked / yield unknown** to see the retention/yield warning.
Negative amounts produce a visible error and preserve the input. Search the notebook by
name or alias (for example Hafer); expand composition to inspect source values per 100 g.

The browser uses same-origin API calls and no third-party assets/analytics. It stores no
recipe, health or household data, and sends none to external nutrition providers.

## HTTP API
- `GET /health`: unchanged process-health contract (`ok` plus a newline).
- `GET /api/ingredients?q=hafer`: catalogue version, name/alias-matched ingredient records
  and nutrient definitions (ID, label, unit, group). Search is case-insensitive.
- `GET /api/ingredients/oats`: the complete record including aliases, edible form, basis,
  source/version/licence, uncertainty, nutrient readings and education. Missing IDs return 404.
- `POST /api/nutrition/calculate`: stateless source-based aggregation. Invalid amounts,
  unsupported fields, unknown ingredient IDs or infeasible bounds return a structured JSON
  error. No mutation endpoint is provided.

```sh
curl --fail-with-body http://127.0.0.1:3000/api/nutrition/calculate \
  -H 'content-type: application/json' \
  --data '{"lines":[{"ingredient_id":"oats","grams":"100"},{"ingredient_id":"chia","grams":"25"}],"servings":2,"preparation":"as_recorded"}'
```

Each nutrient result names its unit and coverage (`complete`, `partial`, `unknown`). `total`
and `per_serving` are decimal strings only when every contributing ingredient has a known
value; otherwise they are null. `known_total`/`known_per_serving` are labelled subtotals,
not complete values. For an entirely unknown nutrient, those known subtotals are zero
because no known contribution exists; status/null totals preserve that distinction.
Missing ingredient IDs are deduplicated. Repeated recipe lines still contribute separately.
The result retains original gram strings, source records, form/basis, catalogue version,
source uncertainty, educational conditions/limitations and preparation context.

## Quantity and calculation contract
All input masses are grams of the record's edible form; the supported basis is
`per_100g_edible_portion`. No volume, piece, yield or raw/cooked conversion is inferred.
Input source values and masses are decimal strings in [0, 1000000000], with at most six
fractional digits and no sign/exponent/whitespace; recipe masses must be strictly positive.
Servings are integer counts in [1, 1000000]; requests contain 1–100 lines. These are
bounded-arithmetic/input-validation limits for this implementation, not subscription limits.

Amounts retain their original spelling and an integer scaled by 1000000. For each nutrient,
the numerator sums `source_atoms * gram_atoms` and the denominator is `100 * 1000000²`.
The per-serving denominator additionally multiplies by serving count. Checked integer
arithmetic preserves the entire fraction until final formatting, with two decimal places
and half-away-from-zero rounding (all valid values are nonnegative). At the bounds, the
scaled sum is at most 10³² and the rounding numerator at most 10³⁴, within a 128-bit unsigned
integer. Portion values are never calculated from rounded recipe totals. This is arithmetic
precision, not a claim that the source food data are accurate to two decimals.

The registry distinguishes kcal/kJ, g, mg and µg. Carbohydrate specifically means the
by-difference value including fibre; folate is total folate, not dietary folate equivalents.
Other definitions require separate IDs before addition. Energy is retained from the source,
not recomputed from macros. Micronutrient coverage is a selected starter subset, not exhaustive.

## Use an operator-provided catalogue
```sh
MEALSHOP_CATALOGUE_PATH=/absolute/path/catalogue.json cargo run --locked
```
This file replaces the entire bundled catalogue at startup. The schema matches
[data/catalogue.json](../../data/catalogue.json). Schema 1 requires a nonempty version;
ingredient IDs are unique ASCII identifiers. Records need aliases (possibly empty), form,
basis, full source fields, uncertainty, a nutrient map and education (possibly empty).
Nutrient readings contain `amount` (decimal string or null) and `uncertainty`. Omitted nutrient
keys are also unknown. Only registry nutrient IDs are accepted; units are fixed by that registry.
Sources and evidence links use HTTPS. Education requires component, title, summary, evidence
reference, conditions and limitations, independent of measured nutrient amounts.

The immutable catalogue is public operator-provided reference data. It is not a place for
private household/health data. Catalogue writes/authentication, persistent ingredient editing,
user unit preferences, cooking retention models and the Flutter shell remain separate
features (F03/F09/F18). The [local recipe authoring slice](recipes.md) now saves recipes
and pins catalogue subsets independently of this public reference dataset.

## Limits and provenance
JSON recipe bodies are limited to 128 KiB; catalogue input to 1 MiB and 1000 records.
The server rejects malformed/oversized/unsupported catalogues and exits on missing files.
An empty catalogue is valid and the browser explains why calculation is unavailable.
Source dates, sample counts and educational references are snapshots, not live provider sync.

See [starter data provenance](../../data/README.md) and
[F02 acceptance criteria](../../specs/0002-ingredients-nutrition/specification.md).

## Demo versus product UI
The customer classified this surface as a pragmatic demo. Keep design effort minimal.
Nextcloud/standalone product shells are F22/F30 (Vue 3); Flutter is separate in F18.
F21 starts with German/English demo messages, starter names and exact decimal presentation.
Source uncertainty and educational text remain explicitly original-language until reviewed
translations are available. UI language does not change nutrient IDs, units, source facts or
canonical API decimals. German entry uses a comma; English uses a dot, without grouping.

## English base and localized demo
Canonical IDs/names remain English; German ingredient names/aliases/forms use `translations.de`.
The language selector changes keyed interface messages and display, not nutrient values or stored
API decimals. Enter e.g. `100,5` grams in German or `100.5` in English, without grouping separators.
Valid inputs/results survive language changes exactly. Previously invalid inputs require an
explicit edit before they can be reinterpreted in a new locale. Unsupported languages fall back to English.
Search covers both languages and common aliases/plurals (e.g. Weizen 405, Hafermilch, Eier).

The 313-item kitchen catalogue is generated offline from reviewed BLS/USDA selections; see
[data provenance and rebuild instructions](../../data/README.md). Available carbohydrate and
by-difference carbohydrate are separate definitions; food folate is separate from total folate.
Mixed-source recipes may show partial values for these definitions, with an explanatory warning.
Error responses add stable `code` alongside the English `error` text; calculation results add
`warning_codes` parallel to the retained English `warnings`. The demo localizes these codes.

## Typed quantity conversion (F03 initial slice)
`POST /api/quantities/convert` accepts `{"quantity":{"amount":"1.250000","unit":"kg"},"target":"g"}` and returns the original input, its dimension and an exact converted quantity. Supported unit identifiers are `g`, `kg`, `ml`, `l`, `tsp`, `tbsp` and `piece`. Cross-dimension and spoon conversions require ingredient data or a named standard and return stable error codes. Conversion rejects values outside the six-decimal precision bound rather than rounding. This endpoint is not yet connected to recipes or user preferences.
