# Interdata intake service

Interdata is a separate service that receives recipes and ingredients for a shared data set
([F43](../../specs/0043-interdata-intake/specification.md)). **Only intake exists today:**
uploads are validated for shape and size and buffered unchanged. Nothing is checked for
plausibility, matched, reviewed or published yet, and the service is not deployed anywhere.
Accepted recipes will be published under CC0 1.0.

## Run locally
```sh
INTERDATA_UPLOAD_SECRET="$(head -c 32 /dev/urandom | base64)" \
  cargo run --package mealshop-interdata
```

| Variable | Default | Meaning |
|---|---|---|
| `INTERDATA_UPLOAD_SECRET` | required | Shared upload secret, at least 32 bytes. |
| `INTERDATA_LISTEN_ADDR` | `127.0.0.1:3100` | Listen address. Terminate TLS in front of it. |
| `INTERDATA_DATA_DIR` | `var/interdata` | Submission buffer directory. |
| `INTERDATA_CDN_ORIGIN` | unset | HTTPS origin image links must use. Unset: image links are rejected. |
| `INTERDATA_CAPACITY` | `10000` | Buffered submissions after which uploads are refused. |
| `INTERDATA_UPLOADS_PER_MINUTE` | `60` | New submissions per minute across all uploaders. |

## Run in a container
```sh
docker compose -f deploy/interdata/compose.yaml --env-file /path/to/private.env up -d --build
```
The private environment file sets `INTERDATA_UPLOAD_SECRET` and optionally
`INTERDATA_CDN_ORIGIN`, `INTERDATA_CAPACITY`, `INTERDATA_UPLOADS_PER_MINUTE`,
`INTERDATA_BIND_ADDR` (default `127.0.0.1`) and `INTERDATA_PORT` (default `3100`). The
container speaks plain HTTP on that port: put a TLS-terminating reverse proxy for the public
host in front of it and do not publish the port directly. Submissions live in the
`submissions` volume; back it up. The image runs as an unprivileged user on a read-only root
file system. Without compose: `docker build -f interdata/Dockerfile .` from the repository root.

The secret admits uploads and identifies nobody; who uploaded what is not recorded. One process
owns the data directory.

## API
All `/v1` requests need `Authorization: Bearer <secret>`. Uploads also need
`Content-Type: application/json` and an `Idempotency-Key` of 16–128 letters, digits, hyphens or
underscores (a UUID fits). Bodies are limited to 256 KiB. Unknown fields are rejected.

| Request | Result |
|---|---|
| `GET /health` | `200`, no secret needed. |
| `POST /v1/submissions/recipes` | `201` with `id`, `kind`, `status: "received"`, `received_at`. |
| `POST /v1/submissions/ingredients` | Same for an ingredient. |
| `GET /v1/submissions/{id}` | The same fields plus the uploaded `content`. |

Repeating an upload with the same key and body returns `200` and the first submission; the same
key with a different body returns `409`. Errors are `{"error": {"code", "message"}}` with a
stable `code`: `401 upload_secret_required`, `400 idempotency_key_required`, `415 json_required`,
`422` for content (`license_required`, `invalid_json`, `invalid_recipe`, `invalid_ingredients`, `invalid_amount`,
`invalid_steps`, `invalid_tags`, `invalid_source_url`, `invalid_image_links`,
`image_links_unavailable`, `invalid_ingredient`, `invalid_source`, `nutrients_need_source`,
`invalid_nutrients`), `429 too_many_uploads`, `507 buffer_full`.

### Recipe
```json
{
  "license": "CC0-1.0",
  "title": "Linsensuppe",
  "language": "de",
  "author": "optional",
  "source_url": "https://example.org/original-recipe",
  "servings": 4,
  "ingredients": [
    {"text": "250 g rote Linsen", "ingredient_id": "red-lentils", "amount": "250", "unit": "g"},
    {"text": "1 Prise Salz"}
  ],
  "steps": [{"text": "Alles 20 Minuten kochen.", "section": "optional"}],
  "tags": ["vegan", "fatty-liver"],
  "image_links": [{"url": "https://cdn.example/r/1.jpg", "alt": "Suppe im Teller"}]
}
```
`license`, `title` and 1–100 `ingredients` are required. `"license": "CC0-1.0"` is your
confirmation that you may dedicate the recipe to the public domain; for someone else's recipe,
write it in your own words and give its `source_url`. Each line needs its original `text` or an
`ingredient_id`. `amount` is an exact decimal **string** with at most six fractional digits;
JSON numbers, ranges and fractions are rejected, so keep those in `text`. `unit` is kept as
written; nothing is converted. Optional texts: `yield_text`, `prep_time`, `cook_time`,
`total_time`. Tags are lowercase ids; they are not yet compared with a vocabulary, and a
health-related tag is an unverified hint, not a medical statement.

### Ingredient
```json
{
  "name": "Red lentils",
  "language": "en",
  "aliases": ["Rote Linsen"],
  "form": "dried",
  "category": "legumes",
  "source": {
    "name": "…", "url": "https://…", "record_id": "…",
    "version": "…", "retrieved_on": "2026-11-28", "license": "…"
  },
  "nutrients_per_100g": {"protein": {"amount": "23.9", "unit": "g"}},
  "declared_allergens": [],
  "tags": ["vegan"],
  "note": "optional"
}
```
Only `name` is required. Nutrient readings need the `source` record they come from; without a
source the ingredient is accepted as an identity with unknown nutrition. An ingredient keeps
the licence of its source record; it is not relabelled as CC0.
