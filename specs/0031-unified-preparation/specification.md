# Unified meal planning, preparation and ingredient extensions

Status: In progress
Origin: Customer-approved visual concepts, 2026-11-10; implementation explicitly authorized.

## Acceptance contract
1. One native Nextcloud meal-plan calendar contains meals and their advance preparation. Normal cooking, mealprep and multi-day bread use the same workflow. No separate calendar or mandatory mealprep mode.
2. A preparation pins a recipe revision and production serving count. Its meal allocations may span days; allocated servings cannot exceed production. Shopping counts production once, including unallocated portions. Moving a meal must not duplicate production or its shopping contribution. Existing attendee exclusions, revision checks and purchase provenance remain enforced.
3. Recipe-owned parts own their ingredient lines and steps in the editor. Parts reference prepared parts, not duplicate their ingredient mass. Flat recipes remain simple and legacy records remain readable. Native controls, translations, draft protection and keyboard operation apply.
4. Recipe ingredient lines accept optional typed preparation extensions. Initial type: temperature, exact signed decimal Celsius with preserved original input and explicit provenance (recipe author declaration, not a device measurement). It does not alter ingredient identity, mass, nutrition, shopping or serving scaling. Malformed, duplicate and unsupported extensions are rejected. Saved revisions and subsequent edits preserve extensions.
5. Calendar preparation reminders and actual cooking completion are distinct. Completing mixing starts the preset waiting timer from the actual action, e.g. 07:08 + 45 min = 07:53. Parallel recipe parts must retain their relationship. No mandatory sourdough readiness assessment.
6. Approved concept screens are retained in this folder, labelled as design references, not implementation evidence. Validate real Nextcloud workflows, API persistence/negative cases, production-once shopping, client checks and repository gate before completion.

## Boundaries
No scale driver, device discovery, remote heating control or promised device protocol. Structured values are available in the existing versioned recipe JSON for a later separately specified integration. No invented cooking, cooling or storage durations; Celsius targets are instructions, not food-safety guarantees. Existing source quantities and unreviewed source content remain intact.

## Reference acceptance examples
Brotkruste: four prepared parts feed the main dough; water can retain 150 g plus a declared 60 °C target. Initial sourdough/brühstück/oat preparations start together; a calendar anchor schedules continuing with spelt at 07:00. Curry: four portions produced once Wednesday, two allocated Thursday and two Friday. Empty/unassigned production portions remain visible.
