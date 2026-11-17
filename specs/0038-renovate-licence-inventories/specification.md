# Automatic Renovate licence inventories

Status: Done
Priority: Must
Origin: Customer, 2026-11-17; recurrent inventory failures in dependency PRs
Milestone: M1

## User value

Review a dependency update and its matching Dart/native Rust licence inventories in one PR.
Ordinary version updates must not fail Android CI solely because an inventory was left behind.

## Acceptance criteria

1. Self-hosted Renovate refreshes both inventories after Cargo/Pub updates and before committing,
   once per branch, using the mobile client's pinned Flutter/Rust tools and locked dependencies.
2. Automatic refresh accepts version-only changes and removed dependencies. New packages, changed
   licence declarations/text hashes, repository identities, supplements or runtime notices require
   explicit review; neither inventory is published if either candidate fails that check.
3. Missing/unknown licence material, source hash mismatches, unlocked resolution and wrong tool
   revisions remain errors. Existing licence/advisory and committed-inventory CI checks stay enabled.
4. Only the exact refresh command is authorized; only the two inventory JSON files are included
   as post-upgrade artifacts. The task requires no Android SDK, emulator, signing key or app data.
5. Regression tests exercise guarded publication, SDK discovery outside the development directory,
   and tool/command restrictions. Repeating a successful refresh produces no further changes.
6. Repair PR #65 with the same refresh path, retain current main and verify current-head CI,
   including native Android integration and APKs. Document activation after merge and manual review.

## Links

[Plan](plan.md) · [Decisions](adrs.md) · [Evidence](validation.md) · [F01](../0001-bootstrap/specification.md)
