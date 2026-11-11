# ADR 0021: completed cooking history and local preference epochs

Status: Accepted for implementation under F11
Date: 2026-11-11

## Context
F11 needs actual cooking/ratings and incremental preferences without treating plans as observed
meals or weakening dietary constraints. Cooking sessions and dietary profiles are household-owned;
profile IDs are not authenticated personal voting identities. The existing planner already checks
current plus frozen declarations, time budgets and exact quantities.

## Alternatives
Counting planned meals fabricates observations. Timer estimates are not measured active effort.
Remote/cross-household training and an opaque framework are unnecessary before evaluation. Replaying
all retained ratings after opt-out would silently rebuild removed preference state. Independent
candidate-batch rankings do not provide a coherent recommendation order for a large recipe library.

## Decision
Add bounded atomic household history with explicit manual completion or a verified completed session.
Retain pinned recipe/source metadata and actor/time provenance; correct/delete with optimistic revision
checks. Session records are unique and survive removal of the session. Ratings are explicitly reported
within the household's existing reader/writer visibility; no private personal-vote claim is made.

Only the owner controls learning, off by default, with optional selected profile-rating scopes.
Changing/enabling/resetting scope starts a fresh epoch and clears aggregates. Only new/explicitly
corrected current-epoch records contribute. Opt-out disables learning; separate history deletion
controls retained raw observations. Incremental integer sums/counts are checked against eligible
records on restart, so corrupted counters fail closed rather than changing rankings silently.

Filter with canonical planner constraints before a versioned, explained integer preference rule.
Rank the whole bounded candidate pool and return a limited number of full previews. Preserve cold
start and discovery; evidence counts are not probabilities, clinical confidence or allergy guarantees.
Evaluate model changes offline against the baseline with a versioned synthetic holdout and explicitly
labelled acceptance, variety and rating-utility proxies. Do not claim measured human satisfaction or
causal improvements without real evidence. No private household dataset is exported or published.

## Consequences
No new crate, identity system or storage adapter is needed. Existing recipe/history/plan/grant files
remain compatible. A history file and its learning aggregates share one atomic transaction; ambiguous
writes stop mutations until verified recovery. Hard dietary constraints and explicit plan review keep
priority over preference ordering. API/UI visibility, reset semantics and acceptance commands remain
in the F11 contract. Individual account consent/privately authenticated voting is outside this existing
household profile model; the UI states who reports and can see feedback.

Feedback is pooled by stable recipe identity, while counts from earlier revisions are explicit in
each explanation. Current target-revision constraints always take precedence. Both rule and learned
ordering retain deterministic discovery within equal declared-dislike groups; the learned version
uses candidates without evidence in the current epoch. Recency counts the seven reported civil days
ending on the selected planning day, without pretending that a date-only history is a measured timeline.
