# ADR 0003: persistence and data boundaries

Status: Customer storage targets accepted; schema and adapters proposed
Date: 2026-11-07

## Context
The same household data must work in Git/files, PostgreSQL, SQLite and MariaDB, standalone or in Nextcloud.

## Decision
Define versioned domain records with stable identities, explicit provenance, household ownership and recipe revisions before an adapter. Implement adapters incrementally with shared behavioural fixtures. Keep Git history/merge semantics separate from SQL transactions. Require backup/export/import validation and conflict handling. Do not select an ORM or create a generic repository trait in the bootstrap.

## Alternatives
SQL-only would exclude Git/files. A single SQL dialect assumed across all engines is unverified. Git does not substitute for authorization or concurrent-write control.

## Consequences and open questions
First adapter, offline sync, encryption, attachment storage and transaction boundaries need feature-level design. Plans/purchases/stock movements must not silently overwrite one another.
