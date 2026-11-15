# ADR 0031: Tagged OCI images and a Flux-owned `ExApp`

Status: Accepted implementation direction for the customer request, 2026-11-15.

## Context

The customer operates Nextcloud 35 through `FluxCD` and requests automatic GHCR publication on
stable `vX.Y.Z` tags plus a Helm chart. Requiring local image builds is unsuitable for that workflow.
The application already supports authenticated `AppAPI` with a manually managed TCP service.

## Alternatives

- Keep local builds and hand-written manifests: repeats operator work for each update.
- Let `HaRP` deploy Kubernetes resources: adds another resource owner and daemon configuration.
- Publish an OCI application chart and let Flux own its workloads: fits the existing operator model.

## Decision

Publish versioned multi-architecture container images and a matching OCI chart from tag-triggered
GitHub Actions after reusable CI/security gates. Require matching source/package versions rather
than publishing a binary that reports a different version. Build and smoke-test on native runners.
Use `GITHUB_TOKEN` with package writes limited to publishing jobs and source revision labels.

The chart uses a supplied existing secret, an internal `ClusterIP` service and a separate retained PVC.
It fixes replicas to one and uses Recreate because file stores require an exclusive writer. Rootless
containers use a read-only filesystem, writable temporary storage and heartbeat probes. Nextcloud
continues to authenticate browser traffic through `AppAPI`. Its one-time `manual-install` registration
is an explicit operator action; no cluster-exec RBAC, Nextcloud administrator credentials or `HaRP`
installation are added to the chart. The public route projection is checked against `appinfo/info.xml`.

## Consequences

Tag creation is the publication action; merging a PR never publishes a release. Existing GHCR package
visibility/access may require an owner action on first publication. Immutable deployment digests and
chart versions support reviewed `GitOps` upgrades. Backups must retain app data, the matching secret and
Nextcloud's `AppAPI` registration together. Recreate causes a short update interruption and cannot make
forced termination or stale-writer recovery safe. Runtime/CI verification is distinct from the first
real registry publication and acceptance on the customer's cluster.
