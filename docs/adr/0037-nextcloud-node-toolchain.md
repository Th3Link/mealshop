# Keep Node toolchains within Nextcloud compatibility

Status: Accepted for customer-requested build repair
Date: 2026-11-24
Scope: F01/F22/F38, PR #81

## Context

PR #81 selects Node 26 for the Vue/MCP jobs, application engine and web image.
The locked and latest published `@nextcloud/auth` 2.6.0, Axios/router libraries and
`@nextcloud/vue` 9.14.1 support Node 20/22/24, excluding 26. Strict npm installation
therefore fails before tests in both jobs. The native Nextcloud 35 integration takes
precedence over an unsupported build-runtime update.

## Alternatives

- Disable engine checks or rewrite third-party engine declarations.
- Change Nextcloud components or maintain forks without a supported release.
- Keep the supported Node series, remove CI version duplication and enforce the boundary.

## Decision

Retain the existing Node 24 engine and digest-pinned Alpine web image. Both npm-consuming
CI jobs use setup-node's `node-version-file` with the client package manifest, making
`engines.node` the canonical CI version. Keep the exact npm pin, ignored install scripts,
strict engines, TypeScript/Vue checks and security gates unchanged.

Restrict Renovate's Node-version and official Node Docker updates to the 24 series.
Use its supported regex filter for numeric versions and Docker's distro-suffixed tags;
patch/digest updates remain eligible. No other Docker images, runtimes or packages are held.
This repairs the incompatible update; it does not claim a completed Node 26 migration.

The regression uses Renovate's real version/filter implementation and YAML parser to
check version/tag acceptance and rejection, all locked Nextcloud engine constraints,
manifest/lockfile agreement, the pinned image series and both CI consumers. It rejects
literal CI versions which would override the manifest. Mount only the named public
configuration files into the isolated, network-disabled test container.

## Consequences

The supported build remains reproducible without relaxing installation safeguards.
Reopen the major boundary after all required Nextcloud libraries support it and strict
installation, both compiler passes, Vue/MCP tests, schema verification and AMD64/ARM64
images pass. Keep native Nextcloud compatibility evidence when component versions change.
Main and PR-specific CI results are recorded separately in the PR verification record.

References: [setup-node version files](https://github.com/actions/setup-node/blob/v7/docs/advanced-usage.md#node-version-file),
[auth engines](https://registry.npmjs.org/@nextcloud/auth/2.6.0),
[Vue engines](https://registry.npmjs.org/@nextcloud/vue/9.14.1),
[Renovate version filters](https://docs.renovatebot.com/configuration-options/#allowedversions).
