# Decisions

All entries are proposals until the measurement in the plan is done; the accepted outcome is
recorded as a cross-cutting ADR in `docs/adr/` in the slice that adds the first model.

## D1 — Speech recognition candidates (proposed)

Requirement: offline, German and English, embeddable, licence compatible with AGPL-3.0-only.
Candidates to measure: whisper.cpp (MIT) with multilingual tiny/base models (MIT), and
sherpa-onnx (Apache-2.0) with a small multilingual or German/English model. The platform speech
recogniser is not a candidate: it is not part of the app and may send audio to its vendor
(same reasoning as the rejected ML Kit in ADR 0031). The keyboard's own voice typing remains
usable in every text field and is the user's own choice.

## D2 — Handwriting recognition candidates (proposed)

Tesseract stays for printed text and is known to be weak on handwriting. Candidates to measure:
`PaddleOCR` mobile recognition models (Apache-2.0) through an ONNX runtime, and a TrOCR-style
handwritten model (MIT) exported to ONNX. Open risks: German handwriting coverage, model size
(tens to hundreds of MB), memory on older phones, and the licence of each model's training data.
One on-device inference runtime should serve both speech and handwriting if possible.

## D3 — Model delivery (open, FL-MODELS)

Bundling keeps first use offline but grows every APK; fetching needs a pinned, checksummed
source, explicit consent, resumable download and a place to host or mirror the files.

## D4 — One recipe-text reader for two clients (proposed)

F39 implements deterministic reading in TypeScript. Proposal: move the rules into the shared Rust
core (already inside the app and the server) and call them from both clients, instead of a second
Dart implementation that drifts. Needs agreement with the F39 workstream before it starts.
