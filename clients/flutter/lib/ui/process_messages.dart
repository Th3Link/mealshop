// Native process controls retain shared domain wording; device/recovery guidance is mobile-specific.
const processMessages = <String, (String, String)>{
  "history_review_retained": ("Review retained entry", "Eintrag erneut prüfen"),
  "history_review_retained_help": (
    "Keep your entries, load the current household, and confirm who ate before saving.",
    "Behalte deine Eingaben, lade den aktuellen Haushaltsstand und bestätige vor dem Speichern, wer mitgegessen hat.",
  ),

  "cook_all_ingredients": (
    "Recipe ingredients · pinned quantities",
    "Rezeptzutaten · fixierte Mengen",
  ),

  "brewTranscribed": (
    "Transcribed from instrument",
    "Vom Messgerät übertragen",
  ),
  "cookCancelStep": ("Cancel step", "Schritt abbrechen"),
  "cookNotify": (
    "Enable personal Nextcloud reminders",
    "Persönliche Nextcloud-Erinnerungen aktivieren",
  ),
  "cookStopNotify": (
    "Disable personal Nextcloud reminders",
    "Persönliche Nextcloud-Erinnerungen deaktivieren",
  ),
  "cookRetry": (
    "Retry reminder delivery",
    "Erinnerungszustellung erneut versuchen",
  ),

  "refresh": ("Refresh", "Aktualisieren"),
  "remove": ("Remove", "Entfernen"),
  "remove_ingredient": ("Remove ingredient", "Zutat entfernen"),
  "previous_page": ("Previous page", "Vorige Seite"),
  "next_page": ("Next page", "Nächste Seite"),
  "cooking_calendar_range_invalid": (
    "Choose a preparation window of up to seven days.",
    "Wähle ein Zubereitungsfenster von höchstens sieben Tagen.",
  ),

  "cook_agenda": ("Preparation agenda", "Anstehende Zubereitung"),
  "cook_agenda_zone": (
    "Each process uses its own time zone. Starts are planned; deadlines belong to actual running timers.",
    "Jeder Ablauf verwendet seine eigene Zeitzone. Starts sind geplant; Fristen gehören zu tatsächlich laufenden Timern.",
  ),
  "cook_agenda_empty": (
    "No preparation starts or timer deadlines on this day.",
    "An diesem Tag stehen keine Zubereitungsstarts oder Timerfristen an.",
  ),
  "history_response_unknown": (
    "The response was lost. Check saved cooking history before recording this cook again. This form will not resend automatically.",
    "Die Antwort fehlt. Prüfe das gespeicherte Kochprotokoll, bevor du die Mahlzeit erneut einträgst. Dieses Formular sendet nicht automatisch erneut.",
  ),
  "history_rebase": (
    "Reload saved record?",
    "Gespeicherten Eintrag neu laden?",
  ),
  "history_rebase_help": (
    "Replace your unsaved edits with the current saved record for a new correction.",
    "Ersetze deine ungespeicherten Änderungen durch den aktuellen Eintrag für eine neue Korrektur.",
  ),
  "history_epoch": ("Learning period", "Lernzeitraum"),
  "cooking_recovery_blocked": (
    "The protected cooking recovery data cannot be read. Actions and device reminders are paused. Check the server state before signing out to clear this device copy.",
    "Die geschützten Wiederherstellungsdaten sind nicht lesbar. Aktionen und Geräteerinnerungen sind angehalten. Prüfe den Serverstand, bevor du dich zum Löschen dieser Gerätekopie abmeldest.",
  ),

  "brew_show_archived": (
    "Show archived records",
    "Archivierte Einträge anzeigen",
  ),
  "brew_no_results": ("No matching records", "Keine passenden Einträge"),
  "cook_local_exact": (
    "Precise Android scheduling allowed · delivery unconfirmed",
    "Genaue Android-Zeitplanung erlaubt · Zustellung unbestätigt",
  ),
  "brew_before": ("Before this change", "Vor dieser Änderung"),
  "brew_after": ("After this change", "Nach dieser Änderung"),
  "brew_review_changed": (
    "Review concurrent changes",
    "Parallele Änderungen prüfen",
  ),
  "brew_review_changed_help": (
    "This recipe or equipment changed elsewhere. Keep your entries and explicitly review them against the latest saved version? The review shows both versions before saving.",
    "Dieses Rezept oder Gerät wurde anderswo geändert. Deine Eingaben behalten und ausdrücklich mit dem aktuellen Stand prüfen? Die Prüfung zeigt beide Versionen vor dem Speichern.",
  ),
  "brew_archive": ("Archive", "Archivieren"),
  "brew_restore": ("Restore from archive", "Aus Archiv wiederherstellen"),
  "brew_original_language": (
    "Original content language",
    "Sprache des Originalinhalts",
  ),
  "brew_unlink_ingredient": (
    "Remove catalogue link",
    "Zutatenverknüpfung entfernen",
  ),
  "brew_linked_ingredient": (
    "Catalogue ingredient linked",
    "Katalogzutat verknüpft",
  ),
  "brew_duration_error": (
    "Enter a whole duration from zero to 365 days.",
    "Gib eine ganze Dauer von null bis 365 Tagen ein.",
  ),
  "brew_local_timer_notice": (
    "The actual phase starts when you confirm. An estimate or elapsed time does not release equipment.",
    "Die tatsächliche Phase beginnt mit deiner Bestätigung. Eine Schätzung oder abgelaufene Zeit gibt kein Gerät frei.",
  ),
  "brew_open_pending": (
    "Open Brewing to review this retained change.",
    "Öffne Brauen, um diese gespeicherte Änderung zu prüfen.",
  ),
  "cook_calendar_anchor": ("Calendar reminder", "Kalendererinnerung"),
  "cooking_operation_capacity": (
    "The saved action history is full. Deleting sessions does not remove retry evidence. Ask the administrator to review a verified backup and retention policy.",
    "Der Aktionsspeicher ist voll. Das Löschen von Plänen entfernt keine Wiederholungsbelege. Bitte den Administrator um Prüfung einer verifizierten Sicherung und Aufbewahrungsregel.",
  ),
  "cook_offline_copies": ("Offline cooking copies", "Offline-Kochkopien"),
  "cook_allow_precise": (
    "Allow precise Android reminders",
    "Genauere Android-Erinnerungen erlauben",
  ),
  "cook_not_started": (
    "Planned · not started",
    "Geplant · noch nicht gestartet",
  ),
  "cookingSessions": ("Cooking & baking", "Kochen & Backen"),
  "cookNew": ("Start cooking or baking", "Kochen oder Backen planen"),
  "cookBack": ("All cooking sessions", "Alle Koch- und Backpläne"),
  "cookCreateHelp": (
    "Choose a recipe. Review its steps and timing before creating a separate, saved cooking session.",
    "Wähle ein Rezept. Prüfe Schritte und Zeiten, bevor du einen eigenen Koch- oder Backplan speicherst.",
  ),
  "cookOverviewHelp": (
    "Your saved sessions, running timers and next preparations.",
    "Deine gespeicherten Pläne, laufenden Timer und nächsten Vorbereitungen.",
  ),
  "cookNoRecipes": (
    "Add a recipe with preparation steps first.",
    "Lege zuerst ein Rezept mit Zubereitungsschritten an.",
  ),
  "cookLoadSteps": ("Continue with recipe steps", "Mit Rezeptschritten weiter"),
  "cookKind": ("Preparation", "Zubereitung"),
  "cookKindCooking": (
    "Cooking · start when ready",
    "Kochen · starten, wenn du bereit bist",
  ),
  "cookKindBread": ("Bread", "Brot"),
  "cookKindRolls": ("Rolls", "Brötchen"),
  "cookKindPizza": ("Pizza", "Pizza"),
  "cookKindBaguette": ("Baguette", "Baguette"),
  "cookTarget": ("Desired finish", "Gewünschter Fertigstellungstermin"),
  "cookStagesHelp": (
    "Durations come from your recipe. Adjust them, add dependencies and equipment as needed. Unknown conditions remain unknown; check the food yourself.",
    "Die Zeiten stammen aus deinem Rezept. Passe sie an und ergänze Abhängigkeiten und Ausstattung. Unbekannte Bedingungen bleiben unbekannt; prüfe das Ergebnis selbst.",
  ),
  "cookAddStage": ("Add preparation step", "Zubereitungsschritt hinzufügen"),
  "cookPreview": ("Review schedule", "Zeitplan prüfen"),
  "cookStageName": ("Step name", "Name des Schritts"),
  "cookSourceNote": (
    "Source or reason for this added step",
    "Quelle oder Grund für den ergänzten Schritt",
  ),
  "cookRecipeStep": ("Original recipe step", "Ursprünglicher Rezeptschritt"),
  "cookActiveDuration": ("Active work", "Arbeitszeit"),
  "cookPassiveDuration": ("Rest or waiting time", "Ruhe- oder Wartezeit"),
  "cookDependencies": (
    "Requires completed steps",
    "Benötigt abgeschlossene Schritte",
  ),
  "cookStageAdvanced": (
    "Equipment, repetitions and time windows",
    "Ausstattung, Wiederholungen und Zeitfenster",
  ),
  "cookEquipment": ("Equipment", "Ausstattung"),
  "cookEquipmentHelp": (
    "One item per line. Overlapping use is flagged in the review.",
    "Ein Eintrag pro Zeile. Zeitliche Überschneidungen werden in der Vorschau angezeigt.",
  ),
  "cookReminderLead": (
    "Preparation reminder before planned start",
    "Vorbereitungserinnerung vor geplantem Beginn",
  ),
  "cookRepeat": ("Repetitions", "Wiederholungen"),
  "cookRepeatWait": (
    "Wait between repetitions",
    "Wartezeit zwischen Wiederholungen",
  ),
  "cookPassiveMin": (
    "Shortest expected waiting time",
    "Kürzeste erwartete Wartezeit",
  ),
  "cookPassiveMax": (
    "Longest expected waiting time",
    "Längste erwartete Wartezeit",
  ),
  "cookUncertainty": (
    "Conditions and uncertainty",
    "Bedingungen und Unsicherheit",
  ),
  "cookUncertaintyHelp": (
    "Record the source of your estimate and relevant conditions. This plan cannot predict fermentation or guarantee a result.",
    "Halte die Quelle deiner Einschätzung und wichtige Bedingungen fest. Dieser Plan kann Gärung und Ergebnis nicht vorhersagen.",
  ),
  "cookRemoveStage": ("Remove step", "Schritt entfernen"),
  "cookDurationError": (
    "Enter whole units between zero and seven days.",
    "Gib ganze Einheiten zwischen null und sieben Tagen ein.",
  ),
  "cookSeconds": ("seconds", "Sekunden"),
  "cookMinutes": ("minutes", "Minuten"),
  "cookHours": ("hours", "Stunden"),
  "cookDays": ("days", "Tage"),
  "cookDaysShort": ("d", "T"),
  "cookLocalTime": ("Local date and time", "Lokales Datum und Uhrzeit"),
  "cookTimezone": ("IANA time zone", "IANA-Zeitzone"),
  "cookTimeZoneHelp": (
    "Times are resolved in this zone. Nonexistent daylight-saving times are rejected.",
    "Zeiten werden in dieser Zone aufgelöst. Nicht existierende Uhrzeiten bei der Zeitumstellung werden abgewiesen.",
  ),
  "cookFoldChoice": (
    "If the clock repeats this time",
    "Wenn diese Uhrzeit doppelt vorkommt",
  ),
  "cookFoldReject": ("Ask me to choose", "Auswahl anfordern"),
  "cookFoldEarlier": ("First occurrence", "Erstes Vorkommen"),
  "cookFoldLater": ("Second occurrence", "Zweites Vorkommen"),
  "cookReview": ("Review your schedule", "Deinen Zeitplan prüfen"),
  "cookPlannedStart": ("Planned start", "Geplanter Beginn"),
  "cookPlannedFinish": ("Planned finish", "Geplantes Ende"),
  "cookPreviousStart": ("Previous start", "Bisheriger Beginn"),
  "cookStartWindow": ("Estimated start window", "Geschätztes Startfenster"),
  "cookReviewExpires": ("Review valid until", "Vorschau gültig bis"),
  "cookReviewed": (
    "I reviewed the times, dependencies and uncertainties.",
    "Ich habe Zeiten, Abhängigkeiten und Unsicherheiten geprüft.",
  ),
  "cookApply": ("Save this schedule", "Diesen Zeitplan speichern"),
  "cookEditDraft": ("Back to editing", "Zurück zum Bearbeiten"),
  "cookSteps": ("Preparation steps", "Zubereitungsschritte"),
  "cookElapsed": ("Time elapsed", "Zeit abgelaufen"),
  "cookActivePhase": ("Active work", "Arbeitsphase"),
  "cookWaitingPhase": ("Waiting phase", "Wartephase"),
  "cookReady": ("Ready to start", "Bereit zum Starten"),
  "cookBlocked": ("Waiting for prerequisites", "Wartet auf Voraussetzungen"),
  "cookWaitUntil": ("Ready no earlier than", "Frühestens bereit ab"),
  "cookPaused": ("Paused", "Pausiert"),
  "cookCompleted": ("Completed", "Abgeschlossen"),
  "cookCancelled": ("Cancelled", "Abgebrochen"),
  "cookOverdueBy": ("Elapsed for", "Überfällig seit"),
  "cookRemaining": ("remaining", "verbleibend"),
  "cookPreset": ("Preset duration", "Voreingestellte Dauer"),
  "cookStartStep": ("Start step", "Schritt starten"),
  "cookCompleteStep": ("Mark step complete", "Schritt abschließen"),
  "cookPause": ("Pause", "Pausieren"),
  "cookResume": ("Resume", "Fortsetzen"),
  "cookDelayStep": ("Move planned start", "Geplanten Beginn verschieben"),
  "cookChangeTimer": ("Adjust timer", "Timer anpassen"),
  "cookTimingDetails": (
    "Timing and source details",
    "Zeit- und Quellenangaben",
  ),
  "cookChangeTarget": ("Change desired finish", "Fertigstellung verschieben"),
  "cookStartAfter": ("Start no earlier than", "Frühester Beginn"),
  "cookDeadline": ("Timer deadline", "Timer-Ende"),
  "cookNotifications": (
    "Reminders and delivery",
    "Erinnerungen und Zustellung",
  ),
  "cookNotificationHelp": (
    "Enable personal reminders through Nextcloud. Accepted means Nextcloud accepted the notification; delivery to a sleeping or offline device is not guaranteed. Your saved timers keep their deadlines.",
    "Aktiviere persönliche Erinnerungen über Nextcloud. Angenommen heißt, dass Nextcloud die Mitteilung angenommen hat; die Zustellung an schlafende oder nicht erreichbare Geräte ist nicht garantiert. Deine gespeicherten Timer behalten ihre Fristen.",
  ),
  "cookNoTransport": (
    "Nextcloud notifications are unavailable in this environment. Timers and upcoming steps remain visible here.",
    "Nextcloud-Mitteilungen sind in dieser Umgebung nicht verfügbar. Timer und anstehende Schritte bleiben hier sichtbar.",
  ),
  "cookPermissionDefault": ("Not requested", "Noch nicht angefragt"),
  "cookPermissionGranted": ("Allowed", "Erlaubt"),
  "cookPermissionDeniedShort": ("Denied", "Verweigert"),
  "cookPermissionUnsupportedShort": ("Unavailable", "Nicht verfügbar"),
  "cookSubscribe": (
    "Send Nextcloud reminders to me for this session",
    "Mir Nextcloud-Erinnerungen für diesen Plan senden",
  ),
  "cookSchedulerWaiting": (
    "No recent scheduler check. Keep this view open to see due actions and check the server if this persists.",
    "Noch keine aktuelle Scheduler-Prüfung. Fällige Schritte bleiben hier sichtbar. Prüfe den Server, falls dieser Zustand anhält.",
  ),
  "cookSchedulerChecked": (
    "Scheduler last checked",
    "Scheduler zuletzt geprüft",
  ),
  "cookDeliveryHistory": ("Delivery history", "Zustellverlauf"),
  "cookNoNotices": ("No reminders due yet.", "Noch keine Erinnerungen fällig."),
  "cookEventPrepare": ("Prepare", "Vorbereiten"),
  "cookEventStart": ("Planned start due", "Geplanter Beginn fällig"),
  "cookEventActive": ("Active phase ended", "Arbeitsphase beendet"),
  "cookEventEnd": ("Timer elapsed", "Timer abgelaufen"),
  "cookDeliveryQueued": ("Queued for delivery", "Zur Zustellung vorgemerkt"),
  "cookDeliverySending": ("Sending to Nextcloud", "Wird an Nextcloud gesendet"),
  "cookDeliveryAccepted": (
    "Accepted by Nextcloud · device delivery unconfirmed",
    "Von Nextcloud angenommen · Gerätezustellung unbestätigt",
  ),
  "cookDeliveryFailed": ("Delivery failed", "Zustellung fehlgeschlagen"),
  "cookDeliveryUnknown": (
    "Delivery uncertain · check Nextcloud",
    "Zustellung unklar · Nextcloud prüfen",
  ),
  "cookDeliveryMissed": (
    "Missed while unavailable · more than 24 hours old",
    "Während Nichterreichbarkeit verpasst · älter als 24 Stunden",
  ),
  "cookAcknowledged": ("Acknowledged", "Bestätigt"),
  "cookAcknowledge": ("Acknowledge", "Bestätigen"),
  "cookRetryRisk": (
    "Nextcloud may already have received this reminder. Send again and accept a possible duplicate?",
    "Nextcloud könnte diese Erinnerung bereits erhalten haben. Erneut senden und eine mögliche doppelte Mitteilung akzeptieren?",
  ),
  "cookOffline": (
    "Showing the retained cooking snapshot. Recorded actions stay pending until the server confirms them. Elapsed time alone never completes a step.",
    "Du siehst den gespeicherten Kochstand. Erfasste Aktionen bleiben offen, bis der Server sie bestätigt. Abgelaufene Zeit allein schließt keinen Schritt ab.",
  ),
  "cookRunActive": ("In preparation", "In Zubereitung"),
  "cookPinnedIngredients": (
    "Quantities refer to this saved recipe revision and serving count. Later recipe edits do not change this session.",
    "Die Mengen gehören zu dieser gespeicherten Rezeptversion und Portionszahl. Spätere Rezeptänderungen verändern diesen Plan nicht.",
  ),
  "cookCancelRun": ("Cancel session", "Plan abbrechen"),
  "cookCancelConfirm": (
    "Cancel this session and stop future timers and reminders?",
    "Diesen Plan abbrechen und zukünftige Timer und Erinnerungen stoppen?",
  ),
  "cookRemoveRun": ("Remove saved session", "Gespeicherten Plan entfernen"),
  "cookRemoveConfirm": (
    "Permanently remove this completed or cancelled session and its reminder history?",
    "Diesen abgeschlossenen oder abgebrochenen Plan samt Erinnerungsverlauf endgültig entfernen?",
  ),
  "cookEmpty": ("Ready when you are", "Bereit, wenn du es bist"),
  "cookEmptyHelp": (
    "Plan a recipe, review its steps and start the first timer.",
    "Plane ein Rezept, prüfe seine Schritte und starte den ersten Timer.",
  ),
  "cookRunningTimers": ("running timers", "laufende Timer"),
  "cooking_wait_not_elapsed": (
    "The explicit start delay or waiting period between repetitions has not elapsed.",
    "Der festgelegte früheste Beginn oder die Wartezeit zwischen Wiederholungen ist noch nicht erreicht.",
  ),
  "cooking_duration_estimate": (
    "Times are recipe estimates. Check readiness and record relevant conditions; fermentation and results are not guaranteed.",
    "Zeiten sind Rezept-Schätzungen. Prüfe das Ergebnis und halte Bedingungen fest; Gärverlauf und Ergebnis sind nicht garantiert.",
  ),
  "cooking_target_missed": (
    "The current constraints move the finish beyond your desired time.",
    "Die aktuellen Vorgaben verschieben das Ende über deinen Wunschtermin hinaus.",
  ),
  "cooking_parallel_work": (
    "Active work overlaps. Check that you can perform these actions together.",
    "Arbeitsphasen überschneiden sich. Prüfe, ob du diese Schritte gleichzeitig ausführen kannst.",
  ),
  "cooking_equipment_overlap": (
    "Equipment is reserved by overlapping steps. Review or change the schedule.",
    "Ausstattung wird in überlappenden Schritten benötigt. Prüfe oder ändere den Zeitplan.",
  ),
  "cooking_paused_projection": (
    "A paused timer has no fixed finish. This preview assumes it resumes now.",
    "Ein pausierter Timer hat kein festes Ende. Diese Vorschau nimmt an, dass er jetzt fortgesetzt wird.",
  ),
  "cooking_time_gap": (
    "This local time does not exist because the clock moves forward. Choose another time.",
    "Diese lokale Uhrzeit existiert wegen der Zeitumstellung nicht. Wähle eine andere Uhrzeit.",
  ),
  "cooking_time_fold": (
    "This local time occurs twice. Choose its first or second occurrence.",
    "Diese lokale Uhrzeit kommt doppelt vor. Wähle das erste oder zweite Vorkommen.",
  ),
  "cooking_timezone_invalid": (
    "Enter an IANA time zone such as Europe/Berlin.",
    "Gib eine IANA-Zeitzone wie Europe/Berlin ein.",
  ),
  "cooking_dependencies_invalid": (
    "Step dependencies must exist and cannot form a cycle.",
    "Schritt-Abhängigkeiten müssen vorhanden sein und dürfen keinen Kreis bilden.",
  ),
  "cooking_dependencies_incomplete": (
    "Complete the required earlier steps before starting this one.",
    "Schließe die benötigten vorherigen Schritte ab, bevor du diesen startest.",
  ),
  "cooking_stage_invalid": (
    "Check step names, sources, whole durations, repetitions and the shortest ≤ expected ≤ longest waiting times.",
    "Prüfe Schrittnamen, Quellen, ganze Zeitangaben, Wiederholungen und die Reihenfolge kürzeste ≤ erwartete ≤ längste Wartezeit.",
  ),
  "cooking_recipe_has_no_steps": (
    "Add preparation steps to the recipe before creating a session.",
    "Ergänze Zubereitungsschritte im Rezept, bevor du einen Plan anlegst.",
  ),
  "cooking_revision_conflict": (
    "Another change was saved. Reload the current state and review your changes again.",
    "Eine andere Änderung wurde gespeichert. Lade den aktuellen Stand und prüfe deine Änderungen erneut.",
  ),
  "cooking_preview_expired": (
    "This review expired or was replaced. Return to editing and create a new preview.",
    "Diese Vorschau ist abgelaufen oder wurde ersetzt. Gehe zur Bearbeitung zurück und erstelle eine neue Vorschau.",
  ),
  "cooking_review_required": (
    "Review the schedule before saving.",
    "Prüfe den Zeitplan vor dem Speichern.",
  ),
  "cooking_storage_failed": (
    "The cooking store is unavailable. Check server storage; existing deadlines have not been restarted.",
    "Der Planspeicher ist nicht verfügbar. Prüfe den Serverspeicher; bestehende Fristen wurden nicht neu gestartet.",
  ),
  "cooking_notifications_unavailable": (
    "Nextcloud notifications are unavailable. Check due actions here.",
    "Nextcloud-Mitteilungen sind nicht verfügbar. Prüfe fällige Schritte hier.",
  ),
  "cooking_delivery_connection_failed": (
    "Connection failed after bounded retries. Check Nextcloud before retrying.",
    "Die Verbindung ist nach begrenzten Versuchen fehlgeschlagen. Prüfe Nextcloud vor einem erneuten Versuch.",
  ),
  "cooking_delivery_rejected": (
    "Nextcloud rejected this notification. Check the AppAPI connection and permissions.",
    "Nextcloud hat diese Mitteilung abgewiesen. Prüfe AppAPI-Verbindung und Berechtigungen.",
  ),
  "cooking_duplicate_risk_required": (
    "A repeat may create a duplicate. Explicitly accept that risk to retry.",
    "Ein erneuter Versuch könnte eine doppelte Mitteilung erzeugen. Bestätige dieses Risiko ausdrücklich.",
  ),
  "cooking_notice_retry_invalid": (
    "This reminder can no longer be retried. Reload the current state.",
    "Diese Erinnerung kann nicht erneut gesendet werden. Lade den aktuellen Stand.",
  ),
  "cooking_timer_transition_invalid": (
    "This timer changed. Reload before choosing another action.",
    "Dieser Timer wurde verändert. Lade ihn vor der nächsten Aktion neu.",
  ),
  "cooking_run_inactive": (
    "This session is already completed or cancelled.",
    "Dieser Plan ist bereits abgeschlossen oder abgebrochen.",
  ),
  "cooking_time_invalid": (
    "Choose a valid date between 2020 and 2100. A timer deadline cannot be in the past.",
    "Wähle ein gültiges Datum zwischen 2020 und 2100. Ein Timer-Ende darf nicht in der Vergangenheit liegen.",
  ),
  "cooking_draft_invalid": (
    "Check the recipe revision, servings, steps and desired finish.",
    "Prüfe Rezeptversion, Portionen, Schritte und Fertigstellungstermin.",
  ),
  "cooking_capacity": (
    "The saved session or reminder limit was reached. Remove completed sessions you no longer need.",
    "Das Limit für gespeicherte Pläne oder Erinnerungen ist erreicht. Entferne nicht mehr benötigte abgeschlossene Pläne.",
  ),
  "cooking_run_not_found": (
    "This session is unavailable. Return to the session list.",
    "Dieser Plan ist nicht verfügbar. Gehe zur Planübersicht zurück.",
  ),
  "brewInstrumentReading": ("Instrument reading", "Gerätewert"),
  "brewPhaseSource": (
    "Household brewing recipe phase",
    "Phase im Braurezept des Haushalts",
  ),
  "brewInUse": (
    "In use until explicit completion",
    "Bis zum ausdrücklichen Abschluss belegt",
  ),
  "brewReservation": (
    "Existing planned reservation",
    "Bestehende geplante Reservierung",
  ),
  "brewKeepInputs": (
    "Load latest state; keep entries",
    "Aktuellen Stand laden; Eingaben behalten",
  ),
  "brewWorking": (
    "In progress · complete manually",
    "Läuft · manuell abschließen",
  ),
  "brewReload": ("Reload saved state", "Gespeicherten Stand neu laden"),
  "brewNone": ("No phase assigned", "Keine Phase zugeordnet"),
  "brewNotes": ("Notes", "Notizen"),
  "brewRemovePhaseRefs": (
    "Remove this phase and its ingredient/dependency references? Other phases remain.",
    "Diese Phase und ihre Zutaten-/Abhängigkeitsverweise entfernen? Andere Phasen bleiben erhalten.",
  ),
  "brewReported": (
    "Reported through the authenticated API",
    "Über die authentifizierte API gemeldet",
  ),
  "brewReviewed": (
    "I reviewed this change and its quantities, times and equipment.",
    "Ich habe die Änderung mit Mengen, Zeiten und Ausrüstung geprüft.",
  ),
  "brewBeer": ("Beer", "Bier"),
  "brewCider": ("Cider", "Cider"),
  "brewWine": ("Wine", "Wein"),
  "brewWhiteWine": ("White wine", "Weißwein"),
  "brewRedWine": ("Red wine", "Rotwein"),
  "brewMead": ("Mead", "Met"),
  "brewing": ("Brewing", "Brauen"),
  "brewBatches": ("Batches", "Chargen"),
  "brewRecipes": ("Brew recipes", "Braurezepte"),
  "brewEquipment": ("Equipment", "Ausrüstung"),
  "brewNewBatch": ("Plan a batch", "Charge planen"),
  "brewNewRecipe": ("Create brew recipe", "Braurezept erstellen"),
  "brewNewEquipment": ("Add equipment", "Ausrüstung hinzufügen"),
  "brewIntro": (
    "Plan beer, cider, wine and mead batches, keep phase timers and record your measurements.",
    "Plane Bier-, Cider-, Wein- und Metchargen, nutze Phasentimer und halte Messwerte fest.",
  ),
  "brewEmptyBatches": (
    "Your next brew starts here",
    "Hier beginnt dein nächster Sud",
  ),
  "brewEmptyRecipes": (
    "Start with a production method",
    "Starte mit einem Herstellungsverfahren",
  ),
  "brewEmptyEquipment": (
    "Add your actual vessels and tools",
    "Erfasse deine Gefäße und Geräte",
  ),
  "brewFirstRecipe": (
    "Create a brew recipe first, then plan its batch.",
    "Erstelle zuerst ein Braurezept und plane anschließend die Charge.",
  ),
  "brewMethod": ("Production method", "Herstellungsverfahren"),
  "brewTemplate": ("Starting template", "Startvorlage"),
  "brewTemplateHelp": (
    "Choose the method that matches your setup. Timings, temperatures and quantities come from your recipe.",
    "Wähle das Verfahren für deine Anlage. Zeiten, Temperaturen und Mengen stammen aus deinem Rezept.",
  ),
  "brewTemplateReplace": (
    "Replace the current draft with this template?",
    "Aktuellen Entwurf durch diese Vorlage ersetzen?",
  ),
  "brewVolume": ("Target batch volume", "Zielmenge der Charge"),
  "brewPhaseVolume": ("Volume in this phase", "Volumen in dieser Phase"),
  "brewCapacity": ("Declared usable capacity", "Angegebene nutzbare Kapazität"),
  "brewVolumeHelp": (
    "Capacity is checked against declared phase volume. Unknown volumes are not assumed to fit.",
    "Die Kapazität wird mit dem eingetragenen Phasenvolumen verglichen. Bei unbekannten Mengen ist die Eignung offen.",
  ),
  "brewUnknownTiming": (
    "Manual phase · duration unknown",
    "Manuelle Phase · Dauer unbekannt",
  ),
  "brewKnownTiming": ("Enter a duration", "Dauer angeben"),
  "brewManualHelp": (
    "Complete each phase yourself. Elapsed time does not confirm fermentation or release equipment.",
    "Schließe jede Phase selbst ab. Abgelaufene Zeit bestätigt keine Gärung und gibt keine Ausrüstung frei.",
  ),
  "brewPhase": ("Phase", "Phase"),
  "brewAddPhase": ("Add phase", "Phase hinzufügen"),
  "brewRemovePhase": ("Remove phase", "Phase entfernen"),
  "brewAddIngredient": ("Add brewing ingredient", "Brauzutat hinzufügen"),
  "brewIngredientHelp": (
    "Keep the original ingredient name and quantity. Missing quantities remain unknown.",
    "Bewahre den ursprünglichen Zutatennamen und die Menge. Fehlende Mengen bleiben unbekannt.",
  ),
  "brewAmount": ("Quantity", "Menge"),
  "brewAddQuantity": ("Add quantity", "Menge hinzufügen"),
  "brewAddTemperature": ("Add target temperature", "Zieltemperatur hinzufügen"),
  "brewTemperature": ("Target temperature", "Zieltemperatur"),
  "brewSource": ("Method evidence", "Quellen zum Verfahren"),
  "brewSourceHelp": (
    "Original method and model references. Your recipe can adapt these phases; no vendor connection or equipment control is configured.",
    "Originalquellen zu Verfahren und Modell. Passe die Phasen an dein Rezept an; eine Herstelleranbindung oder Gerätesteuerung ist nicht eingerichtet.",
  ),
  "brewDaysBefore": (
    "Calendar days before brew day",
    "Kalendertage vor dem Brautag",
  ),
  "brewYeast": (
    "Yeast preparation reminder",
    "Erinnerung zur Hefevorbereitung",
  ),
  "brewYeastHelp": (
    "Optional and product specific. Enter the instruction and intended local time; no universal yeast handling is assumed.",
    "Optional und produktspezifisch. Trage die Anweisung und Ortszeit ein; die Handhabung der Hefe wird nicht pauschal vorgegeben.",
  ),
  "brewAddYeast": ("Add yeast preparation", "Hefevorbereitung hinzufügen"),
  "brewLocalTime": ("Local time", "Ortszeit"),
  "brewDay": ("Brew day starts", "Beginn des Brautags"),
  "brewBatchName": ("Batch name", "Chargenname"),
  "brewRecipePreparation": (
    "Use recipe reminder",
    "Rezept-Erinnerung übernehmen",
  ),
  "brewSkipPreparation": (
    "No preparation reminder",
    "Keine Vorbereitungserinnerung",
  ),
  "brewCustomPreparation": (
    "Set a separate reminder",
    "Eigene Erinnerung festlegen",
  ),
  "brewNoEquipment": ("No equipment assigned", "Keine Ausrüstung zugeordnet"),
  "brewEquipmentHelp": (
    "Assign actual equipment to every phase that uses it, including cooling and fermentation. Only assigned equipment can be reserved.",
    "Ordne den Phasen die tatsächlich genutzte Ausrüstung zu, auch zum Kühlen und Gären. Nur zugeordnete Geräte werden reserviert.",
  ),
  "brewModel": ("Model or description", "Modell oder Beschreibung"),
  "brewEquipmentKind": ("Equipment type", "Art der Ausrüstung"),
  "brewMeasurements": ("Measurements", "Messwerte"),
  "brewAddReading": ("Record measurement", "Messwert erfassen"),
  "brewCorrectReading": ("Correct measurement", "Messwert korrigieren"),
  "brewReadingEmpty": (
    "No measurements recorded yet",
    "Noch keine Messwerte erfasst",
  ),
  "brewReadingValue": ("Original measured value", "Ursprünglicher Messwert"),
  "brewMeasuredAt": ("Measured at", "Gemessen am"),
  "brewEnteredAt": ("Recorded at", "Erfasst am"),
  "brewReadingSource": ("Measurement source", "Messwertquelle"),
  "brewManual": ("Manual reading", "Manuell abgelesen"),
  "brewInstrument": ("Instrument transcription", "Vom Messgerät übertragen"),
  "brewDeviceName": ("Instrument name", "Name des Messgeräts"),
  "brewDeviceId": ("Instrument identifier", "Kennung des Messgeräts"),
  "brewDeviceHelp": (
    "Enter the original instrument reading. This does not configure a live device connector.",
    "Trage den ursprünglichen Gerätewert ein. Dadurch wird keine Live-Anbindung eingerichtet.",
  ),
  "brewCalibration": ("Calibration", "Kalibrierung"),
  "brewCalibrationUnknown": ("Calibration unknown", "Kalibrierung unbekannt"),
  "brewCalibrationDeclared": (
    "Record calibration details",
    "Kalibrierung dokumentieren",
  ),
  "brewCalibrationDetails": (
    "Calibration details and reference",
    "Angaben und Referenz zur Kalibrierung",
  ),
  "brewCalibrationAt": ("Calibrated at", "Kalibriert am"),
  "brewReferenceTemperature": (
    "Calibration reference temperature",
    "Referenztemperatur der Kalibrierung",
  ),
  "brewOriginalUnits": (
    "Original units are retained; SG, °Plato and °Brix are not automatically converted or temperature corrected.",
    "Originaleinheiten bleiben erhalten; SG, °Plato und °Brix werden nicht automatisch umgerechnet oder temperaturkorrigiert.",
  ),
  "brewCorrectionReason": ("Reason for correction", "Grund der Korrektur"),
  "brewCorrected": ("Superseded by a correction", "Durch Korrektur ersetzt"),
  "brewCorrection": ("Correction of", "Korrektur von"),
  "brewEarlier": ("Load earlier measurements", "Ältere Messwerte laden"),
  "brewReview": ("Review brewing change", "Änderung prüfen"),
  "brewReviewHelp": (
    "Check the recipe, quantities, schedule and assigned equipment before saving.",
    "Prüfe Rezept, Mengen, Zeitplan und zugeordnete Ausrüstung vor dem Speichern.",
  ),
  "brewRetryHelp": (
    "The save result is uncertain. Retry this exact request before editing or making another change.",
    "Das Speicherergebnis ist unklar. Wiederhole diese Anfrage unverändert, bevor du weiterbearbeitest oder andere Änderungen vornimmst.",
  ),
  "brewRetry": ("Retry the same save", "Speichern unverändert wiederholen"),
  "brewSaved": ("Brewing change saved", "Änderung gespeichert"),
  "brewArchived": ("Archived", "Archiviert"),
  "brewShowArchived": (
    "Show archived entries",
    "Archivierte Einträge anzeigen",
  ),
  "brewArchive": ("Archive", "Archivieren"),
  "brewRestore": ("Restore", "Wiederherstellen"),
  "brewPinned": ("Pinned recipe revision", "Fixierte Rezeptversion"),
  "brewRecipeLanguage": ("Recipe language", "Rezeptsprache"),
  "brewEquipmentBusy": (
    "This equipment is in use. Complete or cancel its current phase first.",
    "Diese Ausrüstung ist belegt. Schließe ihre aktuelle Phase zuerst ab oder brich sie ab.",
  ),
  "brewEdit": ("Edit brew recipe", "Braurezept bearbeiten"),
  "brewEditEquipment": ("Edit equipment", "Ausrüstung bearbeiten"),
  "brewExtract": ("Malt extract", "Malzextrakt"),
  "brewExtractSteep": (
    "Extract with specialty steep",
    "Extrakt mit Spezialmalz-Auszug",
  ),
  "brewPartialMash": ("Partial mash", "Teilmaische"),
  "brewAllGrain": ("All-grain", "All-Grain"),
  "brewKettle": ("Kettle", "Kessel"),
  "brewBag": ("Brew-in-a-bag", "Maischesack"),
  "brewTopReturn": (
    "Basket with top recirculation",
    "Malzkorb mit oberem Rücklauf",
  ),
  "brewMaltPipe": ("Upward-flow malt pipe", "Aufwärts durchströmtes Malzrohr"),
  "brewSeparateTun": ("Separate mash tun", "Separater Maischebottich"),
  "brewTwoVessel": ("Two-vessel system", "Zwei-Gefäß-System"),
  "brewThreeVessel": ("Three-vessel system", "Drei-Gefäß-System"),
  "brewManualHeat": (
    "Manual / unspecified heat",
    "Manuelle / nicht festgelegte Heizung",
  ),
  "brewDirect": ("Direct heat", "Direkte Heizung"),
  "brewHerms": ("HERMS", "HERMS"),
  "brewRims": ("RIMS", "RIMS"),
  "brewNoneSparge": ("No sparge", "Ohne Nachguss"),
  "brewBasketRinse": ("Basket rinse", "Malzkorb-Nachguss"),
  "brewBatch": ("Batch sparging", "Nachguss in Chargen"),
  "brewFly": ("Continuous sparging", "Kontinuierlicher Nachguss"),
  "brewNoneMash": ("No mash", "Keine Maische"),
  "brewInfusion": ("Single infusion", "Einfache Infusion"),
  "brewStep": ("Step mash", "Stufenmaische"),
  "brewDecoction": ("Decoction", "Dekoktion"),
  "brewStrategy": ("Fermentable strategy", "Vergärbare Grundlage"),
  "brewLayout": ("Vessel layout", "Gefäßaufbau"),
  "brewHeat": ("Heating", "Heizung"),
  "brewSparge": ("Sparging", "Nachguss"),
  "brewMash": ("Mashing", "Maischen"),
  "brewMashTun": ("Mash tun", "Maischebottich"),
  "brewHotLiquorTank": ("Hot liquor tank", "Heißwassertank"),
  "brewPump": ("Pump", "Pumpe"),
  "brewChiller": ("Chiller", "Kühler"),
  "brewFermenter": ("Fermenter", "Gärbehälter"),
  "brewOther": ("Other", "Sonstige"),
  "brewCelsius": ("°C", "°C"),
  "brewFahrenheit": ("°F", "°F"),
  "brewSg": ("Specific gravity (SG)", "Relative Dichte (SG)"),
  "brewPlato": ("°Plato", "°Plato"),
  "brewBrix": ("°Brix", "°Brix"),
  "brewMl": ("ml", "ml"),
  "brewL": ("l", "l"),
  "brewing_equipment_conflict": (
    "This equipment was edited elsewhere. Keep your draft, then reload the saved equipment before reviewing another change.",
    "Diese Ausrüstung wurde an anderer Stelle bearbeitet. Behalte deinen Entwurf und lade die gespeicherte Ausrüstung vor einer erneuten Änderung neu.",
  ),
  "brewing_equipment_busy": (
    "Equipment is still in use. Complete or cancel its current phase first.",
    "Die Ausrüstung ist noch belegt. Schließe ihre aktuelle Phase ab oder brich sie ab.",
  ),
  "brewing_equipment_overlap": (
    "The assigned equipment has overlapping reservations. Use different equipment or adjust the phases and dates.",
    "Für die zugeordnete Ausrüstung überschneiden sich Reservierungen. Wähle andere Geräte oder passe Phasen und Zeiten an.",
  ),
  "brewing_equipment_capacity": (
    "The declared phase volume exceeds the assigned equipment capacity.",
    "Das eingetragene Phasenvolumen überschreitet die Kapazität der zugeordneten Ausrüstung.",
  ),
  "brewing_preparation_invalid": (
    "Check the preparation instruction, timezone and date. A requested reminder must be before brew day and still in the future. Skip or reschedule an earlier reminder.",
    "Prüfe Anweisung, Zeitzone und Datum der Vorbereitung. Die Erinnerung muss vor dem Brautag und noch in der Zukunft liegen. Überspringe eine frühere Erinnerung oder plane sie neu.",
  ),
  "brewing_recipe_conflict": (
    "This recipe has changed. Keep your draft and reload its latest revision before reviewing again.",
    "Dieses Rezept wurde geändert. Behalte deinen Entwurf und lade vor der erneuten Prüfung die aktuelle Version.",
  ),
  "brewing_measurement_future": (
    "A recorded measurement cannot be in the future. Check its timezone and date.",
    "Ein erfasster Messwert darf nicht in der Zukunft liegen. Prüfe Zeitzone und Datum.",
  ),
  "brewing_reading_correction_invalid": (
    "Correct the latest measurement version and enter a reason. The original remains saved.",
    "Korrigiere die neueste Messwertversion und gib einen Grund an. Das Original bleibt gespeichert.",
  ),
  "brewing_measurement_invalid": (
    "Enter the original value with up to six decimal places and a matching unit.",
    "Trage den Originalwert mit bis zu sechs Nachkommastellen und passender Einheit ein.",
  ),
  "brewing_recipe_archived": (
    "Restore this archived recipe before planning another batch.",
    "Stelle dieses archivierte Rezept wieder her, bevor du eine weitere Charge planst.",
  ),
  "brewing_capacity": (
    "The household brewing storage limit was reached. Archiving retains history and does not free space. Ask the server administrator to review a verified backup and storage policy.",
    "Das Speicherlimit für die Braudaten ist erreicht. Archivieren erhält den Verlauf und gibt keinen Speicher frei. Bitte die Serveradministration um Prüfung einer verifizierten Sicherung und des Speicherlimits.",
  ),
  "historyChooseRecipe": (
    "Choose the saved recipe you cooked.",
    "Wähle das gespeicherte Rezept, das du gekocht hast.",
  ),
  "historyChooseAttendees": (
    "Choose at least one attendee.",
    "Wähle mindestens eine Person aus.",
  ),
  "suggestDislikes": (
    "Declared ingredient dislikes affect this position.",
    "Angegebene Abneigungen gegen Zutaten werden berücksichtigt.",
  ),
  "suggestRules": (
    "Rule-based suggestions; learning is off.",
    "Vorschläge nach festen Regeln; das Lernen ist ausgeschaltet.",
  ),
  "suggestLearned": (
    "Ordered using this household’s recorded experience.",
    "Geordnet nach den erfassten Erfahrungen dieses Haushalts.",
  ),
  "suggestSafety": (
    "Exclusions for selected attendees are checked first. Missing dietary information or unassigned attendees still need review; history support is not a safety guarantee or prediction probability.",
    "Ausschlüsse ausgewählter Teilnehmender werden zuerst geprüft. Fehlende Ernährungsangaben oder Teilnehmende bleiben prüfungspflichtig; die Erfahrungsbasis ist keine Sicherheitsgarantie oder Trefferwahrscheinlichkeit.",
  ),
  "suggestDiscovery": ("Try a recipe", "Ein Rezept ausprobieren"),
  "suggestNoHistory": (
    "No current learning history",
    "Noch ohne aktuelle Lernhistorie",
  ),
  "suggestLimited": ("Limited history", "Wenig Kochhistorie"),
  "suggestRepeated": ("Repeated cooking history", "Wiederholt gekocht"),
  "suggestCookOne": ("completed cook", "Kochvorgang"),
  "suggestCookMany": ("completed cooks", "Kochvorgänge"),
  "suggestHouseholdSamples": (
    "Household rating count",
    "Anzahl Haushaltsbewertungen",
  ),
  "suggestPersonalSamples": (
    "Attendee rating count",
    "Anzahl Einzelbewertungen",
  ),
  "suggestRecent": (
    "Cooks in the 7 days up to this meal",
    "Kochvorgänge in den 7 Tagen bis zu dieser Mahlzeit",
  ),
  "suggestEarlier": (
    "Cooks from earlier recipe revisions",
    "Kochvorgänge aus früheren Rezeptständen",
  ),
  "history": ("Cooking history", "Kochhistorie"),
  "historyIntro": (
    "What you actually cooked, who ate and how it went.",
    "Was du wirklich gekocht hast, wer mitgegessen hat und wie es war.",
  ),
  "historyNew": ("Record a completed cook", "Gekochtes Gericht festhalten"),
  "historyEdit": ("Edit cooking record", "Kocheintrag bearbeiten"),
  "historyEmpty": (
    "No completed cooks recorded",
    "Noch keine gekochten Gerichte erfasst",
  ),
  "historyEmptyHelp": (
    "Record a cook here or from a completed cooking session. Planned meals are not counted.",
    "Erfasse einen Kochvorgang hier oder aus einer abgeschlossenen Kochsitzung. Geplante Mahlzeiten zählen nicht mit.",
  ),
  "historyRecipe": ("Cooked recipe", "Gekochtes Rezept"),
  "historyDate": ("Date cooked", "Gekocht am"),
  "historyZone": ("Time zone", "Zeitzone"),
  "historyPeople": ("Who ate?", "Wer hat mitgegessen?"),
  "historyEffort": ("Actual active effort", "Tatsächliche aktive Arbeit"),
  "historyEffortHelp": (
    "Optional. Report your active work; timer estimates are not measurements.",
    "Optional. Gib deine aktive Arbeit an; Timer-Schätzungen sind keine Messwerte.",
  ),
  "historyEffortUnknown": ("Effort not recorded", "Aufwand nicht erfasst"),
  "historyNoRating": ("Not rated", "Nicht bewertet"),
  "historyHouseholdRating": ("Household rating", "Bewertung des Haushalts"),
  "historyPersonalRating": ("Rating for", "Bewertung für"),
  "historyConfirm": (
    "This was actually cooked.",
    "Das wurde tatsächlich gekocht.",
  ),
  "historyVisibility": (
    "Records and ratings are visible to everyone in this household. Individual ratings are reported for dietary profiles.",
    "Einträge und Bewertungen sind für alle Mitglieder dieses Haushalts sichtbar. Einzelbewertungen werden für Ernährungsprofile erfasst.",
  ),
  "historySession": (
    "From a completed cooking session",
    "Aus einer abgeschlossenen Kochsitzung",
  ),
  "historyManual": ("Manually confirmed", "Manuell bestätigt"),
  "historyPinned": (
    "Recipe and servings are pinned to the completed session.",
    "Rezept und Portionen bleiben an die abgeschlossene Kochsitzung gebunden.",
  ),
  "historySaved": ("Cooking record saved", "Kocheintrag gespeichert"),
  "historyDelete": ("Delete cooking record", "Kocheintrag löschen"),
  "historyDeleteConfirm": (
    "Delete this completed cook and its contribution to learning?",
    "Diesen Kochvorgang und seinen Beitrag zum Lernen löschen?",
  ),
  "historySearch": ("Search cooking history", "Kochhistorie durchsuchen"),
  "historyRecordedBy": ("Recorded by", "Erfasst von"),
  "historyUpdatedBy": ("Last corrected by", "Zuletzt korrigiert von"),
  "historyPeopleRequired": (
    "Add a dietary profile before recording who ate.",
    "Lege ein Ernährungsprofil an, bevor du die Teilnehmenden erfasst.",
  ),
  "historyRemovedPerson": ("Removed profile", "Entferntes Profil"),
  "historyRemovedHelp": (
    "An attendee profile no longer exists. Remove it and select current attendees before saving a correction.",
    "Ein früheres Profil ist nicht mehr vorhanden. Entferne es und wähle aktuelle Teilnehmende, bevor du eine Korrektur speicherst.",
  ),
  "historyLearning": ("Preference learning", "Vorlieben lernen"),
  "historyLearningOff": ("Learning is off", "Lernen ist ausgeschaltet"),
  "historyLearningOn": ("Learning is on", "Lernen ist eingeschaltet"),
  "historyLearningHelp": (
    "Only this household’s completed cooks and ratings are used. Nothing is sent to a training service.",
    "Verwendet werden nur gekochte Gerichte und Bewertungen dieses Haushalts. Es wird nichts an einen Trainingsdienst gesendet.",
  ),
  "historyLearningStart": (
    "Use completed cooks for suggestions",
    "Gekochte Gerichte für Vorschläge verwenden",
  ),
  "historyLearningScope": (
    "Also use ratings for these profiles",
    "Zusätzlich Bewertungen dieser Profile verwenden",
  ),
  "historyLearningNew": (
    "Enabling or changing scope starts fresh. Only cooks saved or explicitly corrected afterward contribute; older retained ratings are not silently replayed.",
    "Einschalten oder Ändern beginnt einen neuen Lernzeitraum. Erst danach gespeicherte oder ausdrücklich korrigierte Kochvorgänge zählen; ältere Bewertungen werden nicht still erneut ausgewertet.",
  ),
  "historyLearningSave": ("Save learning scope", "Lernumfang speichern"),
  "historyLearningReset": (
    "Reset and turn off learning",
    "Lernen zurücksetzen und ausschalten",
  ),
  "historyLearningResetConfirm": (
    "Erase learned preference aggregates and turn learning off? Your cooking records remain saved.",
    "Gelernte Vorlieben löschen und das Lernen ausschalten? Deine Kocheinträge bleiben gespeichert.",
  ),
  "historyLearningChanged": (
    "Learning scope saved; a fresh learning period has started.",
    "Lernumfang gespeichert; ein neuer Lernzeitraum hat begonnen.",
  ),
  "historyLearningResetDone": (
    "Learning reset and turned off. Cooking records remain saved.",
    "Lernen zurückgesetzt und ausgeschaltet. Kocheinträge bleiben gespeichert.",
  ),
  "historyRecordedCooks": (
    "Saved cooking records",
    "Gespeicherte Kocheinträge",
  ),
  "historyLearnedCooks": (
    "Contributing completed cooks",
    "Berücksichtigte Kochvorgänge",
  ),
  "historyLearnedRecipes": ("Recipes with history", "Rezepte mit Kochhistorie"),
  "historyOwnerOnly": (
    "The household owner manages learning and deletion of all history.",
    "Der Haushaltsinhaber verwaltet das Lernen und das Löschen der gesamten Historie.",
  ),
  "historyClear": (
    "Delete all cooking history",
    "Gesamte Kochhistorie löschen",
  ),
  "historyClearConfirm": (
    "Delete every cooking record and learned contribution in this household? This cannot be undone.",
    "Alle Kocheinträge und Lernbeiträge dieses Haushalts löschen? Das lässt sich nicht rückgängig machen.",
  ),
  "historyCleared": ("Cooking history deleted", "Kochhistorie gelöscht"),
  "historyRatingHelp": (
    "1 = disliked · 5 = enjoyed. Leave unknown ratings unassigned.",
    "1 = nicht gemocht · 5 = gern gegessen. Lass unbekannte Bewertungen offen.",
  ),
  "historySourceRecipe": ("Recipe revision", "Rezeptstand"),
  "historyAttendeeRating": ("Individual ratings", "Einzelbewertungen"),
  "historyReload": ("Reload saved history", "Gespeicherte Historie neu laden"),
  "history_not_found": (
    "This cooking record is unavailable or has been deleted.",
    "Dieser Kocheintrag ist nicht verfügbar oder wurde gelöscht.",
  ),
  "history_input_invalid": (
    "Check the date, attendees, optional effort and ratings from 1 to 5.",
    "Prüfe Datum, Teilnehmende, optionalen Aufwand und Bewertungen von 1 bis 5.",
  ),
  "history_profiles_changed": (
    "The dietary profiles changed. Review current attendees before saving.",
    "Die Ernährungsprofile haben sich geändert. Prüfe die aktuellen Teilnehmenden vor dem Speichern.",
  ),
  "history_revision_conflict": (
    "Cooking history changed. Your draft is retained; reload explicitly to review the current state.",
    "Die Kochhistorie hat sich geändert. Dein Entwurf bleibt erhalten; lade ausdrücklich den aktuellen Stand neu.",
  ),
  "history_completion_required": (
    "Confirm that this was actually cooked.",
    "Bestätige, dass dies tatsächlich gekocht wurde.",
  ),
  "history_future_date": (
    "A completed cook cannot have a future date in its selected time zone.",
    "Ein abgeschlossener Kochvorgang darf in seiner gewählten Zeitzone nicht in der Zukunft liegen.",
  ),
  "history_session_incomplete": (
    "Complete the cooking session before recording it in history.",
    "Schließe die Kochsitzung ab, bevor du sie in der Historie erfasst.",
  ),
  "history_duplicate_session": (
    "This cooking session is already recorded. Open its existing history entry.",
    "Diese Kochsitzung ist bereits erfasst. Öffne ihren vorhandenen Kocheintrag.",
  ),
  "history_source_changed": (
    "Keep the original completed-session recipe and servings.",
    "Behalte Rezept und Portionen der ursprünglichen abgeschlossenen Kochsitzung.",
  ),
  "history_recipe_invalid": (
    "Choose an existing saved recipe revision.",
    "Wähle einen vorhandenen gespeicherten Rezeptstand.",
  ),
  "history_capacity": (
    "Cooking history reached its record or storage limit. Remove entries you no longer need.",
    "Die Kochhistorie hat ihr Eintrags- oder Speicherlimit erreicht. Entferne nicht mehr benötigte Einträge.",
  ),
  "cook_local_reminders": (
    "Remind me on this device",
    "Auf diesem Gerät erinnern",
  ),
  "cook_local_help": (
    "Local reminders are separate from Nextcloud delivery. Android may delay them, especially in battery saving mode. No guaranteed alarm; reopen the app to check due work.",
    "Lokale Erinnerungen sind von Nextcloud getrennt. Android kann sie verzögern, besonders im Energiesparmodus. Kein garantierter Alarm; prüfe fällige Schritte beim Öffnen der App.",
  ),
  "cook_local_horizon": (
    "Reminders cover the next seven days after authorization; return online to refresh longer preparations.",
    "Erinnerungen gelten bis sieben Tage nach der letzten Berechtigungsprüfung; aktualisiere längere Vorbereitungen online.",
  ),
  "cook_local_limited": (
    "Only the next 200 reminders fit. Refresh before later actions are due.",
    "Nur die nächsten 200 Erinnerungen passen. Aktualisiere vor späteren fälligen Schritten.",
  ),
  "cook_local_inexact": (
    "Inexact Android reminder",
    "Android-Erinnerung kann verzögert sein",
  ),
  "cook_pending": (
    "Recorded actions awaiting confirmation",
    "Erfasste Aktionen warten auf Bestätigung",
  ),
  "cook_pending_help": (
    "Your original action time and request are saved. Refresh to reconcile; an uncertain result retries the identical request.",
    "Dein ursprünglicher Aktionszeitpunkt und Request sind gespeichert. Aktualisiere zum Abgleichen; bei ungewissem Ergebnis wird derselbe Request wiederholt.",
  ),
  "cook_pending_review": (
    "Review recorded action against current state",
    "Erfasste Aktion mit aktuellem Stand prüfen",
  ),
  "cook_discard_actions": (
    "Discard unconfirmed actions",
    "Unbestätigte Aktionen verwerfen",
  ),
  "cook_discard_help": (
    "Discard these unconfirmed intents only? This does not undo work already confirmed by the server.",
    "Nur diese unbestätigten Absichten verwerfen? Bereits vom Server bestätigte Arbeit wird dadurch nicht rückgängig gemacht.",
  ),
  "cook_action_time": ("Reported action time", "Erfasster Aktionszeitpunkt"),
  "cook_action_review": (
    "Review retained action",
    "Gespeicherte Aktion prüfen",
  ),
  "cook_action_confirm": (
    "Apply this reviewed action",
    "Geprüfte Aktion anwenden",
  ),
  "cook_step_ingredients": (
    "Ingredients for this step",
    "Zutaten für diesen Schritt",
  ),
  "cook_recipe_reference": (
    "Recipe reference · no step assignment",
    "Rezeptübersicht · keine Schrittzuordnung",
  ),
  "cook_work_finished": (
    "Work finished · start waiting timer",
    "Arbeit fertig · Wartezeit starten",
  ),
  "cook_manual_complete": ("Step completed", "Schritt erledigt"),
  "cook_complete_confirm": (
    "Confirm this step is actually complete? An elapsed timer alone does not establish readiness.",
    "Ist dieser Schritt tatsächlich abgeschlossen? Ein abgelaufener Timer bestätigt keine Fertigstellung.",
  ),
  "cook_timer_pending": (
    "Recorded locally · not server-confirmed",
    "Lokal erfasst · noch nicht vom Server bestätigt",
  ),
  "cook_unknown": (
    "Outcome uncertain · retry unchanged",
    "Ergebnis ungewiss · unverändert erneut abgleichen",
  ),
  "cook_rejected": (
    "Not applied · review required",
    "Nicht angewendet · neue Prüfung erforderlich",
  ),
  "cook_forget_cache": (
    "Remove offline cooking copy",
    "Offline-Kochkopie entfernen",
  ),
  "cook_cache_help": (
    "Up to five sessions are retained in protected storage for seven days. Forgotten sessions remain on the server.",
    "Bis zu fünf Pläne bleiben sieben Tage im geschützten Speicher. Entfernte Offline-Kopien bleiben auf dem Server.",
  ),
  "cook_date": ("Date", "Datum"),
  "cook_time": ("Time", "Uhrzeit"),
  "cook_scope_explicit": (
    "Explicit ingredient assignment",
    "Ausdrückliche Zutatenzuordnung",
  ),
  "cook_scope_part": (
    "Ingredients of this preparation part",
    "Zutaten dieses Rezeptteils",
  ),
  "cooking_pending_review": (
    "Resolve the saved cooking actions before starting another change.",
    "Gleiche die gespeicherten Kochaktionen ab, bevor du eine weitere Änderung beginnst.",
  ),
  "cooking_clock_unverified": (
    "The clock changed or the device restarted. Reconnect before recording a timer action; saved deadlines remain unchanged.",
    "Die Uhr wurde geändert oder das Gerät neu gestartet. Verbinde dich vor einer Timeraktion; gespeicherte Fristen bleiben unverändert.",
  ),
  "cooking_cache_expired": (
    "The offline authorization window expired. Reconnect to reload cooking.",
    "Der Zeitraum für die Offline-Berechtigung ist abgelaufen. Lade den Kochstand erneut online.",
  ),
  "cooking_cache_full": (
    "Five sessions are already retained. Remove an unused offline copy first.",
    "Fünf Pläne sind bereits gespeichert. Entferne zuerst eine nicht benötigte Offline-Kopie.",
  ),
  "cooking_request_conflict": (
    "This request key belongs to another cooking action. Keep the original request and reconcile its result.",
    "Dieser Request-Schlüssel gehört zu einer anderen Kochaktion. Behalte den ursprünglichen Request und gleiche dessen Ergebnis ab.",
  ),
  "cooking_operation_expired": (
    "This reported action is older than seven days. Check the current session before recording another action.",
    "Diese erfasste Aktion ist älter als sieben Tage. Prüfe den aktuellen Plan vor einer neuen Aktion.",
  ),
  "cooking_review_expired": (
    "This saved review expired. Keep the draft and review the current schedule again.",
    "Diese Prüfung ist abgelaufen. Behalte den Entwurf und prüfe den aktuellen Zeitplan erneut.",
  ),
  "reminders_failed": (
    "Local reminders could not be saved or reconciled. Timers remain visible; retry after checking device settings.",
    "Lokale Erinnerungen konnten nicht gespeichert oder abgeglichen werden. Timer bleiben sichtbar; prüfe die Geräteeinstellungen und versuche es erneut.",
  ),
  "cook_empty_cache": (
    "No cooking copy available offline",
    "Keine Kochkopie offline verfügbar",
  ),
  "cook_recheck": (
    "Refresh current cooking state",
    "Aktuellen Kochstand laden",
  ),
};
