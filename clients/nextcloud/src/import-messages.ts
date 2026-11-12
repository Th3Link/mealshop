export const importEnglish = {
  importStartAgain: 'Start import again',
  importRecipe: 'Import from link',
  importIntro:
    'Add a source recipe to your household after reviewing every ingredient and quantity.',
  importUrl: 'Recipe URL',
  importConsent: 'Allow this server to contact the public source website and validated redirects.',
  importPrivacy:
    'The website receives the link and this server’s IP address. No Nextcloud credentials are sent. Login, paywall and robots restrictions are respected. Images and scripts are not loaded.',
  importFetch: 'Fetch recipe preview',
  importManual: 'Enter privately without fetching',
  importChoose: 'Recipe on this page',
  importUnnamed: 'Untitled source recipe',
  importContinue: 'Review this recipe',
  importNotSaved: 'Preview only · nothing has been saved.',
  importReviewTitle: 'Review imported recipe',
  importSource: 'Original source',
  importFinal: 'Retrieved page',
  importOriginal: 'Original source fields',
  importUnknown: 'Not supplied by the source',
  importSourceTitle: 'Source title',
  importSourceAuthor: 'Source author',
  importSourceLanguage: 'Source language',
  importSourceYield: 'Source yield',
  importSourcePrep: 'Source preparation time',
  importSourceCook: 'Source cooking time',
  importSourceTotal: 'Source total time',
  importSourceIngredient: 'Source ingredient',
  importSourceInstructions: 'Source instructions',
  importSourceIngredients: 'Source ingredients',
  importReviewHelp:
    'Match every ingredient and enter missing amounts or servings. Source times are not allocated to steps. All step times you enter, including zero, are your declarations; missing original times stay unknown below. Review the recipe language without silently translating its text.',
  importReviewed:
    'I reviewed all ingredient mappings, quantities, servings, language, steps and time estimates, including any zero values.',
  importSave: 'Save reviewed import',
  importRetry: 'Retry confirmed import',
  importFrozen:
    'This confirmed request is frozen. Retrying uses the same recipe and key to avoid duplicates. If you leave or reload, check the recipe list before starting a new import.',
  importManualHelp:
    'No website is contacted. Enter your private copy in the usual editor; the source URL remains attached and sharing exposes only that link.',
  importSourceNote:
    'Original fields are retained separately from your reviewed recipe. Missing source values stay unknown; source nutrition, allergy and diet claims are not adopted.',
} as const
export const importGerman: Record<keyof typeof importEnglish, string> = {
  importStartAgain: 'Import neu beginnen',
  importRecipe: 'Über Link importieren',
  importIntro:
    'Übernimm ein Quellrezept in deinen Haushalt, nachdem du alle Zutaten und Mengen geprüft hast.',
  importUrl: 'Rezept-URL',
  importConsent:
    'Diesem Server erlauben, die öffentliche Quellseite und geprüfte Weiterleitungen abzurufen.',
  importPrivacy:
    'Die Webseite erhält den Link und die IP-Adresse dieses Servers. Nextcloud-Zugangsdaten werden nicht gesendet. Login-, Bezahlschranken- und Robots-Regeln werden beachtet. Bilder und Skripte werden nicht geladen.',
  importFetch: 'Rezeptvorschau abrufen',
  importManual: 'Ohne Abruf privat eingeben',
  importChoose: 'Rezept auf dieser Seite',
  importUnnamed: 'Quellrezept ohne Titel',
  importContinue: 'Dieses Rezept prüfen',
  importNotSaved: 'Nur Vorschau · noch nichts gespeichert.',
  importReviewTitle: 'Importiertes Rezept prüfen',
  importSource: 'Ursprüngliche Quelle',
  importFinal: 'Abgerufene Seite',
  importOriginal: 'Ursprüngliche Quellenangaben',
  importUnknown: 'Von der Quelle nicht angegeben',
  importSourceTitle: 'Quelltitel',
  importSourceAuthor: 'Autor der Quelle',
  importSourceLanguage: 'Quellsprache',
  importSourceYield: 'Ergiebigkeit laut Quelle',
  importSourcePrep: 'Vorbereitungszeit laut Quelle',
  importSourceCook: 'Zubereitungszeit laut Quelle',
  importSourceTotal: 'Gesamtzeit laut Quelle',
  importSourceIngredient: 'Zutat laut Quelle',
  importSourceInstructions: 'Anleitung laut Quelle',
  importSourceIngredients: 'Zutaten laut Quelle',
  importReviewHelp:
    'Ordne jede Zutat zu und ergänze fehlende Mengen oder Portionen. Quellenzeiten werden nicht auf Schritte verteilt. Alle eingegebenen Schrittzeiten, auch null, sind deine Angaben; fehlende Originalzeiten bleiben unten unbekannt. Prüfe die Rezeptsprache, ohne den Text stillschweigend zu übersetzen.',
  importReviewed:
    'Ich habe alle Zutatenzuordnungen, Mengen, Portionen, Sprache, Schritte und Zeitschätzungen einschließlich etwaiger Nullwerte geprüft.',
  importSave: 'Geprüften Import speichern',
  importRetry: 'Bestätigten Import wiederholen',
  importFrozen:
    'Diese bestätigte Anfrage ist festgeschrieben. Wiederholen verwendet dasselbe Rezept und denselben Schlüssel, damit keine Duplikate entstehen. Prüfe nach Verlassen oder Neuladen die Rezeptliste, bevor du einen neuen Import startest.',
  importManualHelp:
    'Es wird keine Webseite kontaktiert. Trage deine private Kopie im üblichen Editor ein; die Quell-URL bleibt erhalten und Freigaben geben nur diesen Link weiter.',
  importSourceNote:
    'Originalangaben bleiben getrennt von deinem geprüften Rezept erhalten. Fehlende Quellenwerte bleiben unbekannt; Nährwert-, Allergie- und Ernährungsaussagen der Quelle werden nicht übernommen.',
}
const explanation = (en: string, de: string) => ({ en, de })
export const importExplanations = {
  import_url_invalid: explanation(
    'Use a public HTTPS recipe URL on port 443 without credentials.',
    'Verwende eine öffentliche HTTPS-Rezept-URL auf Port 443 ohne Zugangsdaten.',
  ),
  import_target_blocked: explanation(
    'This target resolves to a local, private or reserved address. Enter privately without fetching.',
    'Dieses Ziel verweist auf eine lokale, private oder reservierte Adresse. Du kannst ohne Abruf privat eingeben.',
  ),
  import_consent_required: explanation(
    'Confirm contact with the source before fetching.',
    'Bestätige vor dem Abruf die Kontaktaufnahme mit der Quelle.',
  ),
  import_no_recipe: explanation(
    'No supported structured recipe was found. Enter your private copy manually.',
    'Kein unterstütztes strukturiertes Rezept gefunden. Trage deine private Kopie manuell ein.',
  ),
  import_source_blocked: explanation(
    'The source restricts access or requires login. No restriction is bypassed; manual private entry is available.',
    'Die Quelle beschränkt den Zugriff oder verlangt eine Anmeldung. Beschränkungen werden nicht umgangen; private manuelle Eingabe ist möglich.',
  ),
  import_robots_blocked: explanation(
    'The source disallows this automated request. Manual private entry remains available.',
    'Die Quelle untersagt diesen automatischen Abruf. Private manuelle Eingabe bleibt möglich.',
  ),
  import_policy_unavailable: explanation(
    'Source policy could not be safely verified. Try later or enter privately without fetching.',
    'Die Quellenregeln konnten nicht sicher geprüft werden. Versuche es später oder gib ohne Abruf privat ein.',
  ),
  import_redirect_blocked: explanation(
    'A redirect uses an unsafe target. Enter privately without fetching.',
    'Eine Weiterleitung verwendet ein unsicheres Ziel. Du kannst ohne Abruf privat eingeben.',
  ),
  import_redirect_limit: explanation(
    'The source redirects too often. Use its final public recipe URL or enter manually.',
    'Die Quelle leitet zu oft weiter. Verwende die endgültige öffentliche Rezept-URL oder die manuelle Eingabe.',
  ),
  import_timeout: explanation(
    'The source request timed out. Try later or enter manually.',
    'Der Quellenabruf hat zu lange gedauert. Versuche es später oder verwende die manuelle Eingabe.',
  ),
  import_response_large: explanation(
    'The source response exceeds the safe limit. Enter your private copy manually.',
    'Die Quellantwort überschreitet die sichere Größe. Trage deine private Kopie manuell ein.',
  ),
  import_content_large: explanation(
    'The extracted source is too large to review safely. Enter the needed recipe manually.',
    'Die extrahierte Quelle ist für eine sichere Prüfung zu groß. Trage das gewünschte Rezept manuell ein.',
  ),
  import_type_unsupported: explanation(
    'This source format is unsupported. Enter your private copy manually.',
    'Dieses Quellenformat wird nicht unterstützt. Trage deine private Kopie manuell ein.',
  ),
  import_encoding_unsupported: explanation(
    'The source encoding is unsupported. Enter your private copy manually.',
    'Die Zeichenkodierung der Quelle wird nicht unterstützt. Trage deine private Kopie manuell ein.',
  ),
  import_dns_failed: explanation(
    'The public source address could not be resolved. Try later or enter manually.',
    'Die öffentliche Quelladresse konnte nicht aufgelöst werden. Versuche es später oder verwende die manuelle Eingabe.',
  ),
  import_transport_failed: explanation(
    'The public source could not be safely reached. Try later or enter manually.',
    'Die öffentliche Quelle konnte nicht sicher erreicht werden. Versuche es später oder verwende die manuelle Eingabe.',
  ),
  import_source_unavailable: explanation(
    'The source is unavailable. Try later or enter manually.',
    'Die Quelle ist nicht verfügbar. Versuche es später oder verwende die manuelle Eingabe.',
  ),
  import_fetch_cooldown: explanation(
    'An import is running or was just requested. Wait ten seconds before fetching again.',
    'Ein Import läuft oder wurde gerade angefragt. Warte vor einem erneuten Abruf zehn Sekunden.',
  ),
  import_fetch_busy: explanation(
    'All import slots are busy. Try again shortly.',
    'Alle Importplätze sind belegt. Versuche es gleich erneut.',
  ),
  import_preview_capacity: explanation(
    'The preview limit has been reached. Existing previews expire after 30 minutes.',
    'Die Vorschaugrenze ist erreicht. Bestehende Vorschauen laufen nach 30 Minuten ab.',
  ),
  import_preview_expired: explanation(
    'This preview expired or the server restarted. Your draft stays here. Check saved recipes before explicitly fetching/reviewing a new preview.',
    'Diese Vorschau ist abgelaufen oder der Server wurde neu gestartet. Dein Entwurf bleibt erhalten. Prüfe gespeicherte Rezepte, bevor du ausdrücklich eine neue Vorschau abrufst und prüfst.',
  ),
  import_preview_unavailable: explanation(
    'This preview belongs to another account.',
    'Diese Vorschau gehört zu einem anderen Konto.',
  ),
  import_request_conflict: explanation(
    'This request key already belongs to another confirmed import. Check saved recipes before starting again.',
    'Dieser Schlüssel gehört bereits zu einem anderen bestätigten Import. Prüfe gespeicherte Rezepte, bevor du neu beginnst.',
  ),
  import_recipe_invalid: explanation(
    'Check the required recipe fields, ingredient mappings, exact quantities and source URL.',
    'Prüfe Pflichtfelder, Zutatenzuordnungen, genaue Mengen und Quell-URL.',
  ),
  import_storage_failed: explanation(
    'Import storage is unavailable. Retain this request and follow recovery instructions before retrying.',
    'Der Importspeicher ist nicht verfügbar. Behalte diese Anfrage und befolge vor dem Wiederholen die Wiederherstellungsanleitung.',
  ),
  import_review_required: explanation(
    'Review and explicitly confirm the recipe before saving.',
    'Prüfe und bestätige das Rezept vor dem Speichern ausdrücklich.',
  ),
}
