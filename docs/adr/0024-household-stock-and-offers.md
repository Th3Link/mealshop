# ADR 0024: atomic household stock, reservations and manual offers

Status: Accepted
Date: 2026-11-12

## Context
F15 links stock, shopping and completed cooking without treating a plan or check-off as physical
consumption. Existing shopping.json already atomically publishes demands and immutable purchases.
Separate independently written stock/shopping files would allow double coverage after a crash.

## Alternatives
A second stock journal needs cross-file recovery; automatic purchase/cooking side effects hide review.
A generic transaction framework or new database/crate is unnecessary. External offer providers introduce
unapproved licensing/privacy contracts. Universal mass/volume/count conversion invents evidence.

## Decision
Add optional/default-empty F15 state to the existing versioned private shopping snapshot, within its
existing 4 MiB publication boundary. Reuse fixed-point amounts and g/kg, ml/l, piece dimensions. Keep
original quantities and explicitly resolved ingredient/product identities; unmatched lots never match
recipes by name. Manual offers retain their supplied source and exact package/price/currency.

All F15 changes have a no-write preview and explicit reviewed confirmation with a stable actor/key,
expected shopping revision and immutable command receipt in the same atomic publication. Identical
retries survive restart; changed key payloads conflict. The current writer role is checked on every
call. Locks follow profiles -> recipes -> cooking -> ingredients -> shopping; source stores are read
only and independent completed actions are never silently reversed.

Reservations reduce proposals without debiting stock. Source changes release obsolete holds; used
allocations remain linked to their original source identity. Purchase receipts enter stock only when
reviewed. Completed cooking consumes reviewed matching lots, optionally using explicitly selected
compatible reservations; unbooked input remains visible. Reversals append opposite physical movements
and release the reversed use's coverage; they cannot make stock negative or steal another hold.

Offer entry is manual. Adding it to shopping rechecks validity on the explicitly selected shopping
date and current intended household profiles; known exclusions block it and unknown coverage stays
unknown. No source claims, nearby food matches, substitutions, scraping or automatic purchases.

## Consequences
Old shopping files remain readable; new F15 snapshots require the current reader on downgrade.
Preserve the entire shopping file and its receipts in backup/recovery. Existing single-writer locks,
private permissions, fsync/rename and fail-closed pending-write recovery remain authoritative.
No retained private household inspection, data reset, new runtime dependency or provider is required.
