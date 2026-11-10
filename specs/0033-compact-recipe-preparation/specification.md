# Compact recipes and explicit homemade ingredients

Status: Done
Priority: Must
Milestone: M2
Origin: Customer corrections to F31 recipe display/editing, 2026-11-10.

1. Match the approved recipe references with compact ingredient rows and desktop ingredients/instructions side by side. Parts, metadata and instructions remain expanded; no disclosure is needed to read/edit the recipe. Stack naturally on narrow screens.
2. Show ingredient/quantity/unit column labels once. Retain accessible row labels, complete selection names, exact values and keyboard operation. Row-end trash removes a line. Optional ingredient actions live behind a labelled native plus menu; adding temperature is one such action, without a repeated temperature invitation or a water-specific rule.
3. Buying remains the legacy default. A recipe may explicitly reference a recipe-owned preparation part for a homemade ingredient using optional `prepared_part` on its ingredient line. No global availability is inferred from a food name. The user declares that part's measured base ingredients and instructions; no recipe, yield or conversion is invented.
4. The finished reference retains its original declared mass/input, identity and source snapshot. Only measured base lines count toward nutrition, constituents, shopping and meal-plan deltas, exactly once. Nutrition explicitly reports unmodelled cooking/yield. All identities, including prepared references, remain in conservative dietary review; homemade never bypasses hard exclusions.
5. Validate existing parts, nonempty transitive base ingredients and instructions, one output line per preparation, and cycles across all dependency kinds. Invalid/stale/read-only writes leave persisted state unchanged. Saved revisions, scaling, reopen and cooking retain preparation references. Cooking depends on the corresponding preparation steps.
6. Creating a preparation keeps the current draft and adds a visible part to complete. Switching a simple preparation back to purchased, or removing its output, confirms removal of that preparation's inputs/steps. Referenced/nested preparations require resolving dependencies first; do not silently discard shared work. Legacy flat recipes retain behavior.
7. Verify actual embedded Nextcloud recipe editing/reading, both languages/themes, keyboard, exact scaling, inline creation, homemade shopping and cooking, 320/390/768/1440 widths, axe, repository/client/container gates and restart. Preserve customer data and archive only verified own fixtures.

See [UX contract](ux-notes.md), [decisions](adrs.md) and [delivery plan](plan.md).
