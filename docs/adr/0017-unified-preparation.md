# ADR 0017: Unified preparation and recipe-line instructions

Status: Accepted for the customer-authorized F31 implementation

## Context
Mealprep and long preparations must share the meal plan. A recipe-line temperature describes how to use a quantity, not another ingredient. A later intelligent scale needs typed exact values rather than parsing translated prose.

## Alternatives
Independent calendar copies lose provenance and duplicate shopping. Arbitrary untyped JSON or ingredient-name suffixes cannot distinguish units, targets and measurements. A device protocol now would invent an integration contract.

## Decision
Keep recipe-owned part references and add optional versioned line extensions. The initial discriminated variant is temperature: Celsius, exact signed decimal, original input and recipe-author provenance. Validate at the API boundary. Do not scale temperature with servings or treat it as measured water temperature. Add production and meal allocations to protected planning with revision-pinned recipes; aggregate production once and retain existing household safety checks.

## Consequences
Legacy lines remain valid. New data needs matching-version backup for downgrade. Existing recipe JSON can carry instructions to a future device adapter, but no connection or device execution is promised. Calendar anchors and elapsed timers remain separate. Preserve original quantity input and unknowns.
