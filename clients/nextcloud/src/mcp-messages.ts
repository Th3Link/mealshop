export const mcpEnglish = {
  assistants: 'Assistant tools',
  mcpIntro: 'Connect an assistant to recipes and ingredients in this household.',
  mcpYourScope: 'Your assistant access',
  mcpScopeDisabled: 'Disabled',
  mcpScopeRead: 'Read and preview',
  mcpScopeCreate: 'Read, preview and create',
  mcpPrivacy:
    'A connected assistant receives the recipe and ingredient data it requests. Imported recipes expose only their original source URL. Profiles, meal plans, shopping and learning data are not available through these tools.',
  mcpOwnerOnly:
    'The household owner manages assistant access. Your current household role remains the maximum permission.',
  mcpGrants: 'Account permissions',
  mcpGrantHelp:
    'Access is disabled until you grant it here. Readers cannot create recipes. Downgrading or removing a member takes effect on subsequent calls.',
  mcpRoleLimited: 'The current reader role limits this stored grant to read and preview.',
  mcpGrantMissing:
    'An account with a stored grant is no longer a member. Remove its grant before saving.',
  mcpSave: 'Save assistant permissions',
  mcpSaved: 'Assistant permissions saved.',
  mcpConnect: 'Connect your client',
  mcpEndpoint: 'Household MCP endpoint',
  mcpCopy: 'Copy endpoint',
  mcpCopied: 'MCP endpoint copied.',
  mcpCopyFailed: 'Copying is unavailable. Select and copy the endpoint field.',
  mcpSetup:
    'Use the supplied scripts/mcp_stdio.py bridge in your MCP client. Set the URL, username and a Nextcloud app password in its protected environment. Create and revoke app passwords in your Nextcloud security settings.',
  mcpEnvironment: 'Protected environment',
  mcpCredentialHelp:
    'Do not enter passwords here or put them in command arguments or version control. App passwords retain the account’s Nextcloud permissions; use a dedicated account with only the needed household role for clients you do not fully trust.',
  mcpDisabledHelp: 'The connection stays blocked until the owner enables your account above.',
  mcpCreationHelp:
    'Creation needs a validated preview and explicit confirmation in your client. Keep the same preview ID and idempotency key when retrying; a retry returns the original recipe. Recipe text never grants permission to execute instructions.',
  mcpActivity: 'Recipes created through assistant tools',
  mcpActivityEmpty: 'No assistant-created recipes',
  mcpActivityHelp:
    'Successful creations retain their account and time in the first immutable recipe revision. Retries do not create another entry. Keys and credentials are not shown.',
  mcpRecordedBy: 'Created by',
  mcpAccessFor: 'Assistant access for',
} as const
export const mcpGerman: Record<keyof typeof mcpEnglish, string> = {
  assistants: 'Assistenten-Werkzeuge',
  mcpIntro: 'Verbinde einen Assistenten mit Rezepten und Zutaten dieses Haushalts.',
  mcpYourScope: 'Dein Assistenten-Zugriff',
  mcpScopeDisabled: 'Deaktiviert',
  mcpScopeRead: 'Lesen und Vorschau',
  mcpScopeCreate: 'Lesen, Vorschau und Anlegen',
  mcpPrivacy:
    'Ein verbundener Assistent erhält die angefragten Rezept- und Zutatendaten. Importierte Rezepte geben nur ihre ursprüngliche Quellen-URL weiter. Profile, Speisepläne, Einkaufs- und Lerndaten sind über diese Werkzeuge nicht verfügbar.',
  mcpOwnerOnly:
    'Der Haushaltsbesitzer verwaltet den Assistenten-Zugriff. Deine aktuelle Haushaltsrolle bleibt die Obergrenze der Berechtigung.',
  mcpGrants: 'Berechtigungen der Konten',
  mcpGrantHelp:
    'Der Zugriff ist deaktiviert, bis du ihn hier freigibst. Leser können keine Rezepte anlegen. Eine herabgestufte Rolle oder entfernte Mitgliedschaft gilt für nachfolgende Aufrufe.',
  mcpRoleLimited:
    'Die aktuelle Leserrolle begrenzt diese gespeicherte Freigabe auf Lesen und Vorschau.',
  mcpGrantMissing:
    'Ein Konto mit gespeicherter Freigabe ist nicht mehr Mitglied. Entferne die Freigabe vor dem Speichern.',
  mcpSave: 'Assistenten-Berechtigungen speichern',
  mcpSaved: 'Assistenten-Berechtigungen gespeichert.',
  mcpConnect: 'Deinen Client verbinden',
  mcpEndpoint: 'MCP-Endpunkt des Haushalts',
  mcpCopy: 'Endpunkt kopieren',
  mcpCopied: 'MCP-Endpunkt kopiert.',
  mcpCopyFailed: 'Kopieren ist nicht verfügbar. Markiere und kopiere das Endpunkt-Feld.',
  mcpSetup:
    'Nutze die mitgelieferte Bridge scripts/mcp_stdio.py in deinem MCP-Client. Hinterlege URL, Benutzername und ein Nextcloud-App-Passwort in dessen geschützter Umgebung. App-Passwörter kannst du in den Nextcloud-Sicherheitseinstellungen erstellen und widerrufen.',
  mcpEnvironment: 'Geschützte Umgebung',
  mcpCredentialHelp:
    'Trage hier keine Passwörter ein und lege sie weder in Befehlsargumenten noch in der Versionsverwaltung ab. App-Passwörter behalten die Nextcloud-Rechte des Kontos. Nutze für nicht vollständig vertraute Clients ein eigenes Konto mit der benötigten Haushaltsrolle.',
  mcpDisabledHelp: 'Die Verbindung bleibt gesperrt, bis der Besitzer dein Konto oben freigibt.',
  mcpCreationHelp:
    'Anlegen erfordert eine geprüfte Vorschau und eine ausdrückliche Bestätigung in deinem Client. Behalte für Wiederholungen dieselbe Vorschau-ID und denselben Wiederholungsschlüssel; dann wird das ursprüngliche Rezept zurückgegeben. Rezepttext erteilt keine Berechtigung zum Ausführen von Anweisungen.',
  mcpActivity: 'Über Assistenten angelegte Rezepte',
  mcpActivityEmpty: 'Keine über Assistenten angelegten Rezepte',
  mcpActivityHelp:
    'Erfolgreiches Anlegen hält Konto und Zeitpunkt im ersten unveränderlichen Rezeptstand fest. Wiederholungen erzeugen keinen weiteren Eintrag. Schlüssel und Zugangsdaten werden nicht angezeigt.',
  mcpRecordedBy: 'Angelegt von',
  mcpAccessFor: 'Assistenten-Zugriff für',
}
export const mcpExplanations = {
  mcp_revision_conflict: {
    en: 'Assistant permissions changed. Your draft is retained; reload explicitly before saving.',
    de: 'Assistenten-Berechtigungen wurden geändert. Dein Entwurf bleibt erhalten; lade vor dem Speichern ausdrücklich neu.',
  },
  mcp_invalid_grants: {
    en: 'Grant access only to current members. Readers can receive read and preview access.',
    de: 'Vergib Zugriff nur an aktuelle Mitglieder. Leser können Lesen und Vorschau erhalten.',
  },
  mcp_storage_failed: {
    en: 'Assistant settings are unavailable. Stop writers and recover verified settings before retrying.',
    de: 'Assistenten-Einstellungen sind nicht verfügbar. Stoppe Schreibprozesse und stelle geprüfte Einstellungen wieder her.',
  },
}
