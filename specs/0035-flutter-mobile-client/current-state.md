# Repository audit for the mobile pack

Audit date: 2026-11-13. Baseline: `fe6dd078d8db77b6ae4e55a48f78310d53487d2a` (`main`, merged PR #37).
This is source/contract evidence, not a new mobile runtime test. Existing acceptance evidence remains
in the linked feature folders. No retained household records were inspected for this planning task.

| Capability from the pack | Verified server/web state and source evidence | Mobile gap / delivery |
| --- | --- | --- |
| Flutter app / build | Absent: `clients/` contains the Nextcloud Vue client; no `pubspec.yaml` or mobile platform project. `flutter`, `dart`, `adb`, `sdkmanager` and `xcodebuild` are not on this host's PATH; Java alone is available. | P1 creates `clients/flutter/`, toolchain/CI and an installable phone app. No Flutter command has been run here. |
| Identity / households / invitations | Nextcloud transport and current owner/editor/reader authority in `src/exapp/mod.rs`, `src/membership.rs`, `src/exapp/sharing/`; F18/F22 evidence. | P1 native login/credential lifecycle and actual proxy-write probe; device pairing is not implemented today. |
| People / dietary rules | Partial relative to the new pack: `src/household.rs::Member` has names, allergy/intolerance notes, allergen IDs, excluded ingredients/tags and dislikes. `evaluate` enforces current hard rules. | P2 adds reviewed presets/exceptions, origin conditions and optional goal/window/prep records. Diagnosis fields, person-level health access and a progressive mobile tour are absent. |
| Meal plans | `src/planning.rs` has pinned meals, attendees/calendar snapshots, time budgets, slot suggestions and reviewed preview/apply; F10/F11/F31. | P3 native day/week editing and a server-owned whole-week proposal. Slot ranking is not proof of a complete multi-objective weekly optimizer or recurring attendance/window policy. |
| Shopping and product identities | `src/shopping/{api,model,sources,products,shops}.rs` and `Shopping*.vue`; manual/recipe/plan contributions, check-off/undo, shops, consented learning, manual GTIN and lookup already exist (F06/F07/F08/F32). | P1 complete native daily list and foreground sync; P5 phone camera entry. Distinct ingredient/product/request/purchase identities must remain. |
| Offline writes / sync | Missing as a mobile protocol. Shopping `Mutation` contains only `expected_revision` and `command`; anonymous/manual source creation allocates a new ID. Purchase records are not durable client-request receipts. Current revisions protect concurrent writes. | P1 must add bounded atomic request receipts and explicit conflict policy before automatic offline retries. A fresh retry after an unknown response can otherwise duplicate intent. |
| Recipes / ingredients / nutrition | `src/recipes.rs`, `src/recipe_api.rs`, `src/recipe_structure.rs`, `src/ingredient_api.rs`, `src/ingredients.rs`, catalogue/quantity/evidence modules; F02–F04/F23/F25–F27/F31/F33/F34. | P2 native search, manual authoring, parts/dependencies, groups/subtypes, optional ingredient creation, source ranges and recipe images. Reuse exact canonical records; do not rebuild calculations in Dart. |
| Sharing / collections / imported rights | `src/exapp/{library,sharing}`, `src/library`; F05 pins revisions, checks recipient membership and projects imported source URLs only. | P2 native shared content/invitations and origin-aware local projection/export. Revocation cannot retract copies already viewed. |
| Mealprep / inventory / offers | Planned production/allocations in `src/planning.rs`; exact lots, reservations and explicit stock transactions in `src/inventory/` with F15/F31 evidence. | P3 complete mobile stock/offer workflows and the pack's physical prepared-batch/storage lifecycle. Planned mealprep is not finished physical inventory or automatic safe-expiry guidance. |
| Cooking / timers / brewing | `src/cooking/`, `src/brewing/`, F12/F13/F14 and `Brew*.vue`; durable deadlines, calendars, source snapshots, explicit phase completion, equipment/readings. | P4 native cooking, cache/reconciliation and permission-aware local reminders; Nextcloud notification acceptance does not deliver a Flutter push notification. Cooking command retries need their own audited idempotency boundary. |
| History / learned preferences | `src/feedback/`, F11; actual history, explicit learning epochs/consent and explained suggestions. | P4 native history/feedback/consent/reset; no claim of automatic clinical personalization. |
| URL / OCR / video capture | `src/imports/` and F16 implement supported public URL/manual reviewed import with network/source safeguards. | P5 share-sheet integration and camera/OCR review. No mobile OCR provider, video downloader or smart-scale driver exists. |
| Activity | No dedicated activity model/API/UI in the audited routes/models. Cooking effort/history is a different concept. | P2 optional type/duration/intensity log with explicit privacy and no fabricated kcal. |
| Localization | Nextcloud EN/DE and exact units exist; F29 retains unfinished cross-client/versioned translations. | P1 mobile UI/exact input; extend coverage per delivery, keep source text and language-independent IDs. |
| Standalone / hosting / other stores | Local file-backed family APIs and demo exist. Production standalone identity, hosted client admission and the SQL/Git adapter matrix are not delivered (F18/F19/F30). | Preserve future transport seams without adding unimplemented auth modes, all adapters or hosted commerce to P1. |

## Actual transport and API map

For Nextcloud, preserve the installation prefix and use its authenticated `/apps/app_api/proxy/mealshop`
path, as `clients/nextcloud/src/api.ts` and `scripts/mcp_stdio.py` already do. The domain prefix is
`/api/households/{home}/domain`. The native app must never send the `ExApp` secret/operator token.
The MCP bridge demonstrates Nextcloud app-password authentication for its own route; it does not
prove the new mobile login flow or every shopping mutation. P1 must exercise those on Nextcloud 35.

| Actual path after proxy prefix | Current use / source |
| --- | --- |
| `/api/households`, `/api/households/{id}` and its invitation routes | Household CRUD subset/current membership; `src/exapp/mod.rs` |
| `/api/households/{home}/collections`, `/api/shared` and sharing routes | Library and permissions; `src/exapp/{library.rs,sharing/}` |
| `/api/ingredients`, `{domain}/api/household/ingredients` | Reference/own ingredient discovery; `src/api.rs`, `src/ingredient_api.rs` |
| `{domain}/api/recipes`, `{domain}/api/recipes/{id}` and `/scale` | List/create/read/revise/scale; `src/recipe_api.rs` |
| `{domain}/api/household/profiles`, `/allergens`, `/evaluate` | Profiles and dietary review; `src/household.rs` |
| `{domain}/api/household/mealplan`, `/preview`, `/apply`, `/suggestions` | Meals and reviewed changes; `src/planning.rs` |
| `{domain}/api/household/shopping`, `/lookup` | Current list mutations and consented lookup; `src/shopping/api.rs` |
| `{domain}/api/household/inventory`, `/preview` | Lots/offers/stock transactions; `src/inventory/api.rs` |
| `{domain}/api/household/cooking`, `/template`, `/preview`, `/{id}` | Durable cooking workflow; `src/cooking/api.rs` |
| `{domain}/api/household/brewing` and recipe/batch/readings/template routes | Beverage workflows; `src/brewing/` |
| `{domain}/api/household/history`, `/learning` | Feedback and consent; `src/feedback/api.rs` |
| `{domain}/api/household/imports/preview`, `/imports` | Reviewed URL/manual import; `src/imports/api.rs` |

Paths above describe existing route declarations, not a newly standardized mobile sync API.
Version/capability discovery, shopping replay receipts, expanded profiles, full-week proposals,
physical prepared batches and activity contracts are **proposed additions** in the corresponding PRs.
Define them with negative tests and migrations before use; do not fabricate current endpoints.
