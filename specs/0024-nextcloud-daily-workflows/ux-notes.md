# Daily workflow UX

Use native Nextcloud controls and tokens. Put the household selector at the top of the existing app navigation and keep household management separately reachable. Remember only the account-scoped household ID in browser preferences.

Recipe rows combine searchable ingredient selection, exact quantity/unit and a trailing labelled trash action. Inline ingredient creation is a bounded native dialog that returns the created ingredient into the same row without leaving the recipe draft. Optional attribution and image controls do not block basic recipe creation.

The calendar exposes a week with four named slots per date. Each slot opens the existing reviewed plan workflow with date/meal prefilled. Attendance is labelled per person, and skipped people are explicit. On narrow screens, days stack rather than making the whole document scroll horizontally.

Ingredient management distinguishes reviewed reference data from household declarations. Source records remain immutable; archive and favourites are annotations. Composition, free tags and structured declared allergens are distinct controls. Reverse lookup names recipes and preserves navigation back to the ingredient.

Search, theme transitions, focus and unsaved-draft handling must be exercised using real Nextcloud DOM and screenshots. No global style reset, font, colour palette or replacement navigation is introduced.
