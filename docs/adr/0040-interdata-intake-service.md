# ADR 0040: Interdata as a second workspace crate with a file buffer

Status: Accepted for F43 slice 1 (intake only); persistence to be revisited with the review slice
Date: 2026-11-28

## Context
The customer requested a separate Rust HTTP service that receives recipes and ingredients over
REST, buffers them and publishes only after half-automatic checking (F43). Its first slice only
accepts and buffers uploads. It holds no household data and must not share a process, secret or
data directory with the household server or the Nextcloud `ExApp`. The customer decided that
uploads are admitted by one operator-configured secret until registration exists, and that no
uploader identity is kept.

## Alternatives
- *A module or second binary inside `mealshop-server`.* No workspace change, but the service
  would ship inside the `ExApp` image and the Android core and share their dependency set.
- *Its own repository.* Clean separation, but a second gate, Renovate and licence setup for a
  few hundred lines, and the wire formats could drift from the recipe model unnoticed.
- *PostgreSQL from the start.* The review queue of a later slice will want queries and several
  writers. Intake alone needs neither, and a database adds a crate, a migration tool and an
  operated dependency before any acceptance criterion requires them.
- *Sharing model types with the server through a third crate.* Premature: the buffer stores
  uploads as sent; only the later check stage needs the server's recipe and catalogue types.

## Decision
- The repository becomes a Cargo workspace with the existing root package and one new member,
  `interdata/` (`mealshop-interdata`), a library plus binary. The existing gate already runs
  with `--workspace` and covers it. The `ExApp` image builds `--package mealshop-server` only.
- The crate uses only dependencies that were already locked for the server: `axum`, `tokio`,
  `serde`, `serde_json`, `subtle` and `url` (so far transitive through `reqwest`). No new
  third-party package enters `Cargo.lock`.
- Submissions are stored as one immutable JSON record per upload in a directory, written with
  create-new semantics and synced. The request body is kept verbatim as a string beside the
  record metadata. The idempotency key is a restricted, file-name-safe token mapped to the
  record id; the record is written before its key so a crash never leaves a dangling key.
  One process owns the directory; a mutex serialises writers.
- Admission is a bearer secret of at least 32 bytes, compared in constant time and never
  logged. The service refuses to start without it. It identifies nobody.
- Without accounts, abuse bounds are global: body size, a per-minute ceiling for new
  submissions and a buffer capacity. Replays are exempt.
- Image links are accepted only on one configured HTTPS origin and are never fetched.
- Intake validates shape and bounds only. Amounts are exact decimal strings; JSON numbers are
  rejected. Tags are syntactic ids at this stage; vocabulary, ingredient matching, unit
  normalization and nutrition belong to the check stages.

## Consequences
- `cargo` commands without `--package` now build both crates; `scripts/release.py` and the
  Android core fingerprint read the unchanged root package.
- A crash between record and key leaves an unreferenced record; a retry then creates a second
  submission. Reviewers may see such a duplicate; nothing is lost.
- A global rate limit lets one uploader exhaust the minute for everyone. Acceptable while a
  single shared secret is the only admission; per-uploader limits need the registration
  decision.
- The file buffer suits one instance. Review, publication and queries (slice 3) need a
  persistence decision in a follow-up ADR before they are built.
- No container image, Helm chart or DNS exists for the service yet.
