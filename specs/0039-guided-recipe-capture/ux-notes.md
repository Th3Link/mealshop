# UX notes

Principles for this feature, applied to the Nextcloud client only:

- **Bring what you have.** The first screen asks how the recipe arrives, not for fields.
- **One review, then done.** Every channel ends in the same editor, prefilled. The cook's original
  line stays above each ingredient until saving.
- **Ask one thing at a time.** A spoon or piece asks "what does 1 tbsp weigh?" — one field.
  Source records, spoon standards and versions sit behind "Use a source record instead".
- **Never guess silently.** Unclear amounts stay as text, unknown ingredients stay unselected with
  candidate chips; estimated and incomplete nutrition is labelled in words, not colour alone.
- **Reward, not bookkeeping.** Cards with pictures and stars; rating is two clicks from the recipe.

Layout: Nextcloud components and theme variables only, no custom fonts or colours. Cards collapse
to two columns on phones; capture stacks picture, text and preview. Targets are at least 44 px.
Stars are a radio group with a separate clear button; averages carry a text alternative.

Known rough edges kept for later slices: the step editor still shows all timing fields; optional
details are not collapsed; planning, shopping and inventory keep their current layout.
