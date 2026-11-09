# Daily workflow validation

Status: In progress; packaged-image and restart verification complete, PR CI pending.

- Rust acceptance suite: 48 passed, including optional author/restart, household declarations,
  composition cycles, isolation, archive, reverse lookup, private raster assets and pinned hard
  allergen/tag exclusions, concurrent tag/plan application and legacy-overlay loading. New archives are also rejected when introduced through recipe writes.
- Strict client type checking and five unit tests passed. Compact decimal display retains exact
  numeric value and does not modify source precision or the six-decimal input contract.
- Real Nextcloud 35.0.2/AppAPI 35.0.0 browser suite: five tests passed together (approximately 1.5 minutes).
  Existing invitation/reader/revocation/conflict, localized quantities and native themes still pass.
- The new browser workflow verifies account-scoped default household and switching, focus across
  every search character, inline composite creation without losing the recipe title, empty author,
  200 g/400 g display, a valid PNG above 128 KiB through the proxy, image reload, shared favourite,
  archive/restore, reverse recipe lookup, calendar attendance and hard tag conflicts only for attendees.
- Browser bounds and automated accessibility are checked at 320/390/768/1440 pixels; native
  light/dark and reduced-motion checks remain in the suite. Screenshots were inspected and the
  calendar was refined to align slots and restore focus/week context after editor operations.
- Two additional daily-workflow tests passed together in 33.5 seconds, including simultaneous
  ingredient editors: stale writes preserve the draft; explicit reload obtains the saved version.

## Final packaged deployment

The complete immutable image (2635bf700f9f) is running as UID/GID 1000 with a read-only root
filesystem. The temporary development asset mount was removed and its absence inspected.
All six browser tests passed together against this package in approximately 1.6 minutes,
including recipe thumbnails, cancelled unsaved-image navigation, unchanged household defaults
when a switch is cancelled, and an independently created ingredient conflict fixture.

A real container restart preserved canonical authenticated API responses for admin and the
dedicated test account across 21 household memberships. SHA-256 comparisons confirmed all
120 persisted files unchanged; nine currently referenced images were also retrieved and
checked for exact bytes, private cache policy and no-sniff headers. Enabled state and household
access recovered without a manual enable hook. Private payloads were not printed.

Rootless TCP and Unix-socket smoke tests passed, including creation of a household ingredient
and recipe and mode-0600 verification of the overlay. Production image build uses Rust 1.85.

Private pre-change copies remain under ignored var/nextcloud/backups. Seventeen old strictly
recognised synthetic browser-fixture memberships were retired from the admin selector, with
ownership/data retained by the dedicated test account; four admin households remain. Fixtures
with unrecognised profile names, recipe titles or other members were excluded from this cleanup.
No existing Nextcloud data reset or AGENTS.md changes.

Repository gates and client audit passed locally. Remote CI is tracked in the feature PR and
must complete before review is declared ready. These checks do not establish formal WCAG
certification, a live calendar provider or nutrient composition for household declarations.
