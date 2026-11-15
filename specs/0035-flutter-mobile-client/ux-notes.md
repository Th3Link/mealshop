# Smartphone interaction plan

Status: P1/P2 merged; P3 implemented for review in PR #48. The ZIP's UX is a target;
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

## P3 interaction contract

Keep the P2 Material theme/system typography. Today presents real preparation timestamps and ordered
meals; Plan uses a week strip, selected-day list and one complete meal editor. A meal action menu also
records actual output. All recipe parts remain in the existing always-open recipe views. Explicit
checkboxes mean per-person attendance or per-slot week confirmation; they do not claim allergy safety.
Unknown/ranged nutrition remains inspectable before apply. Existing/physical slots and infeasibility
are visible, never silently omitted or replaced. A stale review keeps the underlying editor draft.

Stock distinguishes physical ingredient lots, observed prepared portions and manual offers. Show actual,
held, used and available counts together; six/four/two is a count balance, never a mass conversion.
Every write has a concrete before/after review and explicit confirmation. Unknown network outcomes keep
the same stock request body/key; leaving warns about losing retry context. An unconfirmed draft can
explicitly compare a changed current stock revision before creating a new preview; cancel retains
its inputs. Ingredient/product/shop/source choices load the current directory, with an explicit clear
action for source selections that disappeared. No raw JSON editing.

Small screens/large text use wrapping controls, scrollable forms and selected-only bottom labels when
space is constrained; all destinations keep semantic names. Planning/stock refresh on foreground or
explicit pull without repeatedly downloading the whole catalogue in the background. Private activity
polling remains unchanged. Source guidance: [Flutter large-font design](https://docs.flutter.dev/ui/accessibility/ui-design-and-styling)
and [date-only Material picker semantics](https://api.flutter.dev/flutter/material/showDatePicker.html).

## P4 interaction contract

Keep the same five destinations and existing Material controls/system typography. Today and a recipe
lead into real cooking; More exposes Cooking & baking, Brewing and Cooking history. Sessions are
compact lists with visible active/paused/overdue counts and next action. A session shows readable,
always-open steps and their explicitly assigned ingredients/parts. Missing step assignment is labelled
as recipe reference, never invented. Timers have stable tabular text, named deadlines, independent
controls and explicit completion; elapsed passive time is not physical readiness.

Create/replan review shows canonical old/new local times, timezone/fold, dependencies, windows and
warnings. Forms keep drafts on conflicts. Optional details use plus actions; entered phases stay open.
Disconnected cooking remains readable with cache age. Saved timer intents remain visibly pending or
unknown until confirmed; conflicts require fresh explicit review. Uncertain requests keep their body
and key. Do not block the whole recipe behind network retry or turn a disconnected action green.

Local reminders and personal Nextcloud subscriptions are separate, default-off controls. Request OS
permission only from explicit enable. Show denied/unsupported/inexact/limited status and the last
authorization horizon. Notifications use generic private text and open the app; they never complete
work. Recipe/batch removal, replanning, opt-out and scope closure cancel obsolete local events.

Brewing uses a source-linked method picker, always-open phases, exact entered quantities/readings,
explicit unknown calibration and before/after batch review. Food history asks for actual attendees,
optional reported effort/ratings and a completion confirmation. Learning scope/reset and bulk deletion
remain separate owner-only actions. No new nutrition, readiness or device-control claims.

Every new screen has labelled 48dp controls, wrapping layouts at 320dp/200% and landscape, native back,
reader/offline/conflict/empty/error states, dark mode and semantic action names. Use a focused
`ChangeNotifier` for shared recovery state; UI `setState` only owns transient screen state. This applies
the verified UI-UX-Pro-Max Flutter state guidance without adding another state-management package.

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


## P4 cooking, brewing and feedback

Native Material surfaces retain the existing five destinations; Cooking & baking, Brewing and Cooking
history live in More and have contextual meal/recipe links. Steps stay open with 22sp readable
instructions, explicit ingredients/dependencies and independent absolute timers. Optional values use
plus actions; unknown durations/calibration/ratings remain visible unknowns. Day agenda labels each
process timezone and distinguishes planned start from actual deadline. Brewing uses three tabs with
stable-focus search and separate recipe/equipment/batch review, not fake food servings.

Local device reminders and personal server subscriptions are separate switches. Opt-in names Android
permission/inexact delivery and the seven-day horizon. Pending, uncertain, rejected and confirmed work
are distinct; review never silently changes a frozen request. Generic private notification text avoids
recipe/profile names. History explains household visibility before collecting attendees/ratings and
keeps learning/reset/bulk deletion owner-controlled. Large text uses wrapping controls and open steps.
