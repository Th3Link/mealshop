# Delivery plan

- [x] Record the customer request and deployment boundary (AC1–6).
- [x] Implement version validation and an existing-secret Helm chart (AC1, AC4).
- [x] Add reusable quality gates and native GHCR release jobs (AC1–3).
- [x] Document the Flux/registration and recovery path (AC5).
- [x] Run chart contracts, isolated runtime verification and repository gates (AC4, AC6).
- [x] Prepare one reviewable delivery with local evidence and publication limits.

## Verification evidence

Local evidence, 2026-11-15:

- AC1/AC4: 14 contracts pass on Helm 3.22.1 and 4.3.2. Invalid/missing configuration, version/route
  drift, unsupported tags, retained/existing PVCs, digest precedence, registration and package rendering
  are checked. Removing the tag guard makes its negative test fail.
- AC2/AC4: the built AMD64 image passes the existing isolated rootless TCP and Unix transport suite,
  including lifecycle/assets/auth/CSRF, household features and private storage. Both native image
  architectures are added to PR CI and repeated before release publication.
- AC4/AC6: `python3 scripts/test_helm_runtime.py --image mealshop.local/mealshop:release-verification`
  passes on a new disposable Kind cluster with its own kubeconfig. Real install and Recreate upgrade
  retain the synthetic household; authentication/CSRF are enforced and uninstall retains the PVC.
  The fixture implements OCS, not a real Nextcloud installation; the cluster is removed afterward.
- AC5: `bash -n` passes all eight README/operator-guide shell blocks. Actionlint 1.7.12 passes every
  workflow. The documented image-manifest digest command is verified against a public registry manifest.
- All prescribed catalogue/i18n/fmt/clippy/nextest/license/audit gates pass, including 207/207 Rust
  tests; strict generated-documentation/rustdoc checks pass. `AGENTS.md` remains unchanged.

Remote CI evidence belongs to the pull request's check runs. This feature stays In progress until the first
actual stable-tag publication proves GHCR permissions and complete image/chart/release delivery.
No tag, registry publication or customer-cluster change is performed as part of testing this PR;
the customer publishes by pushing a release tag after review.

## First release acceptance — 2026-11-16

The customer merged PR #59 and explicitly authorized publication. Annotated tag `v0.1.0` targets
merge commit `b0c19e6be76a03c04aa9eb1a4f4d5fdd75eaece5`.
[Release run 43770118038](https://github.com/Th3Link/mealshop/actions/runs/43770118038) succeeded,
including both native image builds, reused CI/security/Android gates, OCI chart and GitHub assets.
[Published release](https://github.com/Th3Link/mealshop/releases/tag/v0.1.0).

- Anonymous registry requests verify tags `0.1.0` and `v0.1.0` have the same image index digest
  `sha256:8183133a5ddbed9a96e84567849506998c4f30524532edf710a48f59993d2f9d`.
- Its children are Linux AMD64 (`sha256:6dfbef9c3c1107b7430b3e335b200e0a869c0554b60836f2190685aab180bd1c`)
  and ARM64 (`sha256:cadbef53e3b0522699aa7f86bf6a97abe2a49d9c4eb01a638c9dc17230928a5d`).
  Their configuration blob hashes, architecture, version, source and merge-revision labels match.
- Anonymous `helm pull oci://ghcr.io/th3link/charts/mealshop --version 0.1.0` succeeds; chart digest
  `sha256:641fa866d0067d4b4affb9f8272bfdb5c9224cb63176fec6ec4f26841c3c2807`.
  The archive is byte-identical to its GitHub release asset; all four published checksums pass.
- Published chart/app versions are `0.1.0`; its image value pins the verified combined index and
  rendering uses exactly that digest. The actual GHCR image was pulled and passed the full isolated
  rootless TCP/Unix transport suite again. No customer Nextcloud or `GitOps` deployment was modified.

F36 is Done for its artifact-delivery contract. Customer-cluster installation remains a separate
operator action, and does not imply completion of F18's other adapters or standalone authentication.
