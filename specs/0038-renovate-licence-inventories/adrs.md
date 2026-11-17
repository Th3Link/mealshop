# Decisions

Cross-cutting policy: [ADR 0033](../../docs/adr/0033-guarded-renovate-inventories.md).

## Use self-hosted post-upgrade tasks

Run before Renovate commits, rather than rewriting inventories during the verification job.
The existing committed-inventory checks remain independent. Derive tool constraints from
`clients/flutter/toolchain.json` when rendering the runner configuration. Restrict the command
and artifact paths in that runner configuration. No privileged PR-triggered workflow is needed.

## Preserve reviewed licence identity

Automatically carry forward reviewed metadata only when package versions change or packages
disappear. Changes to licence material, new packages and Rust runtime notices need manual review.
Validate both candidates before publishing either. Explicit `--write` review commands remain
available; the automatic path never uses them to approve unknown material.

Sources: [post-upgrade tasks](https://docs.renovatebot.com/configuration-options/#postupgradetasks),
[command authorization](https://docs.renovatebot.com/self-hosted-configuration/#allowedcommands),
and the official Renovate/Containerbase implementations inspected for tool installation behavior.
