# Implementation and verification plan

Status: Draft. Nothing here is implemented.

- [ ] Spike: build the synthetic test set and measure the D1/D2 candidates (size, memory, time,
      error rates) on the reference emulator; record results and licences; propose the ADR.
- [ ] Agree D4 with the F39 workstream; shared reader and test vectors.
- [ ] Slice A: Add-recipe assistant with paste, link, empty and the existing printed-text photo
      path, shared review screen.
- [ ] Slice B: on-device dictation (also used by F41 answers).
- [ ] Slice C: handwriting model, if the measurement supports it.
- [ ] Guide, licence notices, emulator and handset evidence.

## Open customer decisions

Frage 3 (FL-MODELS): how speech and handwriting models reach the phone.
A: bundled in the APK (fully offline from the first start, APK grows by roughly 100–300 MB
depending on the measured choice). B: fetched once after explicit consent from a pinned,
checksummed source (small APK, needs hosting or an upstream mirror). C: small speech model
bundled, handwriting model fetched.
Recommendation: C. Default if unanswered: B.
