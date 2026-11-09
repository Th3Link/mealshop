# Native Nextcloud client

Mealshop has an opt-in `AppAPI` transport and Vue 3 client for shared households. The existing
loopback-only server/demo remains available. This guide describes the current test deployment;
it does not claim a published app-store release or a verified production `HaRP` installation.

## Verified target and client structure

Target installation: Nextcloud 35.0.2, `AppAPI` 35.0.0. Client: Vue 3, Composition API,
strict `TypeScript`, `@nextcloud/vue` 9.14.1, Vue Router 5.4 and Vite 7. The `AppAPI` loader
receives one IIFE (`js/mealshop.js`) and stylesheet (`css/mealshop.css`). It adds extensions to
registered paths, so registrations use `js/mealshop` and `css/mealshop`.

`NcContent`/`NcAppNavigation`/`NcAppContent` use the actual Nextcloud header, theme, navigation
and skip actions. Application CSS is scoped to `.mealshop`; no reset, web font or replacement
shell is installed. The router base and API proxy use `@nextcloud/router`, preserving webroot
and `index.php` behavior. No origin is embedded in product code.

## Application workflows

Create a household or accept an invitation addressed to your exact Nextcloud user ID.
Owners manage invitations and current roles; editors modify recipes, dietary profiles and
plans; readers can inspect, export and review without applying changes. Invitations explicitly
explain access to allergy information. Dietary profiles are people eating, distinct from the
Nextcloud accounts granted access. Use **Refresh access** after another person accepts; this
updates access without overwriting unsaved dietary edits. Non-owners can leave from the
household list; ownership must first be transferred to another accepted member.

Recipes support creation, immutable revisions, original-language content and exact serving
previews. Ingredient quantities accept g, kg, ml, l, tsp, tbsp and pieces. Non-mass inputs need
an explicitly selected exact-record reference portion or declared density/piece/spoon evidence.
The original amount, unit and evidence persist; no nearby food record supplies missing data.
The per-browser recipe display preference requests g/kg presentation from Rust. Kilograms may
need nine fractional digits to represent normalized micrograms exactly; this changes neither
stored input precision nor ingredient quantities. Repeated edits use expected revisions.
Conflicts retain the draft and require an explicit reload/review rather than silent overwrite.

Profiles separate structured hard allergen exclusions, free-text allergy/intolerance notes,
excluded ingredient IDs and soft dislikes. Export uses the current saved profile revision;
removal is confirmed and saved as a replacement snapshot. Unknown coverage stays unknown.

Planning uses pinned recipe revisions and titles, actual attendees, active/elapsed/advance
budgets, explicit calendar-snapshot consent and override, suggestions, review, shopping mass
deltas, apply and removal. Suggestions explain rejected constraints. Hard conflicts cannot be
acknowledged away. Reference batch times and uncertainty are visible before apply. No live
calendar provider or purchase workflow is implied by the snapshot and quantity preview.

The catalogue searches English/German names and aliases, retains exact nutrient strings,
distinguishes unknown values from measured zero, and exposes source/version/licence/uncertainty.
Source educational content stays original and marked as such when no reviewed translation exists.
UI language is a browser preference; invalid quantity text is never silently reinterpreted when
switching languages. No token, recipe or health profile is stored in browser storage.

## Trust boundaries

The transport validates `EX-APP-ID` and the base64 user/secret pair from `AUTHORIZATION-APP-API`
with constant-time secret comparison. Anonymous heartbeat is the only public infrastructure
endpoint. Empty service users cannot access household APIs. Current membership is checked for
every request before forwarding into a dedicated existing recipe/profile/plan store. Browser
Bearer credentials and cookies never select the household authority; the internal operator token
is supplied server-side only. All domain responses containing household data are `no-store`.

`AppAPI` 35's PHP proxy is explicitly `NoCSRFRequired`. Mealshop therefore requires
`X-Mealshop-Request: 1` on browser writes, in addition to authenticated identity and authorization,
and requires JSON content type for membership bodies. Cross-origin simple forms cannot set the
custom header; no CORS allowance or OPTIONS route is added. Nextcloud axios retains native
session/request-token behavior. The app does not relax the host CSP or log request credentials,
source recipe bodies or dietary profiles.

Membership uses a private, bounded, atomically replaced registry with one owner per household,
addressed invitations, an exclusive writer lock and fail-closed corrupt/pending-file handling.
The current principals are Nextcloud user IDs: revoke household access before deleting/reusing
an account ID. Automated account-deletion reconciliation and identity-backend migrations are
not implemented by this frontend slice. Do not treat this test deployment as production admission.

