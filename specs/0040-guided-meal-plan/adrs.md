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

## Open customer decisions
- **F40-REVIEW.** Whether a plan whose review has nothing to flag may be applied with one button.
- **F40-NOSTEPS.** Whether a recipe without timed steps may be planned with its duration shown
  as unknown; today it is a hard conflict.
