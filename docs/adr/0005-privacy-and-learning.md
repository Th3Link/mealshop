# ADR 0005: privacy and learning

Status: Proposed safeguards for customer-requested learning
Date: 2026-11-07

## Context
Meal planning combines household attendance, dietary constraints, ratings and shopping behaviour. These can reveal sensitive personal information.

## Decision
Scope records and MCP tools to authorized households. Separate hard exclusions from soft preferences; unknown allergen data needs review. Store check-off/cooking/rating events with correction and deletion support. Start from explainable rules; evaluate opt-in incremental learning against that baseline. Keep manual overrides, history reset and opt-out. Calendar and external lookups disclose what leaves the server and request appropriate consent.

## Alternatives
Unconstrained collaborative training could expose health information or override exclusions. Cold-start ML alone has no useful evidence. Silent calendar writes exceed the request.

## Consequences and open questions
Authentication, retention, model choice, external APIs and consent UX remain open. Imported text/remote MCP content is untrusted data, not executable instructions.
