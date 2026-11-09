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

## Container artifact and pending deployment permission

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

Switching the supplied test instance to this container requires its internal service hostname
`nextcloud` to be added to Nextcloud's `trusted_domains`, preserving existing entries. Automatic
approval review rejected that specific persistent security-setting change. It has not been
performed or bypassed. The real Nextcloud verification currently uses the authorized manual
transport; container rollout into that instance awaits explicit customer approval. Backup the
Nextcloud registry/config and app data together before changing identities or resetting the instance.

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
