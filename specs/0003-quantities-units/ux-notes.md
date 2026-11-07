# Quantity demo UX

Each ingredient line selects its original input unit. Conversion evidence is an optional
expanded section with ingredient-specific mass factor, source URL, version, uncertainty and
named spoon standard/volume. These are user declarations, not validated reference data.
Missing/invalid evidence blocks calculation and saving while keeping inputs. Numeric evidence
factors explicitly use canonical decimal points. Original quantities survive recipe reload.
A local g/kg display preference affects mass presentation and scaling previews, not saved input.
No product UI design or authenticated preference synchronization is included.

Matching ingredients offer an explicit curated reference portion selector. Source descriptions
remain original English text. Changing ingredient clears the selected conversion. Loaded
revisions retain their saved evidence independently of the current reference list.
