# Protected local household profiles

Enable the F09 slice by providing `MEALSHOP_HOUSEHOLD_TOKEN` (32–200 random ASCII letters,
digits, underscores or hyphens). Generate a strong operator secret with your normal secret
manager; do not commit it or put it in URLs. Without this variable household routes are absent.
The executable remains loopback-only. This is one household with one operator credential,
not a hosted account system. An optional distinct `MEALSHOP_HOUSEHOLD_READ_TOKEN` grants profile export and dietary evaluation only; writes/removals require the operator token. Anyone holding that credential can read/export, correct
and remove every profile in this household. Revocation/rotation means changing the configured
token and restarting; no token is written into profile files or browser storage.

`MEALSHOP_HOUSEHOLD_DIR` defaults to `var/household`. Use a dedicated local directory separate
from recipe storage. Newly created Unix directories/files use 0700/0600; existing directory
permissions stay intact. Profiles use a schema-1, revisioned `profiles.json` snapshot (at most
100 members); no profile revision history is retained. Editing/deletion replaces the snapshot.
This removes members from subsequent API reads; operator backups and previously exported
copies are outside server deletion. Backups contain sensitive profile data and require the
same access restrictions as the live store.

The demo asks for the operator token in a password field; it stays in page memory only.
Load profiles before editing. Select an existing member or New member; allergy/intolerance
notes are separate from hard ingredient exclusions and soft dislikes. Save or remove submits
the full household snapshot with its expected revision. A stale edit returns a conflict; copy
unsaved changes before reloading/reconciling. The review button evaluates current recipe
ingredient identities for every recorded member. No profile data or token is logged.

## API contracts
All endpoints require `Authorization: Bearer <operator token>` and return `Cache-Control:
no-store`. GET `/api/household/profiles` exports the complete schema/revision/member JSON.
PUT the same path with `{"expected_revision":0,"members":[...]}` creates/updates/removes
members by replacement. A member has `id` (lowercase identifier), `name`, `allergies` (text
notes), `allergen_ids` (structured hard groups), `intolerances` (separate review notes), `excluded_ingredients` and `disliked_ingredients` (canonical catalogue ID arrays).
Removing a member means omitting it from the replacement array; an empty array removes all.
No separate unauthenticated member lookup or profile route exists.

POST `/api/household/evaluate` accepts `member_ids` and `ingredient_ids`. Hard ingredient
exclusions are unioned across every selected member and return `blocked`. Known English/German
names/aliases are resolved conservatively to matching catalogue identities for exclusion
checks; ambiguous names never establish safety. Dislikes are returned separately and cannot
remove a hard exclusion. Unknown ingredients remain explicitly listed. Even with no exclusion,
status is `review_required`, never safe: the catalogue lacks complete allergen mappings and
cross-contamination data. Allergy/intolerance notes are retained for human review; this slice
does not infer an allergen ontology from unreviewed text. No learning mechanism can mutate
hard constraints. Group selection must reference existing household members.

## Persistence and recovery
Writes run on blocking workers, validate the whole replacement, sync a private `.pending`
file, rename it over the current snapshot, and sync the directory on Unix. Ambiguous I/O
failures disable further writes until operator recovery. Startup rejects malformed/unknown
schemas, invalid IDs, oversized data and an interrupted pending write. Exclusive `.household.lock`
prevents concurrent process writers. After a crash, stop all writers, back up the entire
protected directory and inspect the snapshot/pending file before removing a verified stale
lock. Preserve only a validated current snapshot in the recovery directory; never blindly
replace a good snapshot with an incomplete pending file. Restart and verify authorized export.
No automatic Git history, cloud sync or external writers are supported. Linux local filesystems
are the initial target; cross-platform crash durability has not been validated.

## Structured allergen constraints
GET `/api/household/allergens` returns the versioned reference registry to authorized readers.
The demo offers 14 English-base allergen groups with German labels. Selected `allergen_ids`
are hard constraints. 106 positive food-identity classifications retain exact source food IDs,
forms and an FSA source reference. These are reference identity classifications, not laboratory
measurements of retail products. Milk-group constraints also match explicitly named lactose-free
milk; absence of lactose does not remove the milk identity. See the source selection documentation
in data/README.md. Other allergy and intolerance notes remain review-required; exact legacy group
IDs in those notes are also treated conservatively as hard constraints. No natural-language
interpretation or personalised threshold is inferred.

Results include `allergen_matches` with member/ingredient/group/source, the classification version,
hard ingredient matches and separate dislikes. Every unblocked result requires review because
allergen coverage is positive-only and cross-contact is unmodelled. Hosted user management and
additional source coverage do not alter this rule. Read-only credentials cannot correct or
remove members; attempts return 403. Unknown credentials return 401. Rotate either configured
credential by restarting. Existing schema-1 profiles load with empty new fields.
