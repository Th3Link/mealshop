# Validation

Local, 2026-11-28, Nextcloud 35.0.2 test instance with the locally built server and client.

| AC | Evidence |
|---|---|
| 1, 3 | `clients/nextcloud/tests/plan-command.test.ts` (6 tests) |
| 2–4, 6–8 | `clients/nextcloud/e2e/guided-meal-plan.spec.ts` in embedded Chromium, incl. axe, 390 px and 1440 px |

| 9 | `guided-meal-plan.spec.ts`: one-button apply and remove without acknowledgement |
| 10 | `tests/planning.rs::missing_timings_and_storage_failures_never_silently_apply`; `guided-meal-plan.spec.ts`: recipe without steps shows the unknown duration and needs the acknowledgement |

Rerun and passed with the reworked planner: `daily-workflows` (both tests), `unified-preparation`,
`usability-review`, `shopping`, `nextcloud` (two of three) and, in an earlier full pass,
`feedback` suggestions. The remaining `nextcloud` and `feedback` tests need a second (reader)
account that the local instance lacks.

Not verified: speech input with a real microphone (AC 5; headless Chromium has none), Firefox and
Safari, the customer's production instance.
