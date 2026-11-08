# ADR 0011: protected family meal plans and preview/apply

Status: Implementation for customer-requested F10; review pending

## Context
F03/F04/F09 are accepted. Family planning needs pinned recipe inputs, current household
constraints, explicit attendance and time budgets, and inspectable changes before persistence.
No calendar provider, external identity service or shopping-list provider has been selected.

## Decision
Add protected planning routes to the household access boundary. Recipes and planning share one
locked recipe store, so plans refer to actual immutable local recipe IDs/revisions. Household
profiles and plans use separate revisioned snapshots in the same exclusively locked household
directory. All I/O-heavy planning work runs on blocking workers with a consistent lock order:
recipes, profiles, plans. A preview returns actual profile/plan revisions; apply recomputes the
same constraints while holding those locks, requires the matching revisions and explicit review
acknowledgement, and never permits a known hard dietary or time conflict. Readers can inspect,
preview and request suggestions; only operators can apply/remove. No new dependency is needed.

Plans identify a Gregorian date and named meal slot, pinned recipe revision, selected attendees,
target servings and active/elapsed/advance budgets. Duration estimates conservatively sum
ordered step active/passive seconds (no inferred overlap). Missing steps mean timing unknown.
Advance lead is elapsed duration beyond the meal-day window. Times refer to the original
recipe yield and are not silently extrapolated for scaled batches; this uncertainty is shown
in the mandatory preview. Nutrition retains F03 rational scaling. Shopping requirements derive
from the saved plan's pinned recipes; preview computes the signed mass delta for a slot before
replacement/removal. This is planning-derived shopping demand, not completion of F06's shop,
extra-item, purchase or inventory workflow.

Calendar access is an explicit read-only attendance snapshot import: source, event ID, date
and already mapped household member IDs. User consent is required; the source/proposal remains
visible. Manual attendance can override it without changing any external calendar. Snapshots
can be supplied through the API or the demo's bounded JSON file import. No live OAuth provider,
ICS recurrence/timezone parser, calendar polling or external write is claimed. Provider adapters
can later supply this same bounded contract; the current import is an actual local input path.

Suggestions evaluate up to 50 explicitly selected candidate revisions, reject known conflicts
with reason codes and rank remaining candidates by soft dislikes. Unblocked suggestions remain
review-required under F09. Plans retain the profile revision and allergen reference version
reviewed when applied. Later profile/reference changes mark slots for renewed review; they do
not rewrite historical choices or approve them automatically.

## Alternatives
Applying arbitrary client-authored recipes would bypass pinned source/history guarantees.
Automatic application or using soft preferences to bypass exclusions is incompatible with F09.
A provider-specific calendar integration without a chosen provider would introduce unnecessary
credentials/dependencies. Mutating an external shopping list would exceed the current F06 scope.

## Consequences
One household/operator boundary remains local and loopback-only. Plan snapshots are bounded to
100 slots/1 MiB and use synced private pending-file publication with fail-closed recovery.
API export and backups may contain attendance/calendar identifiers. Profile deletion leaves
historical plan member IDs; operators can explicitly remove those meal slots. Source data and
cooking-time estimates are not food-safety guarantees. See the meal-planning operator guide.
