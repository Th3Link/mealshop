# Technical debt and validation backlog

- F04/F18 starts bounded local recipe file persistence. Auth, rate limiting, Git/SQL adapters,
  product-client integration remain open. No shared/hosted access is implemented.
- Validate actual Nextcloud `AppAPI` support and Flutter offline/notification behaviour (F18/F12).
- F02 originally supplied frozen CC0 USDA oats/chia values. F23 adds the curated BLS/USDA kitchen catalogue.
  Broader coverage, provider refresh, density, cooking yield/retention and component measurements
  remain future work (F02/F03); psyllium composition is deliberately unknown.
- Validate official-app service admission without weakening AGPL rights (F19 / ADR 0004).
- Binary-size metrics initially have no previous baseline. No performance budget is invented.

These are explicit future work, not completed features or ignored CI findings.

- F23 catalogue generation is offline; 29 identities still lack exact composition. The current
  file is near the pre-existing 1 MiB demo loader bound; wider loading belongs with F18.
- BLS available carbohydrate and food folate stay separate from other definitions; complete
  product coverage and additional micronutrients need reviewed mappings, never inferred conversion.
- F21 first slice covers de/en demo messages, names and decimals. Reviewed source/evidence prose
  translations, date/plural/error parameters across all clients and production locale negotiation remain open.

- Recipe file storage is a Linux/local-filesystem slice with an exclusive lock, manual crash
  recovery and 1000-revision capacity. POST idempotency and non-Unix crash durability remain open.

- Lowest-priority deferred item (customer request): cross-device preference synchronization.
