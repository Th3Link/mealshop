# F32 acceptance evidence

Status: Done — accepted native Nextcloud scope.

Target: embedded Nextcloud 35.0.2 / `AppAPI` 35.0.0 and `@nextcloud/vue` 9.14.1. The Bring reference
informs discovery and compact tiles; host colors, fonts, icons, navigation and controls remain native.

| AC | Observable verification |
|---|---|
| 1 | Named native checkbox tiles retain exact quantities, decorative icons from explicit metadata, text/check state and purchased goods. Pointer/keyboard checks, unchecking and provenance are covered by visual/shopping browser tests. |
| 2 | Search retains focus and finds canonical/translated names. Enter pre-fills only a label. Explicit `0.100001 kg` persists as `100.001 g`, retaining original quantity in details. Saving restores search focus; no ingredient identity, quantity or recent-use history is inferred. |
| 3 | Confirmed checks expose Undo and restore focus. An injected 503 retains confirmed unchecked state; retry reloads it. A delayed successful write disables controls without moving the tile. Reload clears stale Undo context. |
| 4 | Existing tests retain contributions, purchased receipts, recipe rescaling/plan sync, shops/fallback, keyboard ordering and consented history. Search disables ordering instead of replacing a complete shop order with a subset. Barcode/manual-entry/camera-stop fallbacks remain covered. |
| 5 | Real isolated Chromium uses the Nextcloud shell/proxy and ledger-recorded synthetic households. Light/dark, reduced motion, keyboard/focus, reader restrictions and 320/390/768/1440 widths are covered. Tested states have no axe violations for WCAG 2/2.1/2.2 AA tags. F31 mealprep remains covered. |

## Visual and behavior review
Screenshots exposed unequal native checkbox widths, symbol/check overlap and bad long-name clipping.
Tiles now fill their cells, reserve the checkbox corner and wrap names to two lines. Full names stay
accessible and available in details; quantities remain complete. Mobile uses three columns at
360–600px and two below that. No custom palette, fonts, reset or new framework was introduced.

The delayed-save regression initially measured a 28px tile jump when the page spinner appeared.
Initial loading retains that spinner; list mutations use the existing status region and native busy
controls. No optimistic purchase acknowledgement is used. The installed-container full suite passed
19 tests; after the final label-only refinement, all four existing shopping contracts and the
visual/localization test passed. German status labels additionally remain on one line at the narrowest
three-column width, 360px.
Background tiles remain inert through the native dialog's complete close lifecycle. The shopping
workflow immediately reopens item details after shop assignment/filtering, guarding against a new
open request being lost during the close transition. Learning consent waits for the actual enabled
checkbox and confirms its saved checked state before starting a session.

## Checks and measured cost
Repository/client gates passed: 88 Rust tests, 16 client tests, strict types/build/format, licences and
both dependency audits. Unit tests cover exact zero versus a remaining microgram, translated all-word
matching, stable identity/order and neutral unlinked icons. Temporarily inverting the zero predicate
makes the new test fail; the original was restored before verification.

Locked JavaScript is 1,367,492 bytes versus 1,354,731 in the original PR #27 image. The same Python
gzip level 9 with zero mtime gives 419,997 versus 415,154 bytes (+4,843 bytes). This measures bundle
size, not page-load improvement. No dependency, crate, endpoint or persisted model was added.
Search and ingredient lookup are computed from existing data.

## Operations and limits
Complete private backups precede installation and fixture archival. Customer households are excluded
using recorded identity and content checks. A real restart retained API snapshots, deadlines/timers and 35 files (11 memberships, one image).
Verified fixture archival preserved both customer membership rows and all eight customer files against
the initial backup. Final image assets are compared byte-for-byte with the independent locked build,
with rootless TCP/Unix smoke and health checks before handoff. Axe supports accessibility review; universal conformance and physical camera coverage
are not claimed. Remote CI is checked on the exact PR head before handoff.

Actual [English/German screenshots](screens/README.md) were inspected. The German follow-up also
passes its WCAG-tagged axe audit. CI must pass on the exact PR head; no advisory ignores were added.
