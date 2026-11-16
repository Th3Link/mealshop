# Tagged releases and Kubernetes installation

## Release artifacts

Pushing a stable `vX.Y.Z` tag starts [Release](../../.github/workflows/release.yml). Only commits
reachable from `main` are accepted. Regular PR/main builds never publish packages. Before tagging,
keep these versions equal: `Cargo.toml`, its `mealshop-server` entry in `Cargo.lock`, the version and
image-tag in `appinfo/info.xml`, and `version`/`appVersion` in `charts/mealshop/Chart.yaml`.
Flutter's independently versioned phone package is not changed by a server release.
After changing the Cargo package version, run `cargo check` to update its lockfile entry; do not
hand-edit `Cargo.lock`. Run `python3 scripts/release.py --tag vX.Y.Z` before creating the tag.

The workflow reuses CI, Android and security checks, then builds and smoke-tests native Linux AMD64
and ARM64 images. Each exact tested image is pushed before its digest enters the combined manifest:

- Image: `ghcr.io/th3link/mealshop:X.Y.Z` (also `:vX.Y.Z`; architecture tags have `-amd64`/`-arm64`).
- Chart: `oci://ghcr.io/th3link/charts/mealshop`, chart version `X.Y.Z`.
- GitHub release: chart archive, original `ExApp` manifest, registration helper, image digest and checksums.

The published chart pins the combined image digest. Local source-chart renders default to
`Chart.appVersion`; an explicit `image.digest` overrides a tag. OCI source/revision/version labels
identify the built code. `GITHUB_TOKEN` publishes without a personal access token; package writes
exist only on publishing jobs. New packages inherit repository linkage where supported, but **verify
both GHCR packages' visibility/access after first publication**. Public source alone does not make
packages public. For private packages, grant Actions package write access, configure an OCI pull
credential on Flux's `OCIRepository`, and supply `imagePullSecrets` for Kubernetes image pulls.

Example for the current source version, after merging this workflow:

```sh
python3 scripts/release.py --tag v0.1.0
git tag -a v0.1.0 -m "Mealshop 0.1.0"
git push origin v0.1.0
```

