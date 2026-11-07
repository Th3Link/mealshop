# ADR 0009: local recipe revisions and pinned nutrition inputs

Status: Proposed implementation for the customer-authorized recipe slice; review pending
Date: 2026-11-07

## Context
The customer accepted PR #5 and requested creating/saving recipes with catalogue ingredients
and nutrition per serving. F04 needs a usable persistence slice before the broader F18 adapters,
authentication, Git synchronization or product clients are ready.

## Decision
Start with a dedicated local directory, one household per server, without additional crates.
Each immutable schema-1 JSON revision includes a stable recipe ID, monotonically increasing
revision, author-supplied title/language/author, exact gram strings, batch serving count,
preparation flag, ordered instructions with active/passive seconds, and the full referenced
catalogue subset/version. An empty instruction list is allowed for drafts. User text is
retained in its declared en/de language; coded fields/messages remain English identifiers.
Author is attribution text, not an authenticated identity. This slice supports locally authored
recipes only; imports and their sharing constraints remain F16.

POST creates a recipe; PUT requires the expected latest revision. GET retrieves latest or
an explicit historical revision; listing returns latest summaries. Nutrition is calculated
from the saved snapshot, retaining unknowns, provenance and definition-specific partial totals.
A new revision deliberately uses the current server catalogue; old revisions remain unchanged.
Changing servings repartitions the same batch; proportional ingredient scaling is deferred.

Use exclusive `create_new` directory locking for the application lifetime, plus an in-process
mutex. Filesystem operations run on blocking workers. Write a private `.pending` file, sync
its contents, publish using a same-directory hard link (fails if a target already exists),
sync the directory on Unix, remove `.pending`, then sync the directory again. Only acknowledge
success after publication and synchronization. An ambiguous I/O failure makes further writes
unavailable until operator recovery; no silent overwrite or success response. Load validates
all records and consecutive histories before accepting traffic. Linux/local filesystems with
hard-link and fsync support are the initial deployment target; network filesystems, external
writers and cross-platform crash durability have not been validated.

The executable only binds loopback until authentication exists. Recipe responses use
`Cache-Control: no-store`. The embedding library explicitly documents its trusted-client
boundary. New directories/files use Unix 0700/0600 (existing directory permissions stay intact).
Limits: 128 KiB request bodies, 100 ingredient lines, 100 steps, 1–1000000 integer servings,
1000 total revisions and 2 MiB per persisted revision. The catalogue loader bound stays 1 MiB.
This intentionally bounded in-memory revision index must be replaced/expanded under F18.

## Alternatives
Browser localStorage would not satisfy server persistence or reproducible source retrieval.
Starting all SQL/Git adapters together would delay the requested authoring slice. A mutable
single JSON file would obscure revision conflicts/history; Git commits alone do not provide
application-level transactions or household authorization. No repository trait/ORM is needed.

## Consequences
An exclusive lock survives crashes as a stale file and needs deliberate operator recovery.
Backups/restore, ambiguous-write recovery and startup failures are documented in the
[recipe guide](../guides/recipes.md). No automatic Git commit, deletion, synchronization,
recipe translation, authentication or production web design is included. F04/F18 remain
In progress until their broader contracts and customer acceptance are complete.
