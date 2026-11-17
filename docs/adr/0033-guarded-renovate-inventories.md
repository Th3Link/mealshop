# Guarded Renovate licence inventories

Status: Accepted
Date: 2026-11-17
Scope: F01/F35/F38, dependency update delivery

## Context

Locked Cargo/Pub updates repeatedly left the committed Android licence inventories stale,
blocking otherwise valid PRs. Inventories also record reviewed licence identities and source hashes;
automating an unconditional approval would weaken that review boundary.

## Alternatives

- Continue manual regeneration for every version update.
- Regenerate during verification, potentially masking a stale committed inventory.
- Add a privileged PR-repair workflow with a separate commit/token lifecycle.
- Use self-hosted Renovate's pre-commit post-upgrade task with guarded publication.

## Decision

Use the existing self-hosted runner and token. Render its exact allowed command, mobile tool
constraints and two JSON artifact paths from repository policy. Run once per Cargo/Pub branch.
Validate the pinned Flutter revision and Rust version, resolve only locked inputs, then collect both
inventories. Permit only version changes and removed packages with identical reviewed identities.
Validation failure publishes neither candidate. New packages, changed licence/repository/text hashes,
supplements and Rust runtime notices retain manual review.

Keep the existing committed-inventory, source-hash, MSRV and licence/advisory gates independent.
Do not authorize shell command composition, Android setup or access to application data.

## Consequences

Ordinary version-only updates include their inventories in the same Renovate commit. Review stops
remain visible as artifact/CI failures with explicit manual recovery. The policy activates on the
scheduled runner after merge. Synthetic fixtures and read-only simulation can verify it without
running a live write-enabled update against unrelated PRs.

References: [post-upgrade tasks](https://docs.renovatebot.com/configuration-options/#postupgradetasks),
[allowed commands](https://docs.renovatebot.com/self-hosted-configuration/#allowedcommands),
[F38 evidence](../../specs/0038-renovate-licence-inventories/validation.md).
