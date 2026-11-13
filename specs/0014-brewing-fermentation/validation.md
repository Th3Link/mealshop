# F14 acceptance evidence

Date: 2026-11-13. Functional implementation: `0a9a270`, with final scoped CSS refinement, on
`story/brewing-and-fermentation`, PR #37
against `main` (`fe642d9`). Only explicitly recorded synthetic fixtures were used. No retained household
directory was inspected, copied, hashed or reset during this feature's validation.

## Contract coverage

| Criterion | Delivered evidence |
| --- | --- |
| AC 1: distinct brewing methods and equipment | Twelve primary-source-linked beer templates preserve extract/steeping/partial-mash/all-grain distinctions, BIAB and named/DIY equipment. Exact liquid/ingredient quantities, declared phase temperatures/volumes, editable expanded phases and immutable recipe revisions are covered by catalogue, HTTP and native authoring tests. |
| AC 2: pre-brew and phase reminders | Recipe-specific calendar-day preparation, DST's 23-hour interval, phase deadlines, explicit manual completion and the existing private opt-in delivery worker are tested. A 40-day active fermentation deadline survives an installed-container restart. |
| AC 3: original measurements | Raw six-place SG/Brix/temperature/volume strings, original unit, measured civil/absolute time, authenticated entry actor/time, manual/transcribed/reported attribution, known/unknown calibration and reasoned append-only corrections survive replay/restart. No conversion or configured live device connector is inferred. |
| AC 4: independent batches and reservations | Atomic reviewed reservations, exact phase-volume capacity checks, frozen equipment snapshots, concurrent equipment edits, long-running/paused physical holds and cross-beverage resource conflicts are tested. Explicit completion releases the resource; an elapsed observation timer does not. |
| AC 5: independent fermentation validation | [Primary references](research.md#independent-validation-of-cider-wine-and-mead) independently distinguish prepared-juice cider, white grape wine, red wine fermented on skins and traditional no-boil mead. Their typed processes omit beer-method fields. Incorrect beverage/method/template combinations and grain-mash phases are rejected; all four workflows pass real native authoring, batch and original-reading checks. Source review is not recipe certification or a safety guarantee. |

## Local quality gate

All commands below passed on the final functional implementation: **181 Rust tests**, **36 native client
contracts**, strict TypeScript/Vite, strict rustdoc, and unchanged dependency manifests/lockfiles.
The mutation experiment inverted the beverage/process guard, observed the relevant integration test fail,
restored the guard and then ran the complete passing gate.

```sh
python3 scripts/generate_docs.py --check
python3 scripts/import_catalogue.py --check
python3 scripts/verify_catalogue.py
python3 scripts/import_nutrition_estimates.py --check
python3 scripts/verify_nutrition_estimates.py
python3 scripts/verify_conversions.py
python3 scripts/verify_allergens.py
node --check src/web/demo.mjs
node --test tests/i18n.test.mjs
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo deny check licenses
cargo audit
npm --prefix clients/nextcloud run format:check
npm --prefix clients/nextcloud test
npm --prefix clients/nextcloud run build
npm --prefix clients/nextcloud audit --audit-level=moderate
node scripts/generate_mcp_schema.mjs --check
python3 -m unittest discover -s tests -p test_mcp_stdio.py
cargo build --locked
node scripts/test_mcp_client.mjs
cargo doc --workspace --no-deps --locked
```

## Real installation and browser checks

Nextcloud **35.0.2**, `AppAPI` **35.0.0**, Chromium, existing embedded and household proxy paths.
Installed implementation image:
`sha256:d7437c3e86c7ee93a92701ea6e30219424c8fd397c0903e5193aa2975242bd75`.
Authenticated public asset responses match the container hashes: JS **1,660,580 bytes**, CSS
**174,759 bytes**. The production build reports JS **498.94 kB gzip**, CSS **25.00 kB gzip**;
these are total artifact sizes, not a runtime performance benchmark. No dependencies were added.

Rootless, read-only, dropped-capability container acceptance passes over **TCP and Unix sockets** with
lifecycle/auth/CSRF, existing food workflows, imports/sharing/MCP/inventory and the new source-linked
brew/fermentation workflows. Fresh and replayed mutations return the exact receipt result ID.

The actual installed container was restarted with five independently active beverage workflows on
image `79d72ff…`; the final CSS-only image has byte-identical server and JavaScript artifacts. Original
40-day deadlines, exact liquid quantity/capacity, reported device calibration, original/corrected beer
readings, four original Brix readings and old-revision receipts/result IDs survived without duplication.
A conflicting future equipment reservation remains rejected. The separate food reference and three
food servings remain unchanged. The synthetic household's persisted file is mode `0600`.

Browser runs use one worker and ledger-owned homes. No tests are skipped. Batches are separated by
owned-fixture cleanup to respect the household limit. Initial findings and their targeted reruns are
retained rather than claiming an uninterrupted all-green invocation:

| Run | Image / scope | Result |
| --- | --- | --- |
| A | Earlier complete implementation `2ae33b9…`: authoring, brewing, catalogue, compact recipes, cooking, daily workflows, ingredient groups, Nextcloud, nutrition, theme, preparation and usability | 19/21 passed. The new duplicate-title case and a cooking theme-transition audit failed; both were addressed below. |
| Targeted repeat | Installed `79d72ff…`: all five brewing cases plus the complete cooking case | **6/6 passed**. All four drink templates deliberately reuse the same recipe title; each correct immutable recipe, phase count, batch type and original Brix reading is verified after reload. |
| B | Installed `79d72ff…`: feedback/history, MCP, sharing and shopping | Eight cases passed before the browser process received SIGTERM; the four remaining cases passed separately on CSS image `10d8512…`: **12/12 distinct cases passed**. |
| C | CSS image `10d8512…`: inventory and recipe import | **8/8 passed**. |
| Final visual repeat | Final stacked CSS image `d7437c3…`: complete native authoring, empty states and four fermentation templates | **3/3 passed**, including text bounds checks in all 48 scene/language/theme/width combinations. |

The duplicate-title finding was a product defect: selecting a saved recipe by title could open an older
same-named recipe. Both fresh and replayed API responses now return their durable `result_id`; recipe
and batch navigation use that exact ID. The cooking audit was sampling a native theme transition;
it now waits for font/layout frames and finite animations, retaining all contrast rules. The existing
inventory purchase test waits for the canonical purchase acknowledgement before navigation, rather
than an optimistic checkbox update. Manual screenshot review also found long German fact labels overlapping values at 320 px. Scoped
normal wrapping, stacked mobile facts and native tab spacing fix this; the visual audit now checks text bounds as well as
application overflow and axe. Credential-bearing network tracing is disabled; only synthetic
screenshots/error contexts are retained locally.

All F14 recipe, batch, empty-equipment and red-wine views pass axe's WCAG 2/2.1/2.2 A/AA rules in
**EN/DE × light/dark × 1440/390/320 px**, with no application horizontal overflow. Synthetic screenshots
were inspected at desktop and narrow dark layouts. Automated checks support the accessibility evidence;
they do not establish full manual WCAG conformance. Keyboard, owner/reader, offline, stale-write,
uncertain-save and empty-state flows are exercised through the native UI.

## Final publication and cleanup

All **41 distinct browser scenarios** passed across the recorded runs, with targeted repeats after each
finding. The final scoped CSS has an additional passing three-case visual repeat. Final fixture cleanup
left **zero active ledger-owned households**, preserving every non-ledger household and allocation counter;
retained household directories were not inspected. The exact final image remains healthy.

The final change after the functional commit contains only the tested scoped CSS, its text-bounds audit,
and documentation/generated rustdoc metadata. All mandatory gates are rerun before push. All eleven CI
checks must pass on the final PR head before review handoff; the PR's live checks are authoritative.
The installed image and artifact identities remain explicit above rather than implying every earlier
browser invocation used the same image.
