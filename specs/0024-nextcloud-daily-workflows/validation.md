# Daily workflow validation

Status: In progress; final immutable-image deployment and CI still pending.

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

The running test container was gracefully stopped and a private pre-change data copy retained
under ignored var/nextcloud/backups before the first rollout. UI iteration temporarily mounts only
built assets read-only into the actual test container. Final validation must remove this temporary
mount and verify the complete packaged image and restart persistence before completion.

No existing Nextcloud data reset or AGENTS.md changes. These checks do not establish formal WCAG
certification, a live calendar provider or nutrient composition for household declarations.
