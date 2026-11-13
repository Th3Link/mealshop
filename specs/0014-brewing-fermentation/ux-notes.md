# Native brewing workflow

Use the existing Nextcloud shell, components, theme tokens, MDI icons, responsive typography and keyed
EN/DE messages. UX-UI Pro Max informs hierarchy and recovery; Taste informs compact alignment and clear
actions, under the established native design hierarchy. No marketing layout, extra fonts or UI framework.

A dedicated Brewing navigation entry separates batches, brew recipes and equipment. The empty state
starts with a source-linked process template. Explain method versus equipment without making users
choose between overlapping labels such as "all-grain" and "BIAB". Templates name the source/model and
provide editable phase structure, with no invented universal rest temperatures or fermentation duration.

Recipe editing keeps batch volume, measured ingredients, method and equipment together. Phases stay
open and show the main timing/resource fields compactly; optional details use explicit plus actions.
Users can add/edit phases, their explicit dependencies, actual extract/hop additions and measurements without leaving context.
Save an immutable revision; changing a recipe must not rewrite a running batch's recipe snapshot.

Starting a batch chooses a saved revision, brew-day civil time, optional explicit yeast-preparation
reminder and concrete equipment. Review resolved dates/timezones, uncertainty, resource conflicts and
missing calibration/values before confirmation. Unknown is not zero and a brand is not a verified
capacity. Conflicts retain the draft and permit corrected dates/resources; no silent scheduling override.

Batch detail shows independent phase timers, equipment and original recipe, personal reminder delivery
state and exact readings. Elapsed time does not mark fermentation or packaging complete. Actual running/
paused phases retain equipment until explicitly completed or cancelled. Pausing one batch does not pause
another. Cancellation and corrections require explicit confirmation and preserve original observations.

Reading entry is compact: phase, raw value/unit, measurement time, manual or named instrument attribution
and known/unknown calibration. Recording a device transcription must not suggest a live connection.
Display original units and calibration; never silently convert SG/Plato/Brix, correct alcohol influence
or infer completion from readings. A reasoned correction preserves the earlier record.

Readers inspect without editing, operating timers or subscribing. Protect dirty drafts, retain rejected
saves, reject stale revisions and retry ambiguous confirmed requests without duplication. Ignore late
responses after navigation. Offline views remain visibly stale and read-only until reconnect.

Verify keyboard focus, labels/errors/status announcements, responsive overflow and axe WCAG 2.2 AA
in real Nextcloud owner/reader flows, EN/DE, light/dark and 1440/390/320 px. Use only ledger-owned
synthetic households and stop the sole writer before targeted cleanup; never inspect retained folders.
