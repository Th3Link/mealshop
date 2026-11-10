# F13 acceptance evidence

Scope: recipe-derived native Nextcloud baking plans; all original acceptance criteria remain unchanged.

| AC | Evidence |
|---|---|
| 1 | `cooking::plan::tests::dependency_windows_cycles_and_dst_use_absolute_elapsed_time` verifies multi-day dependencies, repeated occurrences with waits and shortest/longest passive bounds. API test `multiday_replans_save_the_reviewed_deadlines_and_keep_delays_across_later_changes` proves preview causes no durable write and apply stores exactly the reviewed schedule. |
| 2 | Action/equipment/lead times are explicit stage fields; preparation, planned-start, active-phase and elapsed events share the durable outbox. Delivery tests verify recipient, action, equipment and event identity without inventing a baking preset or a fermentation outcome. |
| 3 | Replan tests fix actual completed work, preserve explicit delays across later target changes and flag missed targets, active overlaps and equipment overlaps. Native target-change review presents old/new local times; arbitrary timers and pending starts use the same review/apply boundary. Unspecified conditions remain unknown. |
| 4 | Jiff resolves IANA zones with bundled data. Unit/API tests cross spring and autumn offsets, reject nonexistent local times and require an explicit fold choice. Browser creates a multi-day plan across the Europe/Berlin autumn transition, reviews both UTC offsets and changes its target. |

Shared quality evidence and notification limitations: [F12 validation](../0012-guided-cooking/validation.md).
Additional regression tests enforce repeated-step waits from actual completion and preserve explicit
start delays. A paused phase projects a resume now even with a future baking target.

No temperature sensor, biological prediction, commercial recipe collection, standalone shell or Flutter
alarm implementation is implied. See [ADR 0016](../../docs/adr/0016-durable-cooking-and-civil-time.md)
and the [operator/user guide](../../docs/guides/cooking.md).
