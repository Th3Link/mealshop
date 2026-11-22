# Keep password backups on the audited age series

Status: Accepted for customer-requested dependency repair
Date: 2026-11-22
Scope: F01/F35/F37/F38, PR #72

## Context

Renovate repeatedly proposes age 0.12 even though F37 deliberately uses 0.11.5.
Published age 0.12.0 and 0.12.1 unconditionally enable HPKE/P-256, pulling in
`elliptic-curve` 0.13 and `crypto-bigint` 0.5.5. RUSTSEC-2026-0387 requires
`crypto-bigint` >=0.7.6; that API cannot replace the required 0.5 series in place.
Our password-encrypted standard age v1 backups need none of the added recipient types.

## Alternatives

- Ignore the advisory to accept the version bump.
- Maintain a cryptographic fork/backport or replace the backup implementation.
- Preserve the audited implementation and encode its compatibility boundary in Renovate.

## Decision

Keep age 0.11.5 with default features disabled and restrict only Cargo updates of age
to versions below 0.12.0. Continue checking patch updates within the supported series;
all other dependencies retain the normal update policy. This is a compatibility repair,
not a completed age 0.12 upgrade. No advisory exclusion or encryption-format change is made.

Reopen the boundary only after an upstream graph uses patched arithmetic, its sources
and licences are reviewed, the declared Rust/Android toolchains build it, and the full
backup tests prove cross-version restore and failed-restore preservation. The retained
synthetic 0.12-produced archive remains a mandatory compatibility fixture.

## Consequences

The existing tested password-backup implementation stays available without reintroducing
known vulnerable arithmetic. Renovate no longer recreates the same incompatible update.
Additional recipient types remain outside this feature. CI still runs the full advisory,
licence, host and native Android gates. The PR title and verification record describe
this compatibility hold explicitly.

References: [advisory](https://rustsec.org/advisories/RUSTSEC-2026-0387.html),
[age 0.12.1 dependencies](https://crates.io/crates/age/0.12.1/dependencies),
[Renovate version rules](https://docs.renovatebot.com/configuration-options/#allowedversions).
