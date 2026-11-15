# Tagged container and Helm releases

Status: In progress
Priority: Must
Origin: Customer, 2026-11-15
Milestone: M1

## User value

Install Mealshop in a Flux-managed Nextcloud without building or publishing containers manually.

## Acceptance criteria

1. A pushed canonical stable tag `vX.Y.Z` validates matching Cargo, lockfile, `ExApp` and chart
   versions, then runs the existing CI and security gates before publication. Other events cannot publish.
2. The workflow publishes tested native Linux AMD64 and ARM64 images and their combined versioned
   manifest to GHCR with the repository-scoped `GITHUB_TOKEN`, retaining source revision labels.
3. The same release packages and publishes `mealshop` as an OCI Helm chart to
   `ghcr.io/th3link/charts/mealshop`, and attaches its archive and manifest to a GitHub release.
4. The chart requires the real Nextcloud URL and an existing app secret, exposes only an internal
   service, retains persistent storage, supports an existing PVC and private image pull secrets,
   and enforces one rootless writer with Recreate updates and working heartbeat probes.
5. A Flux example and operator guide describe first installation, one-time `AppAPI` registration,
   package visibility, version bumps, upgrades and data recovery without exposing credentials.
   The chart does not gain permission to execute commands in the existing Nextcloud workload.
6. PR CI validates chart rendering and rejects invalid/missing configuration. Isolated verification
   exercises chart startup, authentication boundaries and persisted data across a normal upgrade.
   Publication and customer-cluster checks are reported separately from local verification.

## Scope

This extends F18 AC3 and N08. SQL adapters, hosted authentication, App Store admission, automated
`HaRP` deployment and changes to the customer's Kubernetes cluster remain outside this delivery.

## Links

[Plan](plan.md) · [Decisions](adrs.md) · [F18](../0018-storage-deployment/specification.md)
