# Native usability review

Status: Done
Origin: Customer review after merging PR #20

1. Review every shipped Nextcloud view in the existing test instance: household selection/creation,
   recipes/list/detail/create/edit/preview, ingredient discovery/detail/create/edit, profiles,
   access/invitations, calendar/editor/review/shopping. Retain the native shell, permissions and exact data.
2. Protect changed recipe, profile, meal and ingredient drafts during internal navigation, household
   switching and browser exit. Cancel/reload must distinguish unchanged views from real edits and never
   discard changes silently. Mutation failures keep drafts and offer explicit recovery.
3. Make profiles easier to scan: a native two-column allergen checklist, optional notes separated from
   canonical intolerance selectors, clear reset/save actions and announced success. Preserve notes and
   hard exclusions exactly; unselected declarations do not establish safety.
4. Provide recipe title search with a useful empty result and retained query on return. Ingredient
   empty results can reset all active filters. Consistently label person selection and nutrient names.
5. Meal editors distinguish new/existing slots, identify unmet prerequisites, and place keyboard focus
   on the review after preview/suggestion. Save/review interactions remain explicit; no new endpoint,
   scheduling feature or dietary inference.
6. Verify all existing browser workflows plus targeted regressions, both languages/themes, responsive
   layouts and accessibility on the real embedded app. Run repository gates and final packaged checks.
