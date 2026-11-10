# ADR 0006: durable process schedules

Status: Proposed shared process model
Date: 2026-11-07

## Context
Cooking, multi-day dough and brewing need timers, dependencies, phase reminders and equipment-specific variants.

## Decision
Model a process instance as versioned steps with dependencies, active/passive durations, target windows, equipment and persisted deadlines/completion. A server scheduler dispatches idempotent reminders with acknowledgement state. Clients show delivery limits and recover deadlines on reconnect. Preview downstream effects when delays or target times change. Store absolute instants with intended local timezone for civil-time scheduling.

## Alternatives
A transient in-app countdown cannot survive restarts or multi-day schedules. A separate scheduler per food type duplicates dependency logic. Automatic equipment control is beyond the request.

## Consequences and open questions
Notification providers, offline authority, retries and daylight-saving semantics need tests. Fermentation duration and notification delivery remain uncertain; no guaranteed outcome or safety claim.

## Native Nextcloud implementation
F12/F13 delivery decisions and acceptance evidence are now recorded in
[ADR 0016](0016-durable-cooking-and-civil-time.md). This earlier shared proposal remains
open for brewing, other clients and notification transports outside that verified scope.
