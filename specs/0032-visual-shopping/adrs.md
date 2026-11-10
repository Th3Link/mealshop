# Decisions for visual shopping

Existing [ADR 0015](../../docs/adr/0015-shopping-provenance-and-local-learning.md) remains the data contract.
This change uses the existing shopping/catalogue endpoints and Nextcloud components. The supplied Bring
reference informs tap/discovery patterns; its assets and branding are not copied. UI-only filtering and
last-action undo introduce no new persisted model or dependency.
