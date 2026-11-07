# Decisions for internationalization

- [ADR 0007](../../docs/adr/0007-localization-and-web-clients.md): de/en first; demo vs product boundary.
- [ADR 0002](../../docs/adr/0002-quantities-and-nutrition.md): preserve exact amounts and unit meanings.

## Slice-local proposals
Use a small framework-free ES module for demo-only resources and pure locale helpers, with
Node's built-in test runner. No JavaScript dependencies or production i18n framework are chosen.
Scope starter ingredient translations by source record identity, not a possibly colliding
operator-defined ingredient ID. Untranslated data keeps its source wording and attribution.
Canonical HTTP decimal strings remain dot-separated; only the presentation boundary uses commas.
