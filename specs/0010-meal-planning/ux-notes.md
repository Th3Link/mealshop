# Pragmatic family-planning demo

Use household credentials already present in the demo. Load family/plans explicitly; choose
participants, date/meal, pinned recipe and preparation budgets. Both manual selection and a
bounded multi-select candidate list support suggestions. Show blocked reasons, current profile
notes, timing uncertainty, source/attendance overrides, scaled nutrition and shopping deltas.
A separate review checkbox and Apply action commit a feasible preview; changed fields invalidate
it. Read-only users can preview but server authorization rejects mutation. Errors keep inputs.
Saved slots can be loaded, replaced or removed after an explicit removal preview.

Calendar input is a bounded JSON attendance snapshot, with source/date/event/proposed member IDs.
Consent is separate from import and manual attendance override is explicit. No external writes.
Unknown timing/data yields conflicts or dietary review, never an inferred safe result. Strings
use English-base keyed resources with German equivalents; this remains a disposable demo.
