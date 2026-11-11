# F11 delivery plan

## Build order
1. Completed history and incremental aggregates: validated model → atomic store → protected API → native recording/editing.
2. Planner ranking: canonical feasibility checks → whole-pool baseline/learned ordering → native reasons/review.
3. Privacy and evaluation: owner scope/reset/deletion → deterministic offline benchmark → live role/restart/browser verification.

These are verifiable parts of the existing F11 contract, not new standalone products or adapters.

## Tasks
- [x] Read current requirements, F11/ADR 0005 and actual cooking/planning/household boundaries.
- [x] Record the conservative delivery contract and native flows before implementation.
- [x] Implement completed/manual history, validated incremental updates and durable restart.
- [x] Integrate household roles, trusted actors and source validation with API regressions.
- [x] Deliver and verify native history entry/list/edit/delete and completed-session entry points.
- [x] Reuse canonical feasibility checks and add versioned whole-pool explainable ranking.
- [x] Implement owner-only opt-in/scope/reset and separate history deletion.
- [x] Compare baseline/learned rules with labelled offline acceptance/variety/rating proxies.
- [x] Verify real Nextcloud roles, exact values, responsive accessibility, restart and synthetic cleanup.
- [x] Run required repository/client/container gates and prepare the single-PR delivery with acceptance evidence.

The PR's live exact-head CI is the remaining remote gate before handoff; its result is recorded on
the PR rather than inferred from this committed checklist. See [validation.md](validation.md).

## Acceptance trace
AC 1: explicit completed/manual history and immutable source tests plus native editor workflow.
AC 2: incremental correction/deletion/count tests and explained suggestion integration.
AC 3: cold-start/discovery tests and visible unreviewed dietary states.
AC 4: reproducible versioned synthetic baseline comparison; no human-effectiveness claims.
AC 5: role/scope/epoch/reset/deletion tests and constraint checks after opt-out.
