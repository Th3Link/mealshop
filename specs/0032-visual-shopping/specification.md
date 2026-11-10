# Visual household shopping

Status: Done
Priority: Must
Origin: Customer request and supplied Bring reference, 2026-11-10.
Milestone: M2

## User value
Recognize groceries quickly, check them off with one tap and add missing goods without navigating away.

## Acceptance criteria
1. The existing native household shopping destination presents open and purchased goods as responsive, named tiles with consistent decorative icons and exact quantities. One tap checks/unchecks an item. Text and check state accompany color.
2. A labelled search retains input focus while filtering translated names. Adding a named item from search opens an inline exact-quantity form; successful saving restores discovery focus. No quantity or ingredient identity is inferred from a name.
3. The last successful check-off can be undone immediately. Errors retain confirmed server state and do not announce success; mutations keep existing writer/revision checks.
4. Item details retain all recipe/production contributions, purchase receipts, shop assignment/fallback and keyboard ordering. Searching cannot silently replace a complete shop order with a filtered subset.
5. Real embedded Nextcloud verification covers keyboard/focus, reader access, failed writes, light/dark, reduced motion and 320/390/768/1440 widths. No customer household is a fixture. Existing F31 mealprep behavior remains intact.

## Boundaries
The reference supplies interaction direction, not brand colors, illustrations, fonts or a replacement shell. Nextcloud components/tokens take precedence. No fake recent-use history, automatic restock, new backend endpoints, quantities, nutrition or allergy classification.

## Links
Implementation: [plan.md](plan.md). Interaction contract: [ux-notes.md](ux-notes.md). Decisions: [adrs.md](adrs.md).
Acceptance evidence: [validation.md](validation.md). Actual browser screenshots: [screens/](screens/README.md).
