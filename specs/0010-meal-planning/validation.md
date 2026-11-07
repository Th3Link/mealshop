# F10 acceptance evidence

Status: Done for the documented local family-planning and read-only attendance-import contract.

## Acceptance criteria
1. Protected manual recipe selection and bounded suggestions use pinned revisions, selected
   attendees and F09 review. Known hard exclusions reject candidates; dislikes only rank them.
2. Consented calendar snapshots retain source/event/date/proposal. Tests verify missing consent
   rejection, source display and explicit manual attendance override without any external write.
   The actual supported adapter is JSON snapshot import, not a live provider connection.
3. Step timing distinguishes active, sequential elapsed and advance lead. Tests cover insufficient
   active/elapsed windows and unknown step timing. UI makes original-batch timing uncertainty
   explicit when portions change; no inferred parallelism or batch-time multiplier is claimed.
4. Preview/apply supports alternatives, changed attendees/servings and removals. Tests verify
   signed grocery deltas, derived persisted shopping requirements, revision conflict rejection,
   explicit dietary acknowledgement, pinned historical recipe inputs and restart round trips.
5. Suggestions return rejected conflict reasons and no-feasible explanations; both API and
   browser tests verify hard constraints cannot be bypassed by acknowledgement or soft dislikes.

## Automated results
`cargo nextest run --workspace --locked`: 40 tests passed, none skipped, including four family
planning journeys. Additional focused planning/household runs passed after final arithmetic
error handling cleanup. Six Node locale tests and JS syntax checks passed. Catalogue import,
nutrition source, conversion and allergen verifiers passed. Rustfmt, clippy with warnings denied,
release build, licenses, cargo-audit, strict rustdoc and MSRV 1.85.0 passed. Current-commit CI
links and outcomes are recorded in the PR.

Storage tests simulate interrupted and corrupt plan snapshots; startup fails closed and releases
locks on failure. Reads/export remain protected; readers cannot apply. Existing F09 access,
profile export/removal/conflict/restart and F03/F04 nutrition/quantity tests remain green.

## Browser results
With fictional profiles (one milk exclusion) and oat/milk recipes, suggestions retained the oat
recipe and rejected milk. The preview showed current member notes, active 600 s / elapsed
2400 s / advance 1200 s and +200 g oats for four servings. Explicit acknowledgement/apply saved
revision 1 and 200 g derived demand; reloading preserved the pinned meal. Setting available
active time to zero produced no feasible meal and distinct timing/dietary conflict explanations.

## Scope and limitations
Calendar access is a user-consented, read-only attendance snapshot import with already mapped
household IDs. Live polling, OAuth, ICS recurrence/timezone expansion, external calendar writes,
purchases, inventory and F06's full shopping workflow remain separate work. Review never
establishes allergen safety; data gaps, cross-contact and batch-time changes remain explicit.
Capacity: 100 meal slots / 1 MiB local snapshot and 50 candidate revisions per suggestion call.
