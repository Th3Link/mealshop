# UX

Show groups as expandable native ingredient lists with subtype counts; searching a type keeps its parent context. Use the same parent-qualified names in recipe selection. The ingredient editor adds group/type controls, a distinct dietary/origin section, and a nutrient table with units per 100 g. Empty numeric fields mean unknown; zero is an explicit value. Inheritance is opt-in and clearly labelled. Existing primary references are preserved separately from household-declared effective values.


Existing native Nextcloud components, typography and theme tokens take priority. UX UI Pro Max is
used for labelled forms, progressive disclosure and derived Vue state; Taste contributes alignment,
spacing and screenshot review only. Its website hero, palette, font, navigation and effects generators
are deliberately excluded by the customer's native-app constraints. No separate design system or
validation library is introduced. Numeric fields appear only for an own profile; inherited values are
labelled in the ingredient detail. Automated browser checks run inside the real embedded Nextcloud
shell, not an isolated marketing-page gate. Archived parents retain their identity in editors and do not
hide active subtypes. The ingredient edit action explicitly includes all ingredient metadata.
