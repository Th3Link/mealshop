# F14 delivery plan

1. Research requested methods with primary sources; record the native UX and source/timer/storage boundary.
2. Separate real food/brew process references while proving old wire/storage/timer behaviour unchanged.
3. Deliver source-linked method/equipment templates, immutable brew recipes, exact readings/calibration,
   reviewed batches/reservations and durable independent phase/reminder actions through real APIs.
4. Build the native batch/recipe/equipment/reading workflow with typed EN/DE contracts and recovery.
5. Validate every criterion with isolated domain tests, real Nextcloud owner/reader and restart flows,
   responsive/axe/screens, rootless transports and the required full gate. Clean only owned fixtures.
6. Complete one reviewable PR against current main; all current-head CI checks must pass before handoff.

- [x] Read requirements, F14, accepted timer/delivery architecture and current main/F15 merge.
- [x] Research BIAB, partial/extract/all-grain and named/DIY equipment; retain source/model distinctions.
- [x] Record native UX and the shared-process/source decision before implementation.
- [x] AC 1: distinct process/equipment templates and exact recipe/batch/reading facts.
- [x] AC 2: explicit pre-brew yeast reminder, phase timers and durable personal delivery.
- [x] AC 3: original units, calibration, timestamp, source and reasoned reading corrections.
- [x] AC 4: atomic equipment reservations and independent simultaneous batches.
- [x] AC 5: beer-only validated templates; reject unvalidated cider/wine/mead assumptions.
- [ ] Native acceptance, all gates, installed evidence, owned-fixture cleanup and current-head CI.

Implementation checkpoint: all five domain criteria are implemented. The full local gate passes
178 Rust tests and 36 native client contracts. Actual Nextcloud owner authoring/timers/readings/offline/
resource conflict and reader/uncertain-save flows have passed with isolated fixtures; final image,
installed restart, complete existing browser suite and current-head CI evidence remain pending below.
