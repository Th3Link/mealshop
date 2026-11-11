# ADR 0018: Explicit homemade ingredient inputs

Status: Accepted for F33

## Context
A catalogue food such as toasted bread may be purchased or made. Treating every finished ingredient
as a purchase is incorrect; guessing raw ingredients, yield or availability from its name is unsafe.
Existing recipe-owned parts already hold measured ingredient lines, instructions and dependencies.

## Alternatives
A global non-purchasable catalogue flag cannot represent different recipes. Simply omitting a homemade
item from shopping loses its base requirements. New nested independently saved recipes would add a
separate revision/yield model outside this slice.

## Decision
Add optional `prepared_part` to a finished recipe ingredient line, referencing an existing part with
instructions and measured transitive inputs. It is a declared output reference, excluded from input
composition and purchasing; its exact original quantity/identity stay preserved. Base lines count
once. Pin all identities; review all conservatively for dietary constraints. Reject unresolved refs,
cycles and ambiguous multiple outputs. Stateless nutrition accepts resolved measured inputs only.
Cooking derives the producer dependency. Legacy absence means purchased as before.

## Consequences
No inferred roasting recipe, output/input ratio, nutrient retention or water loss. Estimates explicitly
mark cooking unmodelled. Simple unshared preparation removal is confirmed; nested/shared work needs
explicit dependency resolution. Older binaries may reject new saved fields: back up complete data
before installation; rollback restores the matching image and snapshot. No endpoint/crate is added.
