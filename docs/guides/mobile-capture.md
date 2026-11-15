# Android capture and reviewed import

P5 extends the native app; it does not embed a web client. Android is the initial supported capture
platform. The manual recipe editor and HTTPS URL entry remain the fallback. The app has no external
OCR/AI provider, runtime model download, continuous camera feed or video downloader.

## Recipe from link or Android sharing

Open Recipes → Import recipe. Android's text/plain share destination also accepts a URL or text; an
incoming banner asks you to review the selected account/household. It never fetches a shared URL or
opens a route automatically. The protected inbox keeps one unreviewed item until dismissed and never
overwrites it with another share. Dismiss the old item before sharing another one. Sign-out clears it.

Enter the original HTTPS URL. Retrieval asks permission to contact the website and validated redirects
from the server IP, without Nextcloud credentials. F16's SSRF/robots/body/redirect/concurrency rules are
unchanged. Select a structured candidate, or enter privately without fetching when unsupported.
Restricted video/social-media access is not bypassed.

The always-open editor retains original fields, yield, times, ingredient lines and instructions separately.
Map every original ingredient to an explicit catalogue/household ID; inline creation remains available.
Only unambiguous mass values suggest exact amounts. Cups, fractions, ranges, volume conversions and
ambiguous locale separators stay unresolved. Supply missing servings/language/amounts, review all steps
and time estimates, then confirm. Every edit clears confirmation. Neither source nutrition nor OCR
allergy/diet text becomes a catalogue claim or a hard-rule exemption.

A protected household-bound draft is retained on this device. The first save durably freezes the reviewed
body and operation key before dispatch. An uncertain response disables editing; Retry sends the identical
body, including after process restart. Already committed receipts survive preview expiry/server restart.
An uncommitted preview expires after 30 minutes or server restart: inspect saved recipes, explicitly
discard the old device draft and review again. A definite validation rejection unlocks corrections.
No background import/rebase happens. Changed account/household or revoked writer rights blocks requests.
Drafts are bounded to 96 KiB within the existing total protected cache budget and retained until explicit
discard, membership removal or sign-out; uncertain requests are not silently expired by wall-clock edits.

## Photo, barcode and label

Camera or system photo selection is explicit and grants no broad storage permission. Cancelling returns
with surrounding fields intact. Camera errors leave manual entry available. A returned image is bounded
to 5 MiB, decoded only after dimensions are checked, downsampled to at most 2500 pixels per side, and
normalized for EXIF orientation. Manual rotation reruns recognition. OCR runs on one worker with a
20-second recognition cancellation bound; image decode/model setup are separately bounded by size.

Bundled fast English/German Tesseract models run offline. The raw text and engine score are candidates;
the score is not calibrated confidence or a guarantee. Retain the original, check it visually and classify
its title/ingredient lines/instructions. Empty recognition supports manual transcription. Only **Keep
original in household** uploads the photo/text to Nextcloud after disclosing household visibility. The
original image is immutable and separate from the editable recipe illustration. Uploaded assets follow
the existing image retention model, including an abandoned upload; discarding a device draft does not
remove server assets. Temporary picker files in the app's own cache are removed after processing; an
interrupted platform picker result is discarded with a notice rather than attached to another scope.

Scan product barcodes from a still photo (EAN-8, EAN-13, UPC-A, UPC-E expanded to UPC-A). Validate the GTIN
and choose it; no lookup runs automatically. Enter a GTIN manually if unsupported, unreadable or denied.
The existing Open Food Facts lookup keeps its explicit separate consent and source/licence metadata.

Products → Declared label values retains the source photo/OCR and **printed basis** (for example per
100 ml or per portion). Enter names and exact decimal values/units, optional printed definitions and
less-than/approximate qualifiers, then confirm. Absent definitions stay unknown. No implicit ml-to-g,
energy/salt conversion or nearby generic nutrition profile is used. These declarations remain attached
to the product; mapping a product to an ingredient does not replace either's nutrition data. Label text
is bounded to 4 KiB and 20 declarations so it fits the existing 8 KiB immutable shopping command limit.
Large declarations must be narrowed before submission; no original is silently truncated.

## Source and compatibility boundaries

URL imports share/export only their original source link. Photo imports have no fabricated link: cross-
household/account sharing and MCP return only `private_source`, without title, original image, OCR,
ingredients, steps or nutrition. Owning-household roles can read the original. Historical shares retain
the latest origin restriction. Vue schemas/readers in the same delivery recognize this redaction.

The Rust store gains optional photo evidence in the first import receipt, a `captured` origin and optional
product label declarations. Old records load unchanged; no rewrite/migration is performed. Retain whole
first recipe revisions and household images in backups. Do not downgrade new records into an older
server/client without a provenance-preserving migration. The phone vault stays schema 1 with an optional
`capture.v1` draft; old shopping/cooking records and their immutable outboxes are preserved.

## Install on a smartphone

Use the retained-key **arm64-v8a** APK for most current Android phones, or armeabi-v7a for supported 32-bit
phones. The emulator's `x86_64` APK is a separate architecture. Package `org.mealshop.pocket.dev`, version
`0.5.0+6`, uses the retained local development key; install updates over the existing app. Do not uninstall
or clear data to resolve a signing mismatch. Ordinary PR CI APKs have an ephemeral key and cannot replace
this retained-key install. Adjacent `.apk.build.json` and `.apk.sha256` identify exact source and bytes.

For the existing loopback-only test Nextcloud, connect the phone by USB, enable USB debugging and approve
this computer. Select its real serial explicitly (do not target the emulator):

```sh
var/mobile/android-sdk/platform-tools/adb -P 5038 devices
var/mobile/android-sdk/platform-tools/adb -P 5038 -s PHONE_SERIAL install -r clients/flutter/build/app/outputs/flutter-apk/app-arm64-v8a-debug.apk
var/mobile/android-sdk/platform-tools/adb -P 5038 -s PHONE_SERIAL reverse tcp:8080 tcp:8080
```

Open Mealshop and enter `http://127.0.0.1:8080`. The system browser/login polling and the returned canonical
Nextcloud origin must agree. The reverse tunnel is lost on reboot/disconnect; re-establish it. Existing
trusted domains already include the test setup. No LAN exposure, TLS bypass or trusted-domain expansion
is required. Wi-Fi needs a genuinely reachable HTTPS test origin configured separately; plain loopback
alone on an untethered phone is not the workstation. Release builds reject cleartext.

Physical camera quality, permission dialogs, large text, `TalkBack`, browser login, connectivity, update
installation and background behavior must be checked on the actual handset. Emulator/CI evidence is
recorded separately in F35 verification and does not complete these physical-device criteria.
