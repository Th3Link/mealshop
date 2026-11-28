# Decisions

- **Deterministic command reader, no model.** The customer prefers low-cost AI and voice control.
  A plan sentence has a small closed vocabulary (action, day, meal, count) plus a recipe title
  from the household's own list, so fixed word lists, calendar arithmetic and title matching
  cover it exactly, offline and for free. Alternative considered: send the sentence to a
  language model through Nextcloud Task Processing; rejected for this slice because it adds
  latency, cost and a provider that the local instance lacks, for no gain on this grammar. A
  small model remains the planned fallback for free-form requests only.
- **Commands stop at the review.** F10 makes apply a reviewed, revision-checked step and hard
  constraints cannot be acknowledged away. A spoken sentence is easy to mishear, so it fills
  the planner and loads the preview; applying stays an explicit action on the screen.
- **Both languages always.** German and English vocabularies are read together. A household
  speaks one language and titles in another; switching the display language must not change what
  a sentence means.
- **Weekday arithmetic is explicit.** A weekday means its next occurrence including today;
  "next" means that weekday of the following calendar week; a `d.m.` date that already passed
  means next year. The resolved date is always shown before anything is applied.
- **Recipe matching refuses to guess.** Every heard title word must match a title word (prefix
  of four letters or more, or one edit in words of five letters or more). One fitting title is
  taken; several are offered; none is reported.
- **Client only.** No server, schema or MCP change. Speech recognition is the browser's, as in
  F39 dictation; its vendor may process the audio and the UI says so.
- **Planner placement.** The planner keeps `NcAppSidebar` and its fields, positioned as a fixed
  panel so that it is visible when it opens. The recipe `<select>` became a searchable `NcSelect`.

## Customer decisions, 2026-11-28
- **F40-REVIEW: A, one button when nothing is flagged.** The client decides from the preview and
  sends `reviewed: true` with the click; the server contract is unchanged. The rule is
  deliberately narrow: any allergy, intolerance, dietary rule, exclusion, dislike, unidentified
  ingredient, conflict or warning keeps the explicit acknowledgement. The cook still sees the
  summary, the shopping changes and the two general allergen notices before the button.
- **F40-NOSTEPS: A, plannable with unknown duration.** `preparation_time_unknown` moved from
  `conflicts` to a new `warnings` list in the preview and the `timing` object is omitted, so no
  client can mistake an unknown duration for zero. Time budgets are not checked for such a
  recipe. Clients that ignore `warnings` (Flutter today) plan it without the notice.