## Local development deployment

Build with supported Node 24 (minimum 24.15), npm 12.2.2 and Rust:

```sh
cd clients/nextcloud
npm ci --ignore-scripts --no-audit --no-fund
npm run build
cd ../..
cargo build --locked
```

Provide a private JSON environment file (mode `0600`) with string values for `APP_SECRET`
(32–200 ASCII letters/digits/underscore/hyphen), `APP_HOST`, `APP_PORT`, `NEXTCLOUD_URL`,
`APP_PERSISTENT_STORAGE` and absolute `MEALSHOP_ASSET_DIR`. Never commit it. Register a
manual-install daemon pointing at an address reachable by Nextcloud and start:

```sh
python3 scripts/run_nextcloud.py /absolute/path/to/private-runtime.json
```

Register `mealshop` with that daemon, the same private secret/port, and the `USER` routes in
`appinfo/info.xml`. The authenticated enable hook idempotently registers menu and assets.
No `/init` work is required: 404 means initialization is unnecessary. Startup restores the
actual enabled state from Nextcloud; disabling denies subsequent domain requests. SIGINT/SIGTERM
release storage locks cleanly. Do not remove a live writer's lock; follow the existing recipe
and household recovery guides after an interrupted write. Restart only one writer per directory.

On the supplied test machine, the private runtime files are under ignored `var/nextcloud/`;
the active app data is `/data/mealshop-nextcloud/mealshop`, separate from Nextcloud's `html`.
A pre-migration copy of the initial synthetic app data remains under ignored
`var/nextcloud/data`. The Nextcloud reset script has not been used.

## Container artifact and verified test deployment

```sh
docker build -t mealshop.local/mealshop:0.1.0 .
python3 scripts/test_exapp_container.py
```

The image pins base digests, builds Rust with 1.85 and the Vue IIFE with Node 24/npm 12.2.2,
and includes the manifest, assets, CA store and FRP entrypoint. Runtime uses a supported Alpine
3.24 release. The isolated smoke test exercises both TCP and Unix socket, lifecycle registration,
assets, authorization, CSRF and private storage as UID 1000 with a read-only root filesystem.
Its OCS mock is a transport fixture, not evidence of a real `HaRP` tunnel.

The copied Nextcloud reference entrypoint retains its copyright/licence. Local adaptations set
`umask 077` and put its FRP configuration in `/tmp/frpc.toml` for rootless/read-only operation.
`HP_SHARED_KEY` selects `/tmp/exapp.sock`; otherwise `APP_HOST`/`APP_PORT` select TCP.
The manifest uses a local-only image registry; nothing has been published to it.

`deploy/nextcloud/compose.yaml` can attach a separately managed test container to the existing
Nextcloud network, with the same secret and data, no added capabilities and an internal bridge
port binding. It requires private environment/settings files for the image, data owner UID/GID,
port, data path and external network. It does not modify the original Nextcloud Compose file.

The customer explicitly approved adding the internal service hostname `nextcloud` to the test
instance's `trusted_domains` on 2026-11-09. It was added at index 3, preserving existing entries.
The manual process was gracefully stopped before starting `mealshop-exapp-test-mealshop-1`
with the same private secret, port and `/data/mealshop-nextcloud/mealshop` directory. The original
Nextcloud Compose configuration was not modified. The image runs as the existing data owner,
with a read-only root filesystem, no additional capabilities and `no-new-privileges`.

Manage this deployment from the repository root using the private configuration:

```sh
docker compose --env-file var/nextcloud/compose.env -f deploy/nextcloud/compose.yaml ps
docker compose --env-file var/nextcloud/compose.env -f deploy/nextcloud/compose.yaml restart mealshop
```

The manual startup helper remains available for development, but must not run concurrently
against the same directory. Back up the Nextcloud registry/config and app data together before
changing identities or resetting the instance. See F22 validation for restart and browser evidence.

## Repeatable checks

```sh
cd clients/nextcloud
npm run format:check
npm test
npm run build
npm audit --audit-level=moderate
MEALSHOP_TEST_URL=http://127.0.0.1:8080 \
  MEALSHOP_TEST_USER=your-test-owner MEALSHOP_TEST_PASSWORD=your-test-password \
  MEALSHOP_READER_USER=your-test-reader MEALSHOP_READER_PASSWORD=your-reader-password \
  npm run test:e2e
```

