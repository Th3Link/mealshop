# Speech and picture transcription through Nextcloud Task Processing

Status: Accepted
Date: 2026-11-28
Scope: F16/F39

## Context

The customer chose Nextcloud Task Processing for AI inputs (F39-AI, 2026-11-28), reported that the
instance offers text, speech-to-text, picture and handwriting tasks, and prefers small, low-cost
models with larger ones only where needed. Recipe capture already structures pasted or dictated
text with a deterministic reader. Browser dictation depends on the browser vendor and is absent
in some browsers; pictures had to be typed off. Task Processing takes audio and pictures as
Nextcloud file IDs, so the input must exist in the user's files.

## Alternatives

- Send text, audio and pictures to a language model that returns a structured recipe.
- Store a provider key in Mealshop and call a model API directly.
- Keep browser dictation and manual typing only.
- Let Task Processing turn audio or a picture into plain text and keep structuring deterministic.

## Decision

Mealshop asks Task Processing only for words: `core:audio2text` for a recording,
`core:image2text:ocr` for printed text and `core:analyze-images` with a fixed transcription
prompt for handwriting. The returned text goes into the capture text field, where the existing
deterministic reader structures it and the cook reviews it. No language model structures a recipe
in this slice, and a text model alone enables nothing.

The server acts as the signed-in user through `AppAPI` authentication. It offers only task types
that `taskprocessing/tasktypes` lists, uploads the input to `.mealshop-capture/` in the user's
files, schedules the task, and removes the file when the task succeeds, fails or is cancelled.
Pending tasks are held in memory (at most 50) and are visible only to the account that started
them. Inputs are limited to 25 MiB audio and 15 MiB pictures in listed media types. Nothing is
logged; Mealshop stores neither the recording nor the provider's answer.

## Consequences

The cheapest suitable task does the work: speech recognition and OCR are small models, the vision
model runs only when the cook asks for handwriting. Quality depends on the provider the
administrator chose; the UI says where the input goes. A restart while a task runs leaves its
temporary file in `.mealshop-capture/`; the user can delete it. Instances without a provider see
no new controls. Free-form requests and model-based structuring remain possible later through
`core:text2text` as an explicit fallback.
