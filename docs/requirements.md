

F43 records the customer's request (2026-11-28) for a separate Rust REST service at
`interdata.mealshop.luehr.dev` that buffers uploaded recipes and ingredients and publishes them
only after half-automatic, AI-assisted checking; recipes carry tags and CDN image links instead
of images. Customer decisions of the same day: condition tags need no evidence and are
plausibility-checked and labelled unverified; uploads are open to registered users without stored
authorship, registration itself being separate and undecided; checks escalate from rules to a
self-hosted micro model to a moderator-approved frontier model; a fixed operator-configured
upload secret admits uploads until registration exists. Slice 1 delivers intake and buffering
only ([guide](guides/interdata.md), ADR 0040); checking, review, publication and deployment are
not implemented. The content licence is open. N03/N05/N06/N09 apply; see
[F43](../specs/0043-interdata-intake/specification.md).
