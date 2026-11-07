# ADR 0004: licensing hosting and sharing

Status: AGPL and business goals accepted; admission/commerce/federation proposed
Date: 2026-11-07

## Context
The customer requests AGPL-3.0 software, free Nextcloud installations, paid official app distribution, optional self-builds, hosted tiers and later paid content/federation.

## Decision
License project code AGPL-3.0-only with complete source. Treat app purchase/hosted service entitlement as service policy, separate from software rights. Validate platform attestation plus server-side account entitlement before implementing official-client admission; never rely on a secret in an open client. Preserve source-link-only sharing on every output path for imported recipes. Purchased recipe content needs a separate explicit content licence and provenance. Federation is opt-in and later.

## Alternatives
Obscuring source or prohibiting AGPL redistribution conflicts with the stated licence. A signature alone does not establish account payment or prevent all custom clients. Republishing imported content contradicts the customer sharing restriction.

## Consequences and open questions
Platform feasibility, AGPL compliance of admission terms, content rights, refunds/tax and federation trust remain gates before commercial launch. No restriction on self-hosting or self-building is introduced by this bootstrap.