Use the next unused version; do not move published tags. A tag push initiates publication, not a
deployment to any cluster. Wait for the full Release workflow before referencing new packages.
A failed run can leave architecture images or the image published before the chart; re-running the
failed jobs completes the same tagged release. Do not create a second tag to hide a failed release.
The first real GHCR publication, `v0.1.0`, is verified in the
[F36 release evidence](../../specs/0036-release-images-helm/plan.md#first-release-acceptance--2026-11-16).

## Install with Flux

Prerequisites: existing Nextcloud 35 with `AppAPI` 35, a namespace shared with Nextcloud (the example
uses `nextcloud`), reachable HTTPS Nextcloud URL and persistent storage. The backend is already
built in the released image; no local Rust, Node or Docker build is required.

1. Create a dedicated installation secret with key `APP_SECRET`: 32–200 ASCII letters, digits,
   underscores or hyphens; a random 64-character hexadecimal value is suitable. Store it through
   your existing SOPS/external-secret mechanism. Never commit plaintext secrets or rotate this value
   independently of the `AppAPI` registration. For an existing installation, reuse its secret and PVC.
2. Copy [flux-example.yaml](../../deploy/helm/flux-example.yaml) into your `GitOps` repository, set
   your Nextcloud URL and existing secret name, choose the released chart version, and add both
   it and your encrypted secret to the existing Kustomization resources. The Longhorn labels are
   an example for the customer's current backup group, not a guarantee of a successful backup.
3. Review and merge your `GitOps` PR, then wait for `HelmRelease/mealshop` to become Ready.
4. Perform the one-time `AppAPI` registration below. No public backend ingress, Docker socket,
   `HaRP` installation or Kubernetes exec permissions are added by this chart.

Example SOPS secret generation in **your `GitOps` checkout**. Set `AGE_RECIPIENT` to its existing public
Age recipient; use your actual resource path and add it to the Kustomization. The plaintext passes
directly to SOPS rather than a terminal or committed file:

```bash
umask 077
set -euo pipefail
set +x
test ! -e apps/controllers/mealshop-secrets.yaml
python3 - <<'PY' | sops --encrypt --age "$AGE_RECIPIENT" \
  --encrypted-regex '^(data|stringData)$' --input-type json --output-type yaml /dev/stdin \
  > apps/controllers/mealshop-secrets.yaml
import json, secrets
print(json.dumps({"apiVersion":"v1", "kind":"Secret", "type":"Opaque",
    "metadata":{"name":"mealshop-appapi", "namespace":"nextcloud"},
    "stringData":{"APP_SECRET":secrets.token_hex(32)}}))
PY
```

Only do this for a first installation and a new file; do not overwrite an existing secret. Check
SOPS's exit status and encrypted output before staging it. On the customer's `hyrule` repository,
`apps-controllers` already decrypts SOPS and reads `apps/controllers/kustomization.yaml`.

For direct Helm use, create the same existing secret first, then:

```sh
helm upgrade --install mealshop oci://ghcr.io/th3link/charts/mealshop \
  --version 0.1.0 --namespace nextcloud \
  --set nextcloud.url=https://cloud.example.org \
  --set appApi.existingSecret=mealshop-appapi \
  --set persistence.storageClass=longhorn --wait
```

Use `helm pull ... --untar` if you need the matching chart's `files/registration.py` without cloning
the source repository. A separate PVC is created by default. `persistence.existingClaim` mounts
an existing claim and skips PVC creation; it must be writable by UID/GID 1000. Resource requests
and the default 10-GiB size are initial settings, not measured capacity guarantees.

## Register with the existing Nextcloud

These commands run from your trusted administrator shell. Set `NC_POD` to the existing Nextcloud
pod (the official chart's main container is named `nextcloud`). Change the namespace/container
where your installation differs. Use your real HTTPS address below; it must work from the Mealshop
pod and already belong to Nextcloud's `trusted_domains`. Never use a pod's `localhost` for Nextcloud.

```sh
export NC_POD=your-nextcloud-pod
occ() {
  kubectl -n nextcloud exec -i "$NC_POD" -c nextcloud -- \
    su -s /bin/sh -c 'exec php /var/www/html/occ "$@"' www-data -- occ "$@"
}
occ status
occ app:enable app_api
occ app_api:daemon:list
occ app_api:app:list
```

Install `AppAPI` from Nextcloud Apps first if absent. If `mealshop` is already registered, use its
existing daemon/secret/storage and the update procedure instead of re-registering. If the daemon
name below already exists, verify its settings: registering the same name does not update it.

Check startup and the actual Nextcloud-to-Service path:

```sh
kubectl -n nextcloud rollout status deployment/mealshop --timeout=300s
kubectl -n nextcloud exec "$NC_POD" -c nextcloud -- \
  curl --fail --silent --show-error http://mealshop.nextcloud.svc:23000/heartbeat
occ app_api:daemon:register mealshop_manual "Mealshop (Flux/Helm)" manual-install http \
  mealshop.nextcloud.svc https://cloud.example.org
```

The daemon host is the internal Service DNS name **without a port**; the registration provides
the configured port. A non-default release/fullname/namespace changes this host; `helm get notes`
shows the actual values. No default daemon is replaced. Cluster policies must permit this internal
HTTP path, DNS resolution and Mealshop's HTTPS callback to Nextcloud. TLS verification stays enabled.

The chart exposes a **public, secret-free** registration template. Combine it with the existing
Kubernetes secret in memory and stream it into Nextcloud. The example assumes a source checkout;
otherwise use the helper from the matching unpacked chart or GitHub release asset.

```bash
set +x
set -euo pipefail
umask 077
PUBLIC_REGISTRATION=$(mktemp)
kubectl -n nextcloud get configmap mealshop-appapi \
  -o jsonpath='{.data.registration\.json}' > "$PUBLIC_REGISTRATION"
kubectl -n nextcloud get secret mealshop-appapi -o json \
  | python3 charts/mealshop/files/registration.py "$PUBLIC_REGISTRATION" \
  | kubectl -n nextcloud exec -i "$NC_POD" -c nextcloud -- \
    su -s /bin/sh -c '
      set +x
      registration=$(cat)
      test -n "$registration" || exit 1
      exec php /var/www/html/occ app_api:app:register mealshop mealshop_manual \
        --json-info="$registration" --wait-finish
    ' www-data
rm "$PUBLIC_REGISTRATION"
occ app_api:app:enable mealshop
occ app_api:app:list
```

Use a shell with `pipefail` enabled so upstream failures are not hidden. A custom `appApi.secretKey`
is the helper's second argument. Do not print, tee or record the secret JSON stream. The secret is
not in the local kubectl command or Exec URL; it briefly exists in PHP's process arguments inside
the trusted Nextcloud container. No administrator password is required by the chart or backend.

Refresh Nextcloud, open Mealshop, create a household, save a recipe and reopen it. Heartbeat and
Ready only verify transport/startup; authenticated writes verify actual integration. The chart does
not claim automatic App Store registration or verified `HaRP` deployment.

## Updates, backup and recovery

Update the reviewed chart version in Flux. Each published chart carries its exact image digest.
Reuse the same secret and data claim. Keep `service.port`, the daemon host and release identity
stable; changing them requires an explicit matching `AppAPI` registration change. Rollbacks across
future data-schema migrations require a compatible backup; an old image alone cannot undo data writes.

Exactly one writer runs with Recreate and a 60-second graceful termination window. The PVC has
`helm.sh/resource-policy: keep` and is retained on ordinary Helm uninstall; never remove it to fix
an installation. Check actual backups and restore procedures, not only PVC labels. Back up its data,
the matching secret and Nextcloud's database/`AppAPI` registration together. Use a coordinated quiet
backup point; ongoing file writes and Nextcloud DB dumps are not an atomic combined backup.

An unclean kill can leave a fail-closed writer lock or pending transaction. Follow the existing
[Nextcloud recovery guide](nextcloud.md) and storage guides; never delete a live lock or force a
second writer. If a retained PVC is reused after reinstalling the same release, confirm its ownership
and binding; `persistence.existingClaim` makes reuse explicit. The chart never generates a replacement
secret automatically.

## Verification and upstream references

PR CI runs both native container architectures, release metadata/route checks, Helm 3/4
schema/render/package contracts and a disposable
Kind install/upgrade/uninstall test with an OCS fixture, in addition to existing transport tests.
The fixture is not a real Nextcloud deployment. The release workflow repeats all quality/security/
Android gates, tests both native image architectures, then publishes. The first real tag verified
registry permissions and publication for `v0.1.0`; the customer's cluster remains untouched.

- [GitHub GHCR publication and GITHUB_TOKEN](https://docs.github.com/en/actions/tutorials/publish-packages/publish-docker-images)
- [Helm OCI registries](https://helm.sh/docs/topics/registries/)
- [Flux OCIRepository](https://fluxcd.io/flux/components/source/ocirepositories/)
- [Nextcloud `AppAPI` manual-install resolution](https://github.com/nextcloud/app_api/blob/stable35/lib/DeployActions/ManualActions.php)
