# Architecture decisions

Use the next free four-digit number. Record context, alternatives, decision, consequences
and status; supersede accepted decisions explicitly rather than silently rewriting history.
A proposal is not customer acceptance of an unresolved choice.

- [ADR 0001: stack and delivery](0001-stack-and-delivery.md) — Accepted for bootstrap; future integrations planned.
- [ADR 0002: quantities and nutrition](0002-quantities-and-nutrition.md) — Proposed implementation; customer unit preferences accepted.
- [ADR 0003: persistence and data boundaries](0003-persistence-and-data-boundaries.md) — Customer storage targets accepted; schema and adapters proposed.
- [ADR 0004: licensing hosting and sharing](0004-licensing-hosting-and-sharing.md) — AGPL and business goals accepted; admission/commerce/federation proposed.
- [ADR 0005: privacy and learning](0005-privacy-and-learning.md) — Proposed safeguards for customer-requested learning.
- [ADR 0006: durable process schedules](0006-durable-process-schedules.md) — Proposed shared process model.
- [ADR 0007: localization and web clients](0007-localization-and-web-clients.md) — Customer direction accepted; product integration proposed.
- [ADR 0008: curated ingredient sources](0008-curated-ingredient-sources.md) — Customer scope accepted; import/model details proposed.

- [ADR 0009: local recipe revisions](0009-local-recipe-revisions.md) — Local authoring slice; review pending.

- [ADR 0010: protected local household access](0010-local-household-access.md) — Proposed local F09 slice.

- [ADR 0011: family meal planning](0011-family-meal-planning.md) — Local protected preview/apply and consented attendance import.

- [0012: native Nextcloud client and shared household boundary](0012-nextcloud-shared-households.md)

- [0013: Household ingredient declarations and recipe images](0013-household-ingredients-and-images.md)

- [0014: Recipe-owned parts and optional constituents](0014-recipe-parts-and-constituents.md)

- [0015: shopping provenance, shop order and product boundaries](0015-shopping-provenance-and-local-learning.md)

- [0016: durable cooking instances and civil-time planning](0016-durable-cooking-and-civil-time.md)

- [0017: Unified preparation and recipe-line instructions](0017-unified-preparation.md)

- [0018: Homemade recipe inputs](0018-homemade-recipe-inputs.md)

- [0019: Source-backed nutrition ranges](0019-source-backed-nutrition-ranges.md)

- [0020: Revision-pinned recipe collections and explicit sharing](0020-recipe-sharing-collections.md)

- [0021: Completed cooking history and local preference epochs](0021-completed-history-and-local-preferences.md)
- [0022: scoped MCP access and atomic recipe creation receipts](0022-scoped-mcp-recipe-creation.md)
- [0023: Safe public retrieval and reviewed recipe import](0023-reviewed-recipe-link-import.md)

- [0024 Household stock and offers](0024-household-stock-and-offers.md)

- [0025: Native brewing processes](0025-native-brewing-processes.md) — Accepted for F14.

- [0026: Flutter phone delivery and the first offline boundary](0026-flutter-phone-delivery.md) — Accepted for the customer-approved Android sequence; P1 merged and P2 implemented.

- [0028: Effective person rules and private activity access](0028-person-rules-and-activity.md) — Accepted for F35 P2; explicit per-person rights without automatic owner access.

- [0029: Mobile household weeks and prepared stock](0029-mobile-week-and-prepared-stock.md) — Accepted for F35 P3.

- [0030: Mobile cooking recovery and Android reminders](0030-mobile-cooking-recovery.md) — Accepted for F35 P4.

- [0031: Tagged OCI images and a Flux-owned `ExApp`](0031-tagged-oci-releases.md) — Customer-requested release and Helm delivery.

- [0032: App-only core execution and portable backups](0032-app-only-core-and-backups.md) — Corrected customer request for durable on-device operation.

- [0033: Guarded Renovate licence inventories](0033-guarded-renovate-inventories.md) — Automatic version refresh with explicit licence review boundaries.

- [0034: Preserve outbound TLS behavior across reqwest 0.13](0034-outbound-tls-compatibility.md) — Explicit ring and Mozilla roots for server and Android clients.

- [0035: Keep password backups on the audited age series](0035-backup-dependency-compatibility.md) — Version policy preserves encrypted-backup compatibility and the advisory gate.

- [0036: Official TypeScript 7 and Vue compiler boundaries](0036-official-typescript-compiler-boundaries.md) — Native modules and full Vue checks use explicit official compiler entry points.

- [0037: Keep Node toolchains within Nextcloud compatibility](0037-nextcloud-node-toolchain.md) — Shared CI version source and guarded Node/Docker updates retain supported strict builds.

- [0038: Guided capture and household-declared unit weights](0038-guided-capture-and-household-unit-weights.md) — One reviewed entry for text, speech and pictures; own unit weights instead of evidence forms.

- [0039: Recipe quantities without a known mass](0039-unweighed-recipe-quantities.md) — Optional line mass; unknown is carried explicitly through nutrition, shopping, planning and cooking.
