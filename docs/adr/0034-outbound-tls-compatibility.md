# Preserve outbound TLS behavior across reqwest 0.13

Status: Accepted for dependency maintenance
Date: 2026-11-22
Scope: F01/F17/F21/F35/F37, PR #73

## Context

The customer requested maintenance of the existing dependency PRs. Reqwest 0.13 removes
`rustls-tls`; its replacement selects AWS-LC and platform certificate verification. The
Android in-process core does not initialize that verifier's JNI integration. Our existing
0.12 configuration uses ring and Mozilla's bundled trust anchors on every platform.

## Alternatives

- Adopt the new defaults and introduce platform/JNI initialization and another native
  crypto toolchain during a dependency update.
- Keep reqwest 0.12, postponing the requested update.
- Configure the existing ring and Mozilla trust store explicitly with supported APIs.

## Decision

Use `rustls-no-provider` and promote the already used `rustls` and `webpki-roots` crates
to direct dependencies. One internal client-builder function supplies a preconfigured
Rustls client with ring, safe TLS protocol versions and the bundled Mozilla roots.
No global provider installation, system-root override, JNI initialization or new service
is required. Use it for Nextcloud, public recipe retrieval and product lookup, retaining
each caller's existing redirect, proxy, DNS pinning, timeout and authentication policies.

## Consequences

Rustls must remain compatible with reqwest's preconfigured-backend API. Locked Android
licences are reviewed and refreshed, and MSRV/native CI remain required.
The new `webpki-root-certs` 1.0.12 data has the same reviewed CDLA-Permissive-2.0
licence as `webpki-roots`; admit that exact package/version without widening the global list.
Loopback TLS tests use a synthetic test authority to prove trusted success and rejection of unknown
authorities, wrong hostnames and expired certificates. They never add roots to production.
No certificate-validation bypass or advisory exception is introduced.

References: [reqwest 0.13 release](https://seanmonstar.com/blog/reqwest-v013-rustls-default/),
[preconfigured TLS](https://docs.rs/reqwest/0.13.5/reqwest/struct.ClientBuilder.html#method.tls_backend_preconfigured),
[Android verifier integration](https://github.com/rustls/rustls-platform-verifier#android).
