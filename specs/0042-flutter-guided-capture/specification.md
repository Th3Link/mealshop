# Flutter guided recipe capture with on-device reading and dictation

Status: Draft
Priority: Must
Milestone: M2
Origin: Customer request, 2026-11-28 (second workstream, Flutter app).

## Objective

Creating a recipe on the phone is a short assistant like the Nextcloud **Add recipe** entry
(F39): take what is nearest — speak or paste, photograph a page or a handwritten card, hand over
a web link, or start empty — look over the result once, save. Reading and speech recognition run
on the device with embeddable open models; the app stays AGPL-3.0-only.

Customer wording (German, summarized): create recipe is the second big construction site; it
should read from the camera, so text recognition is needed, preferably also for handwriting;
dictating a recipe should work; overall similar to Nextcloud, again as a small assistant; with
embeddable models where possible.

## Acceptance

1. One **Add recipe** entry offers four ways: paste or dictate, photo or screenshot, web link,
   empty form. Existing link import, photo import and editor rules stay unchanged underneath.
2. Text from any way is read deterministically into title, servings, ingredient lines and steps
   with the same rules as F39 (exact decimals only, units recognised but never converted, original
   line kept visible, ingredient identified only by exactly one exact name or alias match). The
   reading rules have one shared definition and shared test vectors for Vue and Flutter.
3. A photo is read on the device. Printed text keeps working offline with the bundled engine.
   Handwriting uses a separately selected on-device model; its output is a candidate that is
   always reviewed next to the picture, with unreadable parts left empty rather than guessed.
4. Dictation transcribes on the device, in German and English, shows the text for correction
   before it is read into a recipe, and works without a network once its model is present.
5. Models are pinned by source, version, licence and SHA-256 like the existing OCR data. Whether
   a model is bundled or fetched after explicit consent follows FL-MODELS. No audio, image or
   text leaves the device for recognition. Licence notices ship in the app.
6. Measured evidence precedes any quality statement: a synthetic, versioned test set of printed
   and handwritten German/English recipe pages and spoken recipes, with character/word error
   rates, model size, memory and time on the reference emulator and on a physical handset.
   The UI makes no accuracy promise.
7. Camera and microphone permissions are requested at first use with a stated purpose; denial
   leaves typing and the gallery available. Originals stay private evidence (ADR 0031).
8. German/English, 320 dp at 200 % text scale, screen reader and offline states are verified.

## Scope and limits

Android first. No external OCR, speech or AI service. No video. A generative model that rewrites
or "improves" recipes is not part of this feature; structuring stays deterministic and reviewed.
Handwriting quality for German is uncertain until measured (AC6); if no embeddable model reaches
a useful level, the picture-beside-typing path remains and the limit is documented.

## Links

[Plan](plan.md) · [UX notes](ux-notes.md) · [Decisions](adrs.md) ·
[F35](../0035-flutter-mobile-client/specification.md) ·
[F41](../0041-flutter-guided-onboarding/specification.md) · ADR 0031
