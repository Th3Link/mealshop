# ADR 0001: stack and delivery

Status: Accepted for bootstrap; future integrations planned
Date: 2026-11-07

## Context
The customer requests Rust/Axum, a Flutter app, Nextcloud `ExApp` deployment and the homeautomation CI baseline.

## Decision
Use a single Rust package with Tokio/Axum and a small reusable router plus executable. Declare Rust 1.85 (edition 2024 baseline) and verify it in CI. Ship only /health now. Publish embedded project docs through rustdoc and Pages. Adapt the upstream workflows to a host binary, keeping build, coverage, MSRV, SPDX SBOM, audit, licences, Renovate and size metrics.

## Alternatives
Separate crates now would add unused boundaries. A different backend or frontend stack would contradict the request.

## Consequences and open questions
Flutter, Nextcloud `AppAPI` authentication and packaging need later integration specs. A healthy process is not proof of database readiness. Metrics are binary size, not latency or nutrition accuracy.
