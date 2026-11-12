# F11 model-change evaluation

The versioned fixture is [recommendations-v1.json](../../tests/fixtures/recommendations-v1.json).
It contains four **synthetic** held-out choices with separately dated training observations and
held-out 1–5 rating labels. It is hand-authored test evidence, not customer data or a user study.
The evaluator uses the production incremental aggregate and ranking functions.

Run `cargo test --locked --lib feedback::evaluation -- --nocapture` to reproduce the comparison.
Changing an algorithm version requires updating the explicit fixture contract and reviewing all four
metrics. A constraint violation fails the test independently of any preference improvement.

| Metric | `rules-v1` baseline | `history-v1` learned rule |
|---|---:|---:|
| Selected hard-constraint violations | 0 | 0 |
| Held-out acceptance proxy (first choice equals labelled choice) | 0 / 4 | 4 / 4 |
| Distinct first choices across the four cases | 1 | 2 |
| Held-out rating-utility proxy (rating points / choices) | 4 / 4 | 20 / 4 |

These fractions retain their denominators and the 1–5 rating scale. They do not estimate a real
acceptance rate, prediction confidence, human satisfaction or causal benefit. The fixture deliberately
checks learnable preferences; it cannot establish generalization or demographic fairness. Real-world
effectiveness remains unknown until sufficient observed evidence and an appropriate study exist.

The offline eligible set includes a strongly rated blocked candidate that is excluded before ranking.
Separate authenticated API regressions exercise real current household exclusions against high positive
ratings, opt-out, source revisions and ordinary planning. Existing dietary regressions cover allergen
and intolerance declarations, frozen/current composition and hard-review enforcement. Cold-start,
sparse history, discovery beyond the first 20 recipes, whole-pool pagination and reset are independently
tested. Thus preference gains cannot waive the constraint gate, and the synthetic metric is not used
to justify weakened safety checks or a mandatory ML framework.
