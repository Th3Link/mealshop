# F09 local household slice

- [x] Separate local operator access and protected profile storage (AC 1, 4).
- [x] Record allergy/intolerance notes, hard exclusions and soft dislikes separately (AC 1).
- [x] Group review unions constraints, resolves known aliases and preserves unknowns (AC 2–3).
- [x] Revision-aware correction, authorized JSON export and member removal (AC 4).
- [x] Add pragmatic keyed de/en demo flow and operator/recovery documentation.
- [x] Complete repository gate and review PR; customer acceptance pending.

Tests in tests/household.rs cover unauthorized reads/writes, conflicts, aliases, hard/soft
separation, unknowns, restart/export/removal, invalid data and interrupted/corrupt storage.
Full allergen mapping and hosted identity integration remain future work; unblocked candidates
always require review. No learning component exists that can relax a hard constraint.

Validation: 35 Rust tests, six locale tests, catalogue and conversion verifiers, fmt/clippy,
licenses, audit, strict rustdoc and MSRV 1.85 passed locally. Browser confirmed unauthorized
access rejection, authorized profile save and hard-exclusion blocking.
