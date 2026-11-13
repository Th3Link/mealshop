# MealShop — Conceptual domain model

Status: conceptual target, not a database migration proposal. Align with existing Rust/Nextcloud entities and APIs before changing schemas.

## Entities
| Entity | Key responsibilities |
|---|---|
| Household | Shared planning, membership, preferences and permissions |
| PersonProfile | Individual diet, goals, favorites, health-related optional rules |
| FoodRule | Hard exclusion/allowance, provenance conditions, or soft preference; source and scope |
| AttendancePattern | Weekday × meal attendance and default times |
| AttendanceOverride | Specific date/meal exceptions |
| EatingWindow | Optional time-restricted meal window and effective dates |
| PrepPreference | Per-meal fresh/batch/mixed strategy and storage limits |
| Ingredient | Canonical food concept and normalized units |
| Product | Specific trade item/GTIN, pack size, labels, nutrition source |
| Recipe | Ingredients, instructions, portions, timings, source and revisions |
| RecipeComponent | Ingredient or nested recipe with quantity and yield conversion |
| MealPlan | Planning horizon, status, generated explanation and revision |
| MealSlot | Date, meal type, dish, participants, servings, state |
| PreparedBatch | Recipe yield, preparation time, storage, expiration guidance, allocations |
| PantryLot | Ingredient/product or prepared batch, amount, location, acquired/prepared date |
| ShoppingList | Household list and sync revision |
| ShoppingItem | Requested ingredient/product, amount, state, store/category |
| ShoppingContribution | Source of demand: meal slot, recipe component or manual request |
| PrepTask | Scheduled actionable preparation step, dependencies and notifications |
| CookingSession | Active step, timers and progress |
| ActivityEntry | Person, type, duration, optional intensity and estimated energy |
| ImportJob | Source, extraction status, uncertainty and user review state |

## Modeling invariants
1. Hard food constraints are never relaxed by ranking/optimization. Explicit permissions do not override unrelated allergies.
2. Separate `allowed`, `excluded`, `disliked`, `preferred`, and `unknown`. Origin conditions (e.g. own eggs) require matching inventory provenance or explicit confirmation.
3. Diet labels are presets, not the authoritative storage of all constraints.
4. Meal attendance is per person and meal, with recurring patterns and date overrides.
5. Recipe components may reference other recipes. Reject cycles and define yield/quantity conversions.
6. Prepared batches are physical inventory, not duplicate planned meals; allocations cannot exceed available safe quantity.
7. Shopping demand retains contribution provenance; aggregation is a view, not irreversible loss of sources.
8. Checked/purchased/manual shopping items survive plan recomputation unless user explicitly resolves them.
9. Unknown OCR quantities and nutrition values are nullable/uncertain, never silently treated as zero.
10. Imported content retains source attribution and review status.
11. Sensitive health fields have explicit authorization boundaries and deletion/export behavior.
12. Shared edits use stable identifiers, revision/conflict handling and idempotent operations.

## Planning contract (conceptual)
Input: household + persons + effective food rules + attendance + meal windows + meal-prep settings + available recipes/ingredients + pantry (if enabled) + planning horizon.

Output: versioned proposal of meal slots, participants, servings, prepared-batch tasks, estimated ingredient demands, warnings, hard-constraint validation, and human-readable rationale.

Operations: generate proposal; validate; swap meal; change attendance/servings; confirm slot; calculate shopping diff; allocate/release prepared batch; record actual cooking.

Failure: return a structured infeasibility/conflict explanation, not an unsafe meal.

## Server/client boundary
Server: canonical state, permissions, optimization, durable task scheduling, recipe normalization, shopping provenance, sync and conflict policy.

Client: profile tour, fast edits, camera/share sheet, local cache and offline queue, recipe review, local timer display/notifications.

Keep equivalent domain/API semantics in standalone and Nextcloud modes. Verify the existing API and avoid assuming any endpoint names.

## Data quality and privacy
Mark provenance and confidence for extracted recipe/product/nutrition data. Avoid health inference from diagnoses. Health details are optional, encrypted/protected as appropriate, excluded from telemetry and external AI calls without authorization. Define retention and household role permissions before launch.
