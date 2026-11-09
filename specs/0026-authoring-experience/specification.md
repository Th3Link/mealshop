# Comfortable native authoring and recipe structure

Status: In progress
Origin: Customer screenshots and clarification after PR #19

1. Use native Nextcloud controls with persistent readable labels, clear section hierarchy, consistent input sizes, unclipped selection names and translated empty states. Preserve keyboard/focus, responsive light/dark operation and drafts.
2. Recipe ingredient search offers creating the entered ingredient directly as a selectable result. The dialog pre-fills its name and returns it to the same row without losing quantities or the recipe. Remove the separate inline-create button.
3. Optional food constituents are added individually with exact quantities per 100 g, evidence and unknown states. Beta-glucan and soluble fibre appear in their own category, outside ordinary nutrient entry and results. Existing values and recipe snapshots stay readable. Aggregate selected constituents across recipe ingredients and servings without inferring absence as zero or claiming a health effect.
4. Sorbitol is an explicitly selectable intolerance trigger in ingredients and household profiles, distinct from the fourteen regulated allergen groups. Declared presence blocks affected attendees through composition and pinned snapshots; absence remains unknown.
5. Named recipe parts act as recipe-owned subrecipes, each owning ingredient lines and steps. Steps can reference exact ingredient rows and previously prepared parts. Parts can depend on other parts; reject unknown references, duplicate identities and cycles. All ingredients are counted exactly once for nutrition, shopping, scaling and constraints. Persist associations across edits/restart; flat legacy recipes still load. No predefined bread phases, independent saved-recipe linking, timer scheduler or automatic cooking-loss assumptions are introduced.
6. Complete backend, native-browser, accessibility and repository gates with preserved auth/revision/storage mechanisms. Audit surrounding app navigation, lists and forms for the same layout issues. Only after this slice is verified, organize the full existing catalogue in a separate subsequent slice.
