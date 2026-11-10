# Approved native interaction contract

Nextcloud owns shell, typography, theme, labels and focus. Keep its existing left navigation. The meal plan is the only calendar. Show preparation in the same days as meal allocations. A native detail panel edits production and eating dates. Adding another eating day uses the same entry, without duplicating shopping.

Each recipe part expands locally to ingredient rows and its instructions. Prepared parts are selected by stable reference. Additional ingredients remain separate. Temperatures belong to ingredient rows under an optional extension action: type Temperature, exact value, Celsius. Unknown extensions remain absent, never inferred from text. Show the stored value in recipe reading and cooking context; preserve it through edits and quantity scaling.

One completion action starts the waiting timer at the actual time. Planned notifications only open the relevant task. Unknown durations stay unknown. Range handling and overlap conflicts must be explicit, not silently flattened.

## Approved visual references
The German PNGs in screens/ are customer-approved design evidence. They are intentionally retained in their reviewed language; canonical specification and coded keys remain English. They are drawings, not screenshots of completed implementation. General and multi-part examples both apply. Responsive implementation must stack detail forms, retain readable ingredient names and support keyboard alternatives to dragging.

## Implementation choices for review
Existing time budgets remain available in one collapsed “Limit available time” section. New native plans use the API's supported upper bounds (24 h active, 24 h elapsed plus 30 days advance); a bread recipe is not rejected by an invented default one-hour budget. These are validation limits, not promised user availability or duration estimates. Existing explicit budgets remain intact. A later timing contract may distinguish unset budgets structurally.

Existing entries keep their production identity hidden while editing eating dates through allocations. Removing the final allocation persists an explicit empty list, rather than silently converting it into a full-quantity meal. The shared calendar still shows the unassigned production. Dietary review remains conservative across all allocated days.

A calendar anchor is a planned reminder, not a lock on actual work. The user may complete work earlier; explicit reviewed delays and unfinished prerequisites remain separate constraints.
