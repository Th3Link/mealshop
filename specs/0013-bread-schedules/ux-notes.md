# Native cooking and scheduling UX

Nextcloud components/tokens, existing typography and navigation remain authoritative. UX-UI-Max-Pro
guides progressive disclosure, recovery and focus; Taste guides alignment, rhythm and screenshot review.
Its marketing heroes, own palettes/fonts, global resets, shaders and navigation are explicitly excluded
by the customer hierarchy. Existing native recipe/planning screens are the visual reference.

Cooking has a household list and one focus view per pinned recipe instance. Ready actions and overdue
timers come first; other stages remain visible in dependency order. One action starts a named preset timer.
Pause/resume/complete are large named controls. A paused/overdue state is textual, not only color.
Countdowns do not announce every second; meaningful phase/status changes use a polite status region.

Planning starts from real recipe steps with explicit additional stages. Main fields are name, durations
and dependencies; equipment, repeated cycles, passive-time bounds and uncertainty expand when needed.
A target local date/time and IANA zone are visible together. Repeated civil times require a choice; gaps
require a corrected time. No duration or fermentation advice is invented.

Review shows old/new local times with offsets, dependencies, equipment, start windows, overlaps and
missed targets. Apply requires an explicit review; changing input invalidates the preview. Existing timers
continue while a draft is edited. Polling never replaces draft input or focus. Stale revisions preserve
input and offer reload/review. Cancelling a draft requires discard only when it changed.

Notifications default off. An explicit self-subscription explains Nextcloud acceptance versus actual
device delivery. Denied/unsupported browser permission, failed/unknown attempts, acknowledgements and
missed actions remain visible. Requests never silently retry an ambiguous notification outcome. Offline
state disables mutations while retaining elapsed deadlines. Empty/loading/error/read-only states explain
what to do next. Verify real embedded routes, keyboard, 320/390/768/1440, light/dark and axe.
