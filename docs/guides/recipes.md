# Local recipe authoring

The executable stores locally authored recipes in `var/recipes` (ignored by Git). Override
with `MEALSHOP_RECIPE_DIR=/absolute/dedicated/path`; an empty value is an error. The directory
is created on first run. Only loopback listen addresses are currently supported. There is no
authentication: this is a trusted local, single-household authoring slice, not hosted access.
Stop the server before operating on the storage files. Use a local filesystem supporting
hard links/fsync; Linux is the initial validated target. Existing directory permissions are
not modified; newly created Unix directories/files use 0700/0600.

## Demo
Use the existing demo at `http://127.0.0.1:3000/`. Enter a title, author attribution, recipe
text language (English/German), catalogue ingredients and gram amounts, serving count and
optional steps with active/passive seconds. German amounts use commas; API amounts use exact
dot-decimal strings. Save creates a new recipe; saving a loaded recipe creates a revision.
The saved list and load button retrieve persisted recipes and their saved serving estimates.
Load/new actions explicitly replace inputs. Failed saves keep all inputs. On a revision
conflict, copy your edits before loading the latest revision and reconciling them manually.

Changing the authoring serving count divides the same batch. The separate scaling preview uses the saved original revision and rounds each ingredient half up to 0.000001 g. Nutrition uses the exact original serving ratio before display rounding. A
saved recipe uses its own source snapshot, even after the catalogue changes. Saving a new
revision uses the current catalogue and recalculates nutrition. Missing current ingredient
IDs cannot be silently substituted: restore the catalogue or explicitly replace them.
Unknown nutrient values remain unknown. Authors are self-reported, not authenticated.
This remains a pragmatic demo; product clients and process timers are separate features.

## API
```sh
curl --fail-with-body http://127.0.0.1:3000/api/recipes \
  -H 'Content-Type: application/json' -d '{
    "title":"Porridge", "language":"en", "author":"Local cook",
    "servings":2, "preparation":"cooking_unmodelled",
    "lines":[{"ingredient_id":"oats","grams":"100.500000"},
             {"ingredient_id":"chia","grams":"25"}],
    "steps":[{"instruction":"Mix and rest.","active_seconds":60,"passive_seconds":300}]
  }'
```
POST `/api/recipes` returns HTTP 201 with `id`, `revision`, `recipe` and `nutrition`.
GET `/api/recipes` lists latest summaries. GET `/api/recipes/recipe-00000001` retrieves the
latest recipe and estimate; append `?revision=1` for a historical revision. PUT that same
path with `{"expected_revision":1,"recipe":{...complete recipe fields...}}` creates revision 2.
Concurrent stale writes return HTTP 409 / `revision_conflict`. No PATCH or DELETE is offered.
Other stable codes include `invalid_recipe`, `recipe_not_found`, `invalid_revision`,
`recipe_storage_full` and `recipe_storage_failed`. Recipe responses are marked `no-store`.
Recipe IDs and revisions are local to this store, not globally federated identifiers.

Bounds: 128 KiB body; 1–100 ingredient lines; positive amounts up to 1000000000 g with six
fractional digits; 1–1000000 integer servings; title/author 1–200 UTF-8 bytes; language en/de;
0–100 steps with 1–4000 byte instructions and 0–604800 seconds per active/passive duration.
The local store holds at most 1000 immutable revisions, each at most 2 MiB. It does not delete
old revisions to make room. Capacity expansion/migration belongs to F18.

## Backup and recovery
Stop all processes using the directory; copy the entire directory (including every revision)
to a separate backup location. Never edit files while a server is running: reads use an
in-memory index and external writers are unsupported. Restore into an empty dedicated directory,
then start the server and verify list/detail/portion results. Full adapter migration is not
implemented. Do not delete individual revisions: each recipe history must start at 1 and be
consecutive. Preserve the original backup when repairing any damaged copy.

Normal graceful shutdown removes `.mealshop.lock`. After a crash it remains. Check that no
process is using the directory before removing a stale lock; PID text is advisory, not proof
that no writer exists. Starting with a live/stale lock fails closed.

A `.pending` file means publication may have been interrupted. Stop writers, back up the
entire directory and inspect `.pending` plus its recipe ID/revision target. If the target
exists and is byte-identical, it was published; retain the target and remove `.pending` from
the recovery copy. If no target exists, preserve `.pending` outside the live directory before
removing it there (the save was not published). If they differ, restore a known-good complete
backup or reconcile offline; never replace the target blindly. Remove only a verified stale
lock and restart against the recovery copy. Startup validates schema, source snapshots,
quantities, filenames and consecutive revision histories; corruption never becomes an empty
store. A storage error returns 503 and disables later writes until restart/recovery. After a
network timeout the save may have committed: refresh the list/history before retrying, since
POST retry idempotency is not implemented. Unacknowledged valid published revisions can be
visible after recovery, so inspect the latest revision before resubmitting edits.

## Embedding
`router()` / `router_from_catalogue_json()` remain stateless catalogue/nutrition applications.
`router_with_recipe_directory(json, directory)` explicitly enables durable recipes and holds
its exclusive lock until every cloned router/request state is dropped. The caller must enforce
the trusted local access boundary; the library does not provide authentication or listener policy.

GET `/api/recipes/{id}/scale?servings=3&revision=1` previews a saved revision at another
serving count without writing it. Omit revision for the latest. The response includes both
serving counts, scaled gram strings and nutrition; unsupported or zero rounded amounts fail.

## Original quantity and conversion evidence
A recipe line can supply `quantity: {"amount":"0.125","unit":"kg"}` instead of grams.
The server normalizes to grams and stores both. If both are supplied they must agree exactly.
Legacy gram-only lines remain valid. For ml/l/piece/tsp/tbsp input supply `conversion` with
`ingredient_id`, exact catalogue `form`, `source_url` (HTTPS), `version`, `uncertainty`,
`grams_per_base_unit` (g/ml or g/piece), and optional `spoon_standard` and
`millilitres_per_spoon`. Spoon input requires both optional fields; name the selected spoon
and regional standard. These are user-declared evidence, not independently verified sources.
Cross-dimension arithmetic rejects unsupported precision; it never assumes water density.
The immutable revision retains this context. Scaling previews normalize from saved grams,
leaving original inputs/evidence in the source revision. Local demo g/kg preference affects
mass display only; other users' saved recipes are not rewritten.
