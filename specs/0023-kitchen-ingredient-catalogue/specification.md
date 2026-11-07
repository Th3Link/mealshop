# English-base kitchen ingredient catalogue

Status: In progress
Priority: Must
Origin: Customer (2026-11-07)
Milestone: M1

## User value
Try real recipes with a broad set of familiar ingredients instead of a three-item catalogue.
All coded names/content have an English base and stable identifiers, with German translations
kept separately. The existing browser surface remains a pragmatic demo.

## Acceptance criteria
1. Canonical ingredient IDs, category identifiers and base names are language independent or
   English. German names/aliases/forms live in keyed translation fields. Selecting a language
   never rewrites identities, nutrients, units or canonical decimal strings.
2. Include wheat/rye/spelt grain, their common German flour types and wholemeal forms, semolina,
   water, skimmed/low-fat/whole/UHT/lactose-free milk, plant drinks, cocoa/sugar/xylitol/erythritol,
   quark grades including double-cream identity, cream/butter/margarine/eggs, tofu/legumes/edamame/
   quinoa, the requested vegetables, herbs, potatoes/skyr, conventional/wholemeal/legume pasta,
   tomato products, protein powders/products, rice paper, beetroot, chicken cuts and varied cheeses.
   Useful related seeds/nuts/oils/baking basics are included. Names distinguish raw/cooked,
   fresh/dried, fortified/unfortified, product-specific and fat-in-dry-matter variants.
3. Every numeric profile maps to an exact frozen primary source record with unit/basis, publisher,
   version, retrieval date, licence and source uncertainty/acquisition metadata. Reviewed BLS 4.0
   records provide German flour types and dairy variants; USDA adds appropriate herbs/reference powders.
4. No nearest-food substitution: unresolved product variants are present as clearly named
   identity-only records with unknown composition. Lentil flour is not lentil pasta; whey powder
   is not a universal whey isolate; double-cream quark does not inherit 50%-in-dry-matter quark data.
5. Keep BLS available carbohydrate separate from USDA by-difference carbohydrate, and food folate
   separate from total folate/DFE. Mixed definitions yield separate/partial totals with an
   explanatory warning, never a silent conversion. Existing oats/chia recipe results remain unchanged.
6. Source extraction/import is reproducible offline from committed snapshots/selections. Preserve
   original source cells/tokens and references; document six-decimal import normalization and
   never convert unavailable/below-resolution positive data into claimed zero.
7. Search finds English/German names and aliases, common plurals and multiword queries (e.g.
   Weizen 405, Hafermilch, Magerquark, Kichererbsen, Eier). The demo uses keyed English-base UI
   messages with German display and exact comma input/output, without product UI design expansion.

## Current delivery scope
313 identities: 284 source-backed profiles, 29 identity-only records; 3,778 mapped nutrient values
across 17 supported nutrient definitions. Data selections are in `data/catalogue-selection.json`.
Source-backed means source composition exists, not that all 17 definitions are available or
that the food is a measured match for the customer's specific product.

## Out of scope
Every worldwide brand, automatic provider refresh, invented product averages, exhaustive
micronutrient coverage, medical outcomes, storage adapters or production web clients.
Product-specific pulse pasta/protein products and other unresolved variants need verified
package/manufacturer records before their nutrients can be populated.

## UX notes
Use the existing demo and F21 language controls. Original source/evidence prose is labelled
as original when no reviewed translation exists. No design system or Vue migration in this slice.

## Links
[F02](../0002-ingredients-nutrition/specification.md), [F21](../0021-internationalization/specification.md),
[F22](../0022-product-web-clients/specification.md), [ADRs](adrs.md), [plan](plan.md).
