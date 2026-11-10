# F12 acceptance evidence

Scope: native Nextcloud 35 client and the shared protected Rust backend. The existing F12 contract is
unchanged. No Flutter/standalone shell or guaranteed sleeping-device alarms are claimed.

| AC | Evidence |
|---|---|
| 1 | `tests/cooking.rs::concurrent_named_timers_preserve_pause_deadlines_sources_and_restart`: source revision and servings pinned, exact `150.000002 g` scaled from `100.000001 g`, one start uses the recorded 90 seconds. Native recipe entry and preset start covered in `e2e/cooking.spec.ts`. |
| 2 | Two independent running timers; pausing one leaves the other byte-equivalent. Resume, explicit complete, dependencies and inactive states are API-tested. Unit tests cover active/passive reminders, overdue resume with zero remaining time and explicit adjustment previews. |
| 3 | Private store reopen preserves actual timer states and deadlines. Recovery test turns persisted sending into unknown without replay. Browser reload retains pause; offline mode keeps elapsed time visible and disables writes until reconnect. |
| 4 | Opt-in actor-bound `AppAPI` delivery test covers denied desktop permission, exact recipient and removal of private subscriber/user fields. Browser verifies real Nextcloud acceptance, upcoming actions and offline recovery. Accepted is explicitly not device-delivered. Unit tests cover unknown/failed/missed/cancelled states, bounded retries and duplicate-risk consent. |

`cargo nextest run --workspace --locked`: 81 passed. `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
Frontend unit tests: 13 passed; strict Vue/`TypeScript` and Vite production build passed.
Native browser test uses only a ledger-recorded synthetic household. Axe checks WCAG 2/2.1/2.2 AA tags;
light/dark screenshots at 320/390/768/1440 have no horizontal content overflow. Visual review corrected
the native selector's broken text split, a translation key collision, heading/state alignment and
post-review focus. No external fonts, palette, reset or decorative animation was introduced.

Review experiment: temporarily bypassing the dependency-readiness condition makes the protected API
regression test fail; the source was restored and its diff checked. The test verifies observable
rejection and unchanged persisted bytes, not an implementation mock.

## Runtime and delivery verification (2026-11-10)

- Nextcloud 35.0.2 / `AppAPI` 35.0.0, using the real embedded app and proxy routes.
- Full native browser suite: 17 passed. The final image additionally passed the focused cooking test,
  including a held live backend response during pause and a concurrent tab changing the domain revision
  while a target draft stays open. No page errors or WCAG-tagged axe violations; screenshots reviewed.
- Rootless Rust 1.85 / Vue image: `sha256:c0be9a5cadf7aee80782fb5232387b766c815e36e5d77a48253cc6440d81e271`.
  Both TCP and Unix-socket container smoke tests passed. Deployed JavaScript/CSS SHA-256 matches the
  independently built local assets exactly. Test container reports healthy.
- An actual container restart retained cooking deadlines, a deliberately paused synthetic timer,
  all authenticated API snapshots and 28 persisted files byte-for-byte (10 memberships, one image).
  Volatile server clock/worker-check fields are excluded from API comparison, not timer state.
- Rendered Nextcloud reminder actions and browser-visible links were verified through the real
  notification API. Only identified synthetic notification IDs were removed; unrelated notices retained.
- Recorded synthetic households were archived into complete private backups. Both customer households,
  their registry entries and persisted files match the pre-feature backup; IDs were not reused.
- Required catalogue/import/i18n/fmt/clippy/nextest/licence/audit gate, frontend formatting/types/tests/build,
  `npm audit --audit-level=moderate` (zero vulnerabilities), conversions/allergen verifiers and strict
  rustdoc passed. Remote CI is checked on the exact PR head before handoff.

