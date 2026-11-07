# Household demo UX

The existing demo adds a protected household section with session-only token input, load,
member selection/new member, name, allergy/intolerance notes, hard exclusions and soft dislikes.
Ingredient selections use canonical IDs and localized catalogue names. Save/remove require
operator authorization; conflicts retain edits and explain reload/reconciliation. Load/export
is explicit. Evaluation reviews current recipe ingredients for all recorded members and
shows blocked or review-required, never safe. Tokens/profiles are never stored in browser
storage. Empty/new and access/error states use English-base keyed de/en messages.
