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
