# Flutter guided start, people wizard and taste calibration

Status: In progress
Priority: Must
Milestone: M2
Origin: Customer request, 2026-11-28 (second workstream, Flutter app).

## Objective

A new on-device ("standalone") user opens the app and is led by a short assistant instead of
household administration: who are you, how do you eat, what must be avoided, what do you not like,
which dishes appeal to you. The answers become that person's rules and the starting values for
recipe suggestions. Further people can be added with as little or as much detail as wanted.

Customer wording (German, summarized): standalone has no households, only one internal default
whose name is never shown; a start wizard adds people, beginning with the person typing (the main
user) and asking for diet, possible diet goals (lose/gain weight, target weight, eat better, …),
intolerances and dislikes, by dictation or easy selection; some recipes are then proposed and
ranked, which seeds recipe suggestions and can be retrained later; more people can follow and
every step can be skipped; the wizard also asks for general consent to share recipes publicly
online, GDPR-conformant. The exchange protocol and its server are specified in a separate session.

## Acceptance

Slice 1 (this delivery) — one invisible household on the device:
1. Choosing on-device operation never asks for, creates by hand or displays a household. Exactly
   one internal household is created automatically when none exists and is selected.
2. In on-device mode no household name, household switcher, "create household" action or
   household wording appears on Home, More, exit or backup screens. Connected Nextcloud mode keeps
   its household selection unchanged.
3. An existing on-device dataset with several households (earlier builds, restored backups) loses
   nothing: the switcher stays available in exactly that case, and only there.

Slice 2 — start assistant and people:
4. After the first on-device start an assistant opens once: welcome → main user → optional further
   people → sharing consent → done. Every step and the whole assistant can be skipped; an
   interrupted assistant resumes; it can be reopened from More. Nothing is saved before the
   person's own summary is confirmed.
5. The main-user step asks one topic per screen with large selectable options first and free text
   second: name, diet (existing presets and exceptions), allergies and intolerances (hard),
   excluded and disliked ingredients (dislikes stay soft). It writes the existing profile model;
   allergies remain hard constraints that no later ranking can weaken.
6. The first person is marked as the main user of this device. Further people can be added with
   only a name. A person is not a login account.
7. Every free-text answer can be dictated (slice 4) or typed. Dictated text is shown and editable
   before it is used; nothing is interpreted silently into a hard rule without confirmation.

Slice 3 — goals and taste calibration (calibration waits for the recipe server API):
8. An optional goal step records a coarse intention (none, eat more balanced, lose, gain, keep)
   and, only when the person chooses to, current and target weight. No calorie target, diet plan
   or health claim is derived. Goal and weight data is stored in on-device and in connected
   Nextcloud mode (FL-GOALS C), only after explicit health-data consent, readable and editable
   through person-specific grants like activities (ADR 0028), never shared, exported with the
   person and erasable on its own.
9. A calibration step shows a small set of real starter recipes fetched from the online recipe
   server (FL-STARTER); none are bundled in the app. Only recipes that satisfy the person's hard
   rules are shown. Without a network or server the step explains that and is skipped.
   The person orders or rates them, or skips. The result is stored as explicit, inspectable
   starting preferences for that person, labelled as declared at setup, not as observed cooking.
10. Calibration can be repeated later for any person ("train again"); repeating replaces the
    declared starting preferences and never deletes cooking history.

Slice 4 — consent and privacy:
11. The assistant asks whether the user generally agrees to share their own recipes publicly.
    Default is no. Agreement is specific, informed, dated, versioned and revocable in More; it
    names what would be published. Until the exchange protocol exists nothing is transmitted and
    the UI says so. Imported and photographed recipes stay excluded (N06, ADR 0031).
12. A privacy summary states which data stays on the device, what a backup contains and how to
    export and erase everything. Health-related entries (allergies, intolerances, goals, weight)
    are collected only with an explicit, separately worded confirmation.

All slices: German/English keyed messages, 320 dp at 200 % text scale, screen-reader names, 48 dp
targets, loading/empty/error states; exact evidence in [plan.md](plan.md).

## Scope and limits

- Android, on-device mode first. The assistant's people steps reuse the existing person tour
  and therefore also work in connected mode; household handling there is unchanged.
- "Model training" here means small, explainable, on-device preference values built on F11
  (ADR 0021). No remote or cross-household training, no opaque model, no private data export.
- No diagnosis-to-diet engine, no medical advice, no invented nutrition or weight-loss claims.
- The public exchange protocol, its server, access control, identity and licence of shared
  recipes belong to F20 and the customer's parallel session. This feature only records consent.

## Customer decisions, 2026-11-28

- FL-STARTER: real starter recipes, served online by the customer's recipe server whose API is
  being built in a separate session; nothing is bundled into the app.
- FL-GOALS: C — coarse goal plus optional weight and target weight, also in connected mode.
- FL-MODELS (F42): B — models are fetched once after explicit consent from a pinned source.

## Links

[Plan](plan.md) · [UX notes](ux-notes.md) · [Decisions](adrs.md) ·
[F35](../0035-flutter-mobile-client/specification.md) ·
[F37](../0037-app-only-backups/specification.md) ·
[F11](../0011-recommendation-feedback/specification.md) ·
[F42](../0042-flutter-guided-capture/specification.md)
