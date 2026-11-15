const captureMessages = <String, (String, String)>{
  'capture_zoom': ('Open original image', 'Originalbild öffnen'),
  'capture_assign': ('Assign recognized lines', 'Erkannte Zeilen zuordnen'),
  'capture_assign_help': (
    'Use + to assign original lines to title, ingredients or instructions. Unassigned text stays in the original source. Quantities and ingredient identities are reviewed next.',
    'Ordne Originalzeilen mit + dem Titel, den Zutaten oder Schritten zu. Nicht zugeordneter Text bleibt in der Originalquelle. Mengen und Zutatenidentitäten prüfst du anschließend.',
  ),
  'capture_unused': ('Original only', 'Nur im Original'),
  "capture_import": ("Import recipe", "Rezept importieren"),
  "capture_link": ("From a link", "Aus einem Link"),
  "capture_url": ("Original HTTPS recipe URL", "Originale HTTPS-Rezeptadresse"),
  "capture_fetch": ("Retrieve for review", "Zur Prüfung abrufen"),
  "capture_fetch_body": (
    "Allow this server to contact the website and validated redirects? It receives the URL and server IP, without your Nextcloud credentials.",
    "Darf der Server die Website und geprüfte Weiterleitungen kontaktieren? Sie erhält die URL und Server-IP, keine Nextcloud-Zugangsdaten.",
  ),
  "capture_manual": (
    "Enter privately without fetching",
    "Privat ohne Abruf eingeben",
  ),
  "capture_photo": ("Recipe or label photo", "Rezept- oder Etikettfoto"),
  "capture_camera": ("Take photo", "Fotografieren"),
  "capture_gallery": ("Choose photo", "Foto auswählen"),
  "capture_local": (
    "Recognition runs on this device, with bundled English/German models. No photo is sent to an OCR provider.",
    "Die Erkennung läuft auf diesem Gerät mit mitgelieferten deutschen/englischen Modellen. Kein Foto geht an einen OCR-Anbieter.",
  ),
  "capture_original": ("Original source", "Originalquelle"),
  "capture_rotate": ("Rotate and recognize", "Drehen und erkennen"),
  "capture_transcribe": (
    "Manual transcription (optional)",
    "Manuelle Abschrift (optional)",
  ),
  "capture_score": (
    "OCR engine score · not a correctness guarantee",
    "OCR-Qualitätswert · keine Richtigkeitsgarantie",
  ),
  "capture_unknown": (
    "Check every mapping, amount, serving count and step. Unknown units, durations, nutrition and allergy safety are not inferred.",
    "Prüfe jede Zuordnung, Menge, Portionszahl und jeden Schritt. Unbekannte Einheiten, Zeiten, Nährwerte und Allergiesicherheit werden nicht abgeleitet.",
  ),
  "capture_upload": (
    "Keep original in household",
    "Original im Haushalt ablegen",
  ),
  "capture_upload_body": (
    "Upload this original photo and its recognized/transcribed text to the selected Nextcloud household? Its members can read it. It stays private and is retained with the source.",
    "Dieses Originalfoto und den erkannten/abgeschriebenen Text in den gewählten Nextcloud-Haushalt hochladen? Seine Mitglieder können es lesen. Es bleibt privat und wird als Quelle aufbewahrt.",
  ),
  "capture_classify": (
    "Use the original to enter a title, ingredient lines and instructions below. One ingredient or instruction per line. The unchanged photo/text is kept separately.",
    "Übernimm aus dem Original Titel, Zutaten und Schritte. Pro Zeile eine Zutat oder Anweisung. Unverändertes Foto und Text bleiben getrennt erhalten.",
  ),
  "capture_review": ("Review recipe", "Rezept prüfen"),
  "capture_candidate_help": (
    "Choose the source recipe, then map every ingredient and review exact quantities. Source nutrition is not copied into the catalogue.",
    "Wähle das Quellrezept, ordne jede Zutat zu und prüfe genaue Mengen. Quellnährwerte werden nicht in den Katalog kopiert.",
  ),
  "capture_map": ("Choose matching ingredient", "Passende Zutat zuordnen"),
  "capture_mapped": (
    "Mapped · check amount below",
    "Zugeordnet · Menge unten prüfen",
  ),
  "capture_confirm": (
    "I checked source, mappings, quantities, servings, language, instructions and all time estimates.",
    "Ich habe Quelle, Zuordnungen, Mengen, Portionen, Sprache, Anweisungen und alle Zeitangaben geprüft.",
  ),
  "capture_frozen": (
    "The result of this save is unknown. Retry the unchanged request, or close this view and inspect your recipes before discarding. A retry also finds an already saved recipe after preview expiry.",
    "Das Speicherergebnis ist unklar. Wiederhole die unveränderte Anfrage oder schließe diese Ansicht und prüfe deine Rezepte vor dem Verwerfen. Die Wiederholung findet auch nach Vorschauablauf ein bereits gespeichertes Rezept.",
  ),
  "capture_retry": (
    "Retry confirmed import",
    "Bestätigten Import erneut senden",
  ),
  "capture_inspect": (
    "Close and inspect recipes",
    "Schließen und Rezepte prüfen",
  ),
  "capture_resume": (
    "Continue saved import",
    "Gespeicherten Import fortsetzen",
  ),
  "capture_discard": (
    "Discard saved import?",
    "Gespeicherten Import verwerfen?",
  ),
  "capture_discard_body": (
    "An uncertain save may already exist. Inspect your recipes first. Discarding removes only this device draft, not server records or uploaded source photos.",
    "Ein unklarer Speichervorgang kann bereits erfolgreich sein. Prüfe zuerst deine Rezepte. Verwerfen entfernt nur diesen Geräteentwurf, keine Serverdaten oder hochgeladenen Quellfotos.",
  ),
  "capture_incoming": ("Incoming shared text", "Geteilter Text eingegangen"),
  "capture_destination": (
    "Review using the selected account and household. Nothing is fetched automatically.",
    "Mit dem gewählten Konto und Haushalt prüfen. Es wird nichts automatisch abgerufen.",
  ),
  "capture_barcode": ("Scan product barcode", "Produktbarcode erfassen"),
  "capture_no_barcode": (
    "No supported barcode found. Try a sharp, upright photo or enter the GTIN manually.",
    "Kein unterstützter Barcode gefunden. Versuche ein scharfes, aufrechtes Foto oder gib die GTIN manuell ein.",
  ),
  "capture_label": ("Declared label values", "Deklarierte Etikettwerte"),
  "capture_label_help": (
    "Keep the original label and review its printed basis and values. These declarations do not replace ingredient nutrition or establish allergy safety.",
    "Bewahre das Originaletikett auf und prüfe Bezugsmenge und Werte. Diese Deklarationen ersetzen keine Zutatennährwerte und bestätigen keine Allergiesicherheit.",
  ),
  "capture_basis": (
    "Printed basis (e.g. per 100 ml or per portion)",
    "Gedruckte Bezugsmenge (z. B. je 100 ml oder Portion)",
  ),
  "capture_value": ("Add declared value", "Deklarierten Wert hinzufügen"),
  "capture_definition": (
    "Printed definition (optional; otherwise unknown)",
    "Gedruckte Definition (optional; sonst unbekannt)",
  ),
  "capture_qualifier": ("Printed qualifier", "Gedruckte Einschränkung"),
  "capture_exact": ("Declared value", "Deklarierter Wert"),
  "capture_less_than": ("Less than", "Weniger als"),
  "capture_approximate": ("Approximately", "Ungefähr"),
  "capture_label_confirm": (
    "I checked the original basis, names, values, units, qualifiers and definitions.",
    "Ich habe originale Bezugsmenge, Namen, Werte, Einheiten, Einschränkungen und Definitionen geprüft.",
  ),
  "capture_use_label": ("Use reviewed label", "Geprüftes Etikett übernehmen"),
  "capture_label_saved": (
    "Original and reviewed declarations saved",
    "Original und geprüfte Deklarationen gespeichert",
  ),
  "capture_private_source": (
    "Private photo source. Its content and image cannot be shared.",
    "Private Fotoquelle. Inhalt und Bild können nicht freigegeben werden.",
  ),
  "capture_share_source": ("Share original link", "Originallink teilen"),
  "capture_title": ("Original title", "Originaltitel"),
  "capture_author": ("Original author", "Originalautor"),
  "capture_language": ("Original language", "Originalsprache"),
  "capture_yield_text": ("Original yield", "Originalportionen"),
  "capture_prep_time": (
    "Original preparation time",
    "Originalvorbereitungszeit",
  ),
  "capture_cook_time": ("Original cooking time", "Originalgarzeit"),
  "capture_total_time": ("Original total time", "Originalgesamtzeit"),
  "capture_failed": (
    "Recognition failed. Your other fields are retained. Try again or enter manually.",
    "Erkennung fehlgeschlagen. Deine anderen Eingaben bleiben erhalten. Erneut versuchen oder manuell eingeben.",
  ),
  "capture_unavailable": (
    "Capture is unavailable on this platform. Use manual entry.",
    "Erfassung auf dieser Plattform nicht verfügbar. Nutze die manuelle Eingabe.",
  ),
  "capture_busy": (
    "Recognition is still running. Try again shortly.",
    "Die Erkennung läuft noch. Versuche es gleich erneut.",
  ),
  "capture_invalid": (
    "Capture exceeds supported limits or contains invalid data.",
    "Erfassung überschreitet unterstützte Grenzen oder enthält ungültige Daten.",
  ),
  "capture_image_missing": (
    "Upload the original photo in this household first.",
    "Lade das Originalfoto zuerst in diesem Haushalt hoch.",
  ),
  "capture_draft_invalid": (
    "Saved import has an unsupported format. Inspect your recipes before explicitly discarding this draft.",
    "Gespeicherter Import hat ein nicht unterstütztes Format. Prüfe vor dem ausdrücklichen Verwerfen deine Rezepte.",
  ),
  "capture_label_limit": (
    "Label text exceeds 4 KiB. Crop the label or transcribe only its printed declarations.",
    "Etiketttext überschreitet 4 KiB. Beschneide das Etikett oder schreibe nur seine gedruckten Angaben ab.",
  ),
  "import_preview_expired": (
    "This unsaved preview expired or the server restarted. Inspect recipes, then discard and review the source again.",
    "Diese ungespeicherte Vorschau ist abgelaufen oder der Server wurde neu gestartet. Prüfe die Rezepte, verwirf dann den Entwurf und prüfe die Quelle erneut.",
  ),
  "import_request_conflict": (
    "The saved request conflicts. Inspect your recipes before discarding or starting a new review.",
    "Die gespeicherte Anfrage steht im Konflikt. Prüfe deine Rezepte vor dem Verwerfen oder einer neuen Prüfung.",
  ),
  "import_url_invalid": (
    "Use an original HTTPS recipe URL without credentials.",
    "Nutze eine originale HTTPS-Rezeptadresse ohne Zugangsdaten.",
  ),
  "import_url_blocked": (
    "This URL is blocked. Use private manual entry without fetching.",
    "Diese Adresse ist gesperrt. Nutze die private Eingabe ohne Abruf.",
  ),
  "import_recipe_missing": (
    "No supported structured recipe found. Use private manual entry.",
    "Kein unterstütztes strukturiertes Rezept gefunden. Nutze die private Eingabe.",
  ),
  "import_fetch_failed": (
    "The website could not be retrieved. Retry or enter privately without fetching.",
    "Die Website konnte nicht abgerufen werden. Erneut versuchen oder privat ohne Abruf eingeben.",
  ),
  "import_recipe_invalid": (
    "Check required recipe fields and exact ingredient quantities.",
    "Prüfe Pflichtfelder und genaue Zutatenmengen.",
  ),
  "import_storage_failed": (
    "The server could not confirm the import. Retry the unchanged request.",
    "Der Server konnte den Import nicht bestätigen. Wiederhole die unveränderte Anfrage.",
  ),
  "import_write_required": (
    "An owner or editor must import this recipe.",
    "Ein Besitzer oder Bearbeiter muss dieses Rezept importieren.",
  ),
  "import_preview_capacity": (
    "Too many previews are open. Wait for expiry before starting another.",
    "Zu viele Vorschauen geöffnet. Warte vor einer weiteren auf den Ablauf.",
  ),
  "import_preview_unavailable": (
    "This preview belongs to another account or household.",
    "Diese Vorschau gehört zu einem anderen Konto oder Haushalt.",
  ),
  "import_review_required": (
    "Review the recipe and confirm before saving.",
    "Prüfe und bestätige das Rezept vor dem Speichern.",
  ),
  "import_fetch_cooldown": (
    "Wait a moment before retrieving another page.",
    "Warte kurz vor dem nächsten Seitenabruf.",
  ),
  "import_fetch_busy": (
    "Import is busy. Try again shortly.",
    "Import ausgelastet. Versuche es gleich erneut.",
  ),
  "import_target_blocked": (
    "The source or a redirect is blocked. Enter privately without fetching.",
    "Die Quelle oder eine Weiterleitung ist gesperrt. Gib das Rezept privat ohne Abruf ein.",
  ),
  "import_redirect_blocked": (
    "The source or a redirect is blocked. Enter privately without fetching.",
    "Die Quelle oder eine Weiterleitung ist gesperrt. Gib das Rezept privat ohne Abruf ein.",
  ),
  "import_redirect_invalid": (
    "The source or a redirect is blocked. Enter privately without fetching.",
    "Die Quelle oder eine Weiterleitung ist gesperrt. Gib das Rezept privat ohne Abruf ein.",
  ),
  "import_redirect_limit": (
    "The source or a redirect is blocked. Enter privately without fetching.",
    "Die Quelle oder eine Weiterleitung ist gesperrt. Gib das Rezept privat ohne Abruf ein.",
  ),
  "import_robots_blocked": (
    "The source policy does not permit this retrieval. Use private manual entry.",
    "Die Quellrichtlinie erlaubt diesen Abruf nicht. Nutze die private manuelle Eingabe.",
  ),
  "import_source_blocked": (
    "The source policy does not permit this retrieval. Use private manual entry.",
    "Die Quellrichtlinie erlaubt diesen Abruf nicht. Nutze die private manuelle Eingabe.",
  ),
  "import_policy_unavailable": (
    "The source policy does not permit this retrieval. Use private manual entry.",
    "Die Quellrichtlinie erlaubt diesen Abruf nicht. Nutze die private manuelle Eingabe.",
  ),
  "import_no_recipe": (
    "No supported structured recipe could be read. Enter privately without fetching.",
    "Kein unterstütztes strukturiertes Rezept lesbar. Gib es privat ohne Abruf ein.",
  ),
  "import_type_unsupported": (
    "No supported structured recipe could be read. Enter privately without fetching.",
    "Kein unterstütztes strukturiertes Rezept lesbar. Gib es privat ohne Abruf ein.",
  ),
  "import_encoding_unsupported": (
    "No supported structured recipe could be read. Enter privately without fetching.",
    "Kein unterstütztes strukturiertes Rezept lesbar. Gib es privat ohne Abruf ein.",
  ),
  "import_parser_failed": (
    "No supported structured recipe could be read. Enter privately without fetching.",
    "Kein unterstütztes strukturiertes Rezept lesbar. Gib es privat ohne Abruf ein.",
  ),
  "import_dns_failed": (
    "The source is unavailable. Retry or enter privately without fetching.",
    "Die Quelle ist nicht erreichbar. Erneut versuchen oder privat ohne Abruf eingeben.",
  ),
  "import_timeout": (
    "The source is unavailable. Retry or enter privately without fetching.",
    "Die Quelle ist nicht erreichbar. Erneut versuchen oder privat ohne Abruf eingeben.",
  ),
  "import_transport_failed": (
    "The source is unavailable. Retry or enter privately without fetching.",
    "Die Quelle ist nicht erreichbar. Erneut versuchen oder privat ohne Abruf eingeben.",
  ),
  "import_source_unavailable": (
    "The source is unavailable. Retry or enter privately without fetching.",
    "Die Quelle ist nicht erreichbar. Erneut versuchen oder privat ohne Abruf eingeben.",
  ),
  "import_response_large": (
    "The source exceeds import limits. Enter a bounded private copy manually.",
    "Die Quelle überschreitet die Importgrenzen. Gib eine begrenzte private Kopie manuell ein.",
  ),
  "import_content_large": (
    "The source exceeds import limits. Enter a bounded private copy manually.",
    "Die Quelle überschreitet die Importgrenzen. Gib eine begrenzte private Kopie manuell ein.",
  ),
  "import_consent_required": (
    "Check the source fields and confirm retrieval only if you agree. If the server clock is invalid, contact its administrator.",
    "Prüfe die Quellfelder und bestätige den Abruf nur bei Zustimmung. Bei ungültiger Serverzeit wende dich an den Administrator.",
  ),
  "import_invalid_input": (
    "Check the source fields and confirm retrieval only if you agree. If the server clock is invalid, contact its administrator.",
    "Prüfe die Quellfelder und bestätige den Abruf nur bei Zustimmung. Bei ungültiger Serverzeit wende dich an den Administrator.",
  ),
  "import_clock_invalid": (
    "Check the source fields and confirm retrieval only if you agree. If the server clock is invalid, contact its administrator.",
    "Prüfe die Quellfelder und bestätige den Abruf nur bei Zustimmung. Bei ungültiger Serverzeit wende dich an den Administrator.",
  ),
};
