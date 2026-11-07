# Customer-operated recipe link import

Status: Draft
Priority: Should
Origin: Customer
Milestone: M3

## User value
Keep recipes from source websites in a private recipe library.

## Acceptance criteria
1. Given a customer-supplied URL (e.g. Chefkoch, Instagram, Einfach Backen), an available importer extracts a preview or clearly reports unsupported/blocked content.
2. Given an import preview, the customer reviews ingredient mapping, units, steps and provenance before saving; missing fields remain marked unknown.
3. Given any imported recipe, its original URL and imported-content status persist through revisions, exports and collections; sharing is restricted to the source link.
4. Given an untrusted URL, fetching rejects private/local addresses, unsafe redirects and excessive responses and does not execute supplied scripts.
5. Given source restrictions or required login, import does not bypass access controls; manual private entry remains available.

## Out of scope
Guaranteed support for every website or publication rights obtained merely by importing.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F16](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).
