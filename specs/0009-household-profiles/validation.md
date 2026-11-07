# F09 completion evidence

Status: Done for the local household contract; customer requested completion in PR #12.

## Acceptance mapping
- AC 1: separate structured hard allergen IDs, allergy notes, intolerance notes, hard ingredient
  exclusions and dislikes. Operator/read-only credentials enforce all API access; unknown
  credentials are denied and readers cannot edit/remove. Legacy profiles load new fields empty.
- AC 2: selected attendees' exclusions and positive allergen matches are unioned. Known names
  and aliases resolve conservatively. Dislikes never remove exclusions. Matches retain member,
  ingredient, group, source and reference version. No learning writer exists for profiles.
- AC 3: unknown ingredients, unstructured notes, incomplete positive-only allergen coverage and
  cross-contact never yield a safe result. Unblocked results always require review. Reference
  classifications apply only to matching catalogue source record IDs and states.
- AC 4: authorized correction, revision conflicts, JSON export, removal and restart are tested.
  Reader write/delete attempts fail; interrupted/corrupt storage fails closed without replacing
  good state. Backups/exports are outside subsequent server deletion, as documented.

## Automated evidence
36 tests passed with `cargo nextest run --workspace --locked`. Household acceptance tests
cover existing access/restart/removal cases plus structured milk/soy constraints for multiple
attendees, milk identity in lactose-free milk, read-only profile export/evaluation, denied
writes/deletion, unknown members/data and structured-field persistence. Six locale tests pass.
Catalogue/import/conversion checks and `scripts/verify_allergens.py` pass (106 positive
identity classifications, 14 groups). Fmt, clippy, licenses, audit, strict rustdoc and MSRV
1.85 pass. Current-commit CI links are recorded in PR #12.

## Browser evidence
An operator created a demo member with the milk allergen group. Evaluating milk in the
current ingredients returned blocked. Loading profiles with the separate reader credential
worked, while attempting to save changes was rejected and inputs remained available.

## Bounds
Done applies to local household profiles and dietary review, not hosted account management,
medical diagnosis, universal allergen coverage or guaranteed absence of cross-contact.
Data coverage can grow independently without relaxing hard constraints or unknown handling.