The browser suite creates fictional household/recipe/profile data. Use an explicitly disposable
instance and two prepared accounts; do not point it at personal household data. It verifies the
actual host, read/write boundaries, revocation, conflicts, exact quantities, hard constraints,
source portions, deep-link refresh, responsive layout, native light/dark, keyboard and axe checks.
Screenshots/traces remain local in ignored test-result directories. Formal accessibility
certification, a live calendar connector, standalone shell and Flutter are separate work.

## Daily workflow extension (F24)

The navigation contains a searchable household switcher. App entry restores the last accessible
household ID for the current Nextcloud account/browser; explicit household management remains
available. Both pretty embedded URLs and their `index.php` aliases retain deep links. Search-query
updates do not run page focus management. Membership is always rechecked on the server.

Recipe authors are optional. Display quantities trim redundant fractional zeros without changing
saved input, exact scaling or nutrient precision. Ingredient rows use searchable native selects,
with household favourites first, inline declaration creation and trailing labelled trash actions.
Optional JPEG/PNG/WebP images are uploaded through protected household routes and referenced by
immutable asset ID. Removing the current image preserves historical revisions. Images are limited
to 5 MiB, 8192 pixels per dimension and 32 million pixels, with 100 MiB/1000 assets per household.
Active image formats and remote image fetching are not supported. Back up the `images` directory
with recipes, profiles, plans, membership and the ingredient overlay. A leftover `.image.pending`
requires inspecting/recovering that write before restarting; never remove a live writer's lock.

The weekly calendar has breakfast, lunch, snack and dinner slots. Selecting a slot opens the
existing review/apply workflow with that date and meal. Each person can attend or skip independently;
servings remain explicit. Empty slots represent no planned meal. Week navigation survives opening
and cancelling an editor, and applying a future meal returns to its week. Existing custom slot
identifiers remain visible/editable rather than being silently discarded. Skipped-person details
can be expanded without stretching every calendar row with long names.

The invitation picker calls the installed Nextcloud file-sharing sharee search using the current
browser session, matching file-sharing account visibility. It selects local people, not groups,
federated addresses or guessed accounts. Household acceptance and roles remain unchanged.

### Household ingredient API

Under each authorized household domain prefix (or the protected local family API):

- `GET /api/household/ingredients`: merged catalogue, nutrient definitions and overlay `revision`.
- `POST /api/household/ingredients`: `{ expected_revision, ingredient: { name, language, form,
  recorded_on, source_url?, annotations? } }`; returns stable `id` and new revision.
- `PUT /api/household/ingredients/{id}`: `{ expected_revision, annotations }`, where annotations
  contain `tags`, `components`, `declared_allergens`, `favorite` and `archived`.
- `DELETE /api/household/ingredients/{id}`: `{ expected_revision }`; archives without deleting history.
- `GET /api/household/ingredients/{id}/recipes`: latest saved recipes that include this ingredient,
  directly or through their pinned composition.
- `POST /api/household/images`: bounded binary body with the actual raster MIME type; returns `id`.
- `GET /api/household/images/{id}`: authorized raster bytes, fixed MIME type, `nosniff`, `no-store`.
  Recipe create/update accepts optional `image_id`, referencing an existing household asset.

All shared mutations require writer permission and the existing browser-write marker. Reader
accounts can inspect the merged catalogue and reverse lookup but cannot mutate annotations or
assets. Favourites are explicitly shared household annotations. Reference source records remain
immutable; household annotations never rewrite a primary nutrient record. Archived ingredients
are hidden from new selection and cannot be newly introduced through the recipe API, while
existing recipes remain readable/editable with their pinned identity.

Declarations use `source.kind = household_declaration`; a missing source URL is explicit and no
nutrient value is fabricated. German declarations preserve their entered name/form in German
translation fields with a neutral English canonical identity. Composition records ingredient
membership, not inferred mass proportions. Cycles, unknown references and excessive depth fail.
User-declared allergens are distinct from primary evidence. Profiles additionally accept
`excluded_tags`; exact case-insensitive matches explicitly block meals, including tags/allergens
on nested or pinned ingredients. Free-text notes and unmatched descriptive tags never establish
safety. Current and recipe-pinned positive constraints are both retained during review.

The overlay uses `ingredients.json` and `.ingredients.pending` under the existing private household
lock, with expected revisions and atomic replacement. Conflicts retain the ingredient editor;
its explicit reload action refreshes the version for review instead of silently overwriting it.
Plan checks and durable application hold the same ingredient guard, preventing concurrent tags
from being skipped between review and save. A separate dietary revision avoids invalidating
plans when only a favourite/archive annotation changes; older overlays retain their global
revision as a conservative fallback.
