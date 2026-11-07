# Pragmatic recipe demo UX

Scope: disposable demo only, English keyed base with German translations. No product design.

Use the existing ingredient lines/precise grams/servings controls with title, author attribution,
recipe text language and optional ordered steps. Each step has instruction and active/passive
seconds; add/remove use labelled buttons. Inputs have visible labels and numeric constraints.
Nutrition remains prominent in the existing per-serving results and source/unknown tables.

Save creates a new recipe or revision of the loaded identity. Show saved ID/revision in the
live status region. List/refresh/load expose an empty state and storage failure state. Load
and new explicitly say they replace/clear inputs. Failed validation, writes or conflicts keep
inputs; conflicts require copying edits before reloading/reconciling. Changing form fields
hides stale estimates. A successful save/load renders nutrition from the saved source snapshot.
Language changes preserve exact amounts and never translate user-authored text automatically.
A loaded recipe with missing current IDs is shown with its saved nutrition and an explicit
message: restore the catalogue or deliberately replace the missing ingredient before saving.
The UI must not silently pick another ingredient. Saving a revision uses the current catalogue;
this rule and same-batch serving division appear beside the controls. No authenticated author
or allergen safety claim, timer scheduling, sharing or proportional scaling is implied.
