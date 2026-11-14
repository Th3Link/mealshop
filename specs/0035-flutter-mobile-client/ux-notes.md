# Smartphone interaction plan

Status: P1 merged; P2 direction accepted with customer activity-access decisions. The ZIP's UX is a target;
this document reconciles it with existing data and the five complete deliveries.

## Navigation and the first useful launch

Target bottom destinations: Today / Plan / Shopping / Recipes / More. P1 starts with Shopping/More,
P2 adds Recipes, P3 activates Today/Plan. Preserve selected household and destination safely across
relaunch; never mix caches across accounts. No disabled “coming soon” tabs or decorative empty screens.

Use adaptive Flutter controls and platform back/navigation conventions, accessible touch targets,
text scaling, screen-reader semantics, high-contrast light/dark surfaces and reduced-motion behaviour.
Use a small coherent token set; the phone app is not the Vue screen embedded in a `WebView`. Keep the
existing Nextcloud client's native components/styles unchanged except where contract compatibility
requires changes. Every shipped action has loading, empty, error, pending/offline and conflict states.

## Daily shopping first (P1)

Open the last authorized household; a visible switcher allows changing it. Fast text add with clear
quantity/unit input and optional ingredient/product selection; large check targets, separate purchased
section and reachable undo. Details retain exact manual/recipe/plan contributions and provenance.

Display the last server sync time and pending count. Optimistic check-off has a visible pending state;
“purchased” is confirmed only after the canonical server response/receipt. A failed or conflicted write
keeps the entry/draft and explains reconciliation. Airplane-mode process restart retains the outbox.
Do not turn a check of an old amount into a purchase of newly added unseen requirements.

## People and content (P2)

The tour asks one or two questions per step, saves a protected local draft, supports optional skip and
returns to an editable derived-rule summary. Hard exclusions, soft dislikes and source uncertainty use
text/icons, not colour alone. Presets with exceptions display their actual meaning. An own-egg allowance
cannot waive an unrelated allergy or treat unknown provenance as known.

Keep sensitive medical context optional and hidden until its access/retention contract is approved.
Optional profile avatars never replace accessible person names. Solo users need no extra account.
No required diagnosis/weight/calorie target. Activity entry remains a small optional personal log.

Recipes have compact always-open ingredient/part/step composition, inline ingredient creation, explicit
homemade outputs and retained exact original quantities. Source/ranged/unknown nutrition is visible.
Incoming shared content follows server projections; imported material offers its source link only.

## Plan, prepare and cook (P3–P4)

Today shows real chronological meals, participants and next preparation/cooking action. Plan uses a
week strip plus a selected-day list for breakfast/lunch/snack/dinner; per-person skips and portion edits
are explicit. Review shopping deltas and infeasibility before committing changes. Batch allocation
explains actual produced/available/allocated/remaining quantities, not just planned meals.

Cooking uses large steps with their linked ingredients, persistent independent timers and meaningful
pause/complete/reschedule actions. Offline cached content identifies its freshness. Canonical server
state, pending local actions and local reminder delivery remain distinct. Notification denial leaves a
usable workflow and an explanation of actual limitations. Brewing keeps explicit equipment completion.

## Capture and locale (P5; locale from P1)

Camera/share-sheet content enters review before save. Retain original source/candidates next to reviewed
fields; highlight unknown units/ingredient mapping and allow manual fallback. Permission cancellation
returns to the prior context with its draft intact. No automatic restricted-video retrieval.

English is the code/canonical base; DE/EN UI ships in P1, with device-language normalization, a clear
manual choice and documented fallback. Search includes localized aliases. Switching locale must not
reinterpret in-progress numbers; decimal strings and original unit/source precision remain exact.
Respect safe areas, keyboard insets and one-handed use without hiding required review actions.

## Phone acceptance per delivery

Each release includes synthetic normal/error/offline/conflict screens, smallest supported width and
large-font checks, screen-reader/keyboard focus where applicable, reader-role screenshots and actual
phone OS/build evidence. Exercise interrupted drafts, account switches and denied permissions. Mocked
widgets and screenshots supplement physical-device journeys; they do not prove device installation,
camera access, background notifications or offline replay by themselves.

Quality targets: at least 48×48 logical-pixel touch areas, 4.5:1 normal text/control contrast,
320-logical-pixel layouts with 200% text scaling, and actual `TalkBack`/`VoiceOver` checks for the
chosen platforms. Validate these alongside conflict/permission states, not just happy-path screenshots.
[Flutter accessibility guidance](https://docs.flutter.dev/ui/accessibility) supports this proposed bar;
no mobile conformance is claimed by this plan.

## P2 screen contract

Keep the established Material theme, system typography and 4/8dp spacing. UI-UX-Pro-Max contributes
small-step forms, escape/back paths, native controls and 48dp targets; Taste contributes consistent
edges, hierarchy and actual screenshot review. Website heroes, palettes/fonts, effects and CSS gates
are inapplicable to this native app and do not override the existing product system.

Recipes opens a searchable compact library with explicit collection/shared destinations and one
create action. Detail keeps ingredients, named parts and steps open; scaling is a preview of the
selected revision. Authoring edits the existing domain fields with inline ingredient creation,
button alternatives to reorder, optional plus actions and visible source/nutrition uncertainty.
Ingredient forms group identity, classification and explicit nutrient/constituent declarations;
blank values remain unknown. Complex evidence stays editable through labelled fields, not raw JSON.

More exposes People and Activities. The person tour asks name/identity, diet/exceptions, hard
constraints, preferences/availability and review in small steps, with protected resume/discard.
People are never silently treated as login accounts. Activity screens state the selected person and
current access. Owners without a grant see Manage access; Grant myself access is an explicit action.
Permission forms distinguish read and edit, with visible current recipients and a revocation action.
Activity records are online-only; errors retain a draft, no kcal is calculated, and exports require
current per-person access. A household/account switch disposes every open sensitive route.
