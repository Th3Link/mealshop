# Acceptance evidence

- AC 1–2: `grouping_preserves_all_exact_records_and_keeps_group_nutrition_unknown` checks all 313
  source records for exact serialized equality after excluding only their new group ID. All 73 groups
  have unknown nutrients/constituents and no copied education, allergens, tags or composition.
- AC 3: `catalogue-organization.spec.ts` exercises the 73 group rows, expansion of wheat flour,
  exact compound-spelling subtype search, detail navigation and mobile bounds. Source-code IDs are
  absent from organizational list rows. Counts distinguish ingredients from groups.
- AC 4: `python3 scripts/verify_catalogue.py` invokes the independent group verifier: every exact
  bundled ID occurs once, all group names have German translations, and previous flour groups retain
  their membership and stable IDs. Existing frozen-source verifications still pass.
- AC 5: 58 Rust tests and all nine real embedded Nextcloud browser workflows pass, including historical
  recipes, grouped snapshots, source conversions, optional constituents, role revocation, stale writers,
  native themes, reduced motion and source quantity precision. The oversized custom-catalogue test
  continues to reject an oversized snapshot at publication without lowering the loader limit.

First verify F26, then organize the catalogue: separate ordered commits in the same review PR. Final
immutable container rollout, rootless transport checks, restart integrity and current-head CI results
are recorded in PR #20. Screenshots were inspected at desktop/mobile sizes; the household field's
selected text is explicitly checked against its input bounds. No primary data or AGENTS.md rewrite.

`organizational_reverse_lookup_finds_legacy_recipes_without_rewriting_snapshots` additionally proves
that new groups discover older recipes using their current member IDs while historical files remain
byte-for-byte untouched. Lookup also retains pinned composition/group membership.
