# ADR 0007: localization and web clients

Status: Customer direction accepted; product integration details proposed
Date: 2026-11-07

## Context
After merging PR #4, the German-speaking customer requested internationalization and
explicitly classified the existing browser workbench as a pure demo. Further demo design
investment should stay pragmatic; substantial UX/UI work should be handed to a dedicated
UX/UI agent when the customer chooses to involve one. Nextcloud and the standalone server
need product web clients, with Vue 3 as the preferred low-friction direction. Flutter remains
the separate app target from the original brief.

## Decision
- Keep `src/web/` a disposable developer/test demo, visibly labelled as such. Use it to try
  domain/API behaviour; do not develop a product design system or Nextcloud shell there.
- Introduce F21 for cross-cutting i18n: interface messages, localized ingredient/content names,
  language negotiation/fallback, number/date/plural presentation, exact locale-aware quantity
  entry and stable language-independent identities/contracts. Start with German and English.
- This change adds a small de/en demo message catalogue, record-owned ingredient
  display names, exact decimal formatting/input and explicit original-language content fallback.
  It does not introduce a production translation service or localize stored nutrient values.
- Plan F22 for Vue 3 product web clients: a Nextcloud-hosted shell using compatible Nextcloud
  components and AppAPI-aware asset/API routing, and a standalone server web shell. Share
  domain API contracts and translation semantics, while letting host navigation/auth differ.
- Choose concrete packages, compatibility targets and component-sharing boundaries when F22
  becomes Ready. Do not install Vue/build tooling or start a framework migration in this slice.

## Evidence and alternatives
Nextcloud's [JavaScript standards](https://docs.nextcloud.com/server/stable/developer_manual/getting_started/coding_standards/javascript.html)
recommend Vue and its provided components for UI consistency. The
[ExApp overview](https://docs.nextcloud.com/server/stable/developer_manual/exapp_development/development_overview/ExAppOverview.html)
describes a Vue frontend with proxy-aware loading paths. These support the requested direction;
they do not prove compatibility of the current root-relative demo with a Nextcloud installation.

Growing the demo into the product would mix a debugging surface with host integration and
consume design effort the customer explicitly does not want spent now. Starting two full web
projects immediately would add unrequested implementation before the UX and integration specs.

## Consequences
F02 is accepted through merged PR #4; its browser surface remains demo-only. F21/F22 retain
separate completion criteria. The first i18n slice leaves source metadata, uncertainty and
educational content in the original language with an explicit label when no reviewed
translation exists. Full versioned ingredient/recipe/content translations are later F21 work.
The demo may remain framework-free or be removed once product clients provide equivalent
review flows. A UX/UI agent is not spawned by this decision; the customer decides when to involve one.
