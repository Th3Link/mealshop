const deviceMessages = <String, (String, String)>{
  'device_dataset_changed': (
    'Local data was restored. Reopen on-device mode before continuing.',
    'Lokale Daten wurden wiederhergestellt. Öffne den lokalen Modus erneut, bevor du fortfährst.',
  ),
  'device_backup_access_required': (
    'Saved activities need explicit read access before a whole-device backup. Open More → Activities → Access for the affected profiles. No data was omitted.',
    'Gespeicherte Aktivitäten brauchen vor einem vollständigen Backup ausdrückliche Leserechte. Öffne Mehr → Aktivitäten → Zugriff für die betreffenden Profile. Es wurden keine Daten ausgelassen.',
  ),
  'backup_size': ('Saved data', 'Gespeicherte Daten'),
  'backup_continue': ('Continue', 'Weiter'),
  'local_mode_title': ('Start with Mealshop', 'Mit Mealshop starten'),
  'local_mode_intro': (
    'Use Mealshop on this device or connect your Nextcloud.',
    'Nutze Mealshop auf diesem Gerät oder verbinde deine Nextcloud.',
  ),
  'local_use': ('Use on this device', 'Auf diesem Gerät nutzen'),
  'local_intro': (
    'Saved locally, with encrypted backup and restore.',
    'Lokal gespeichert, mit verschlüsselter Sicherung und Wiederherstellung.',
  ),
  'local_account': ('On this device', 'Auf diesem Gerät'),
  'local_data': ('Local data', 'Lokale Daten'),
  'local_sharing_body': (
    'Account sharing requires a Nextcloud connection.',
    'Für Kontofreigaben brauchst du eine Nextcloud-Verbindung.',
  ),
  'local_exit': ('Connection options', 'Verbindung auswählen'),
  'local_exit_body': (
    'Saved local households remain on this device. Switching modes does not transfer them to a server. Unsaved drafts and pending actions will be cleared.',
    'Gespeicherte lokale Haushalte bleiben auf diesem Gerät. Der Moduswechsel überträgt sie nicht auf einen Server. Ungespeicherte Entwürfe und ausstehende Aktionen werden entfernt.',
  ),
  'local_privacy': (
    'Saved data stays in this app’s private storage. Backups contain saved local records and images, protected by your passphrase. Server credentials and unsaved drafts are excluded.',
    'Gespeicherte Daten liegen im privaten App-Speicher. Backups enthalten gespeicherte lokale Einträge und Bilder, geschützt durch dein Passwort. Serverzugangsdaten und ungespeicherte Entwürfe sind ausgenommen.',
  ),
  'requires_server': (
    'This action requires a Nextcloud connection.',
    'Diese Aktion braucht eine Nextcloud-Verbindung.',
  ),
  'device_unavailable': (
    'The on-device core is not available in this build.',
    'Der lokale Kern ist in diesem Build nicht verfügbar.',
  ),
  'device_engine_failed': (
    'Local data could not be opened. Try again; saved data has not been reset.',
    'Lokale Daten konnten nicht geöffnet werden. Versuche es erneut; gespeicherte Daten wurden nicht zurückgesetzt.',
  ),
  'device_storage_failed': (
    'Local storage is unavailable. Check free space and try again.',
    'Der lokale Speicher ist nicht verfügbar. Prüfe den freien Platz und versuche es erneut.',
  ),
  'device_recovery_required': (
    'Local data needs recovery. Restore a verified backup; current files are kept until you confirm replacement.',
    'Lokale Daten müssen wiederhergestellt werden. Nutze ein geprüftes Backup; die bisherigen Dateien bleiben bis zur bestätigten Wiederherstellung erhalten.',
  ),
  'device_in_use': (
    'Another local writer is still stopping. Try again shortly.',
    'Ein anderer lokaler Schreibprozess wird noch beendet. Versuche es gleich erneut.',
  ),
  'device_protocol_failed': (
    'The installed app and local core are incompatible. Install a matching build.',
    'Die installierte App und der lokale Kern passen nicht zusammen. Installiere einen passenden Build.',
  ),
  'device_format_invalid': (
    'This local dataset cannot be opened by this app version.',
    'Dieser lokale Datenbestand kann mit dieser App-Version nicht geöffnet werden.',
  ),
  "backups": ("Backup and restore", "Sichern und wiederherstellen"),
  "backup_caption": (
    "Encrypted files you can keep and restore on another device.",
    "Verschlüsselte Dateien zum Aufbewahren und Wiederherstellen auf einem anderen Gerät.",
  ),
  "backup_intro": (
    "Back up all saved local households, recipes, images and records. Unsaved drafts, server accounts and device reminder settings are excluded.",
    "Sichere alle gespeicherten lokalen Haushalte, Rezepte, Bilder und Einträge. Ungespeicherte Entwürfe, Serverkonten und Geräteeinstellungen für Erinnerungen sind ausgenommen.",
  ),
  "backup_last": ("Last completed backup", "Letzte abgeschlossene Sicherung"),
  "backup_never": (
    "No completed backup yet.",
    "Noch keine abgeschlossene Sicherung.",
  ),
  "backup_create": (
    "Create encrypted backup",
    "Verschlüsseltes Backup erstellen",
  ),
  "backup_restore": ("Restore from backup", "Aus Backup wiederherstellen"),
  "backup_passphrase_notice": (
    "Keep your backup and passphrase safe. The passphrase cannot be recovered. App updates preserve local data; uninstalling the app removes it.",
    "Bewahre Backup und Passwort sicher auf. Das Passwort kann nicht wiederhergestellt werden. App-Updates behalten lokale Daten; eine Deinstallation entfernt sie.",
  ),
  "backup_encrypting": (
    "Encrypting saved local data…",
    "Gespeicherte lokale Daten werden verschlüsselt …",
  ),
  "backup_destination": (
    "Choose where to save your encrypted backup and wait for saving to finish.",
    "Wähle den Speicherort für dein verschlüsseltes Backup und warte, bis es vollständig gespeichert ist.",
  ),
  "backup_saved": (
    "Encrypted backup saved.",
    "Verschlüsseltes Backup gespeichert.",
  ),
  "backup_select": (
    "Choose an encrypted Mealshop backup.",
    "Wähle ein verschlüsseltes Mealshop-Backup.",
  ),
  "backup_verifying": (
    "Decrypting and checking the backup. Current data stays active.",
    "Das Backup wird entschlüsselt und geprüft. Die bisherigen Daten bleiben aktiv.",
  ),
  "backup_restore_preview": ("Backup verified", "Backup geprüft"),
  "backup_replace_body": (
    "Replace all saved local data with this backup? This affects every local household. Unsaved drafts and pending actions will be discarded. Server data stays separate. Saved timer deadlines are retained; enable device reminders again afterwards.",
    "Alle gespeicherten lokalen Daten durch dieses Backup ersetzen? Dies betrifft jeden lokalen Haushalt. Ungespeicherte Entwürfe und ausstehende Aktionen werden verworfen. Serverdaten bleiben separat. Gespeicherte Timer behalten ihre Fristen; Geräteerinnerungen musst du anschließend erneut aktivieren.",
  ),
  "backup_replace": ("Replace local data", "Lokale Daten ersetzen"),
  "backup_restoring": (
    "Activating the verified backup…",
    "Das geprüfte Backup wird aktiviert …",
  ),
  "backup_files": ("Saved files", "Gespeicherte Dateien"),
  "backup_password_create": ("Protect your backup", "Backup schützen"),
  "backup_password_enter": ("Unlock backup", "Backup entsperren"),
  "backup_password": ("Backup passphrase", "Backup-Passwort"),
  "backup_password_confirm": ("Repeat passphrase", "Passwort wiederholen"),
  "backup_password_help": (
    "Choose at least 12 characters. You will need this passphrase to restore your backup.",
    "Wähle mindestens 12 Zeichen. Du brauchst dieses Passwort zur Wiederherstellung.",
  ),
  "backup_password_restore_help": (
    "Enter the passphrase used when this backup was created.",
    "Gib das Passwort ein, mit dem dieses Backup erstellt wurde.",
  ),
  "backup_password_show": ("Show passphrase", "Passwort anzeigen"),
  "backup_password_hide": ("Hide passphrase", "Passwort verbergen"),
  "backup_password_mismatch": (
    "Passphrases do not match.",
    "Die Passwörter stimmen nicht überein.",
  ),
  "device_passphrase_invalid": (
    "Use 12–256 characters for your passphrase.",
    "Verwende 12–256 Zeichen für dein Passwort.",
  ),
  "device_restore_password": (
    "The passphrase could not unlock this backup. Try again or choose another file.",
    "Das Passwort konnte dieses Backup nicht entsperren. Versuche es erneut oder wähle eine andere Datei.",
  ),
  "device_restore_invalid": (
    "This backup is incomplete, damaged or contains unsupported data. Current data has been kept.",
    "Dieses Backup ist unvollständig, beschädigt oder enthält nicht unterstützte Daten. Die bisherigen Daten bleiben erhalten.",
  ),
  "device_restore_version": (
    "This backup format needs a different app version. Current data has been kept.",
    "Dieses Backup-Format benötigt eine andere App-Version. Die bisherigen Daten bleiben erhalten.",
  ),
  "device_restore_expired": (
    "Local data changed after the preview. Check the backup again before replacing it.",
    "Lokale Daten wurden seit der Vorschau geändert. Prüfe das Backup vor dem Ersetzen erneut.",
  ),
  "device_backup_failed": (
    "The encrypted backup could not be created. Check free space and try again.",
    "Das verschlüsselte Backup konnte nicht erstellt werden. Prüfe freien Speicher und versuche es erneut.",
  ),
  "device_backup_capacity": (
    "This backup exceeds the supported limit: 512 MiB total, 64 MiB per file or 20,000 files. Current data has been kept.",
    "Dieses Backup überschreitet die unterstützte Grenze: insgesamt 512 MiB, 64 MiB je Datei oder 20.000 Dateien. Die bisherigen Daten bleiben erhalten.",
  ),
  "device_document_failed": (
    "The selected document could not be read or saved. Check space and provider access; no backup success was recorded.",
    "Die gewählte Datei konnte nicht gelesen oder gespeichert werden. Prüfe Speicherplatz und Zugriff; die Sicherung wurde nicht als erfolgreich vermerkt.",
  ),
  "device_document_busy": (
    "Finish or cancel the open document picker first.",
    "Schließe zuerst die geöffnete Dateiauswahl ab oder brich sie ab.",
  ),
  "device_cleanup_failed": (
    "Temporary files could not be removed. Restart the app to retry cleanup.",
    "Temporäre Dateien konnten nicht entfernt werden. Starte die App für einen erneuten Versuch neu.",
  ),
  "device_request_invalid": (
    "The local request could not be processed. Update the app and try again.",
    "Die lokale Anfrage konnte nicht verarbeitet werden. Aktualisiere die App und versuche es erneut.",
  ),
  "device_response_too_large": (
    "This local response is too large. Narrow your selection and try again.",
    "Diese lokale Antwort ist zu groß. Grenze deine Auswahl ein und versuche es erneut.",
  ),
  "device_frame_too_large": (
    "This local request is too large. Use a smaller image or selection.",
    "Diese lokale Anfrage ist zu groß. Verwende ein kleineres Bild oder eine kleinere Auswahl.",
  ),
  "device_capacity": (
    "The local data limit has been reached.",
    "Die Grenze für lokale Daten wurde erreicht.",
  ),
  "device_random_failed": (
    "Secure randomness is unavailable. Restart your device and try again.",
    "Sichere Zufallswerte sind nicht verfügbar. Starte dein Gerät neu und versuche es erneut.",
  ),
  "device_clock_invalid": (
    "Set a valid device date before creating a backup.",
    "Stelle vor dem Sichern ein gültiges Gerätedatum ein.",
  ),
  "backup_restored_refresh": (
    "The backup was activated. Reopen on-device mode to refresh the app.",
    "Das Backup wurde aktiviert. Öffne den lokalen Modus erneut, um die App zu aktualisieren.",
  ),
  "local_saved": ("Saved on this device", "Auf diesem Gerät gespeichert"),
  "local_unavailable": (
    "Local data is currently unavailable. Try reopening it.",
    "Lokale Daten sind gerade nicht verfügbar. Versuche, sie erneut zu öffnen.",
  ),
  "local_content": (
    "Create your first local recipe.",
    "Erstelle dein erstes lokales Rezept.",
  ),
};

const localMessageOverrides = <String, (String, String)>{
  'no_home_body': (
    'Create a household on this device to start planning, cooking and shopping.',
    'Lege auf diesem Gerät einen Haushalt an, um zu planen, zu kochen und einzukaufen.',
  ),
  "synced": ("Saved on this device", "Auf diesem Gerät gespeichert"),
  "syncing": ("Reading local data…", "Lokale Daten werden gelesen …"),
  "offline": (
    "Local core unavailable. Saved data is kept.",
    "Lokaler Kern nicht erreichbar. Gespeicherte Daten bleiben erhalten.",
  ),
  "last_sync": ("Last read", "Zuletzt gelesen"),
  "online_content": (
    "Changes are saved in this app’s local data store.",
    "Änderungen werden im lokalen Datenspeicher dieser App gesichert.",
  ),
  "online_required": (
    "Reopen local data before continuing.",
    "Öffne die lokalen Daten erneut, bevor du fortfährst.",
  ),
  "server_version": ("Current saved version", "Aktueller gespeicherter Stand"),
  "invalid_response": (
    "The local response is incompatible. Install a matching app build.",
    "Die lokale Antwort ist nicht kompatibel. Installiere einen passenden App-Build.",
  ),
  "cooking_recovery_blocked": (
    "Protected cooking recovery data cannot be read. Check saved local sessions before clearing this device cache.",
    "Geschützte Wiederherstellungsdaten fürs Kochen sind nicht lesbar. Prüfe die lokal gespeicherten Abläufe, bevor du diesen Gerätecache entfernst.",
  ),
  "cookSchedulerWaiting": (
    "Keep this view open to check upcoming actions.",
    "Lass diese Ansicht offen, um bevorstehende Aktionen zu prüfen.",
  ),
  "cookOffline": (
    "Showing a retained snapshot. Pending actions need confirmation from the local core. Elapsed time alone never completes a step.",
    "Eine gespeicherte Momentaufnahme wird angezeigt. Ausstehende Aktionen benötigen die Bestätigung des lokalen Kerns. Abgelaufene Zeit allein schließt keinen Schritt ab.",
  ),
  "cooking_storage_failed": (
    "The cooking store is unavailable. Check device storage; saved deadlines have not been restarted.",
    "Der Kochspeicher ist nicht verfügbar. Prüfe den Gerätespeicher; gespeicherte Fristen wurden nicht neu gestartet.",
  ),
  "brewing_capacity": (
    "The brewing storage limit was reached. Archiving keeps history. Keep a verified backup before reviewing the saved data.",
    "Die Speichergrenze fürs Brauen ist erreicht. Archivieren behält den Verlauf. Bewahre vor der Prüfung gespeicherter Daten ein verifiziertes Backup auf.",
  ),
  "cook_discard_help": (
    "Discard these unconfirmed intents only? Saved changes remain in the local data store.",
    "Nur diese unbestätigten Aktionen verwerfen? Gespeicherte Änderungen bleiben im lokalen Datenspeicher.",
  ),
  "cook_timer_pending": (
    "Pending local confirmation",
    "Lokale Bestätigung ausstehend",
  ),
  "cook_cache_help": (
    "Five sessions can be cached for seven days. Removing a cached session keeps its canonical local record.",
    "Fünf Abläufe können sieben Tage zwischengespeichert werden. Entfernen aus dem Cache behält den eigentlichen lokalen Datensatz.",
  ),
  "capture_fetch_body": (
    "Allow this device to contact the website and validated redirects? The website receives your URL request and device network address.",
    "Darf dieses Gerät die Website und geprüfte Weiterleitungen aufrufen? Die Website erhält deine URL-Anfrage und die Netzwerkadresse des Geräts.",
  ),
  "capture_discard_body": (
    "An uncertain save may already exist. Check your recipes first. Discarding removes only this draft, not saved recipes or source images.",
    "Ein unbestätigter Speichervorgang kann bereits erfolgt sein. Prüfe zuerst deine Rezepte. Verwerfen entfernt nur diesen Entwurf, keine gespeicherten Rezepte oder Quellbilder.",
  ),
  "import_preview_expired": (
    "This preview expired or the local core restarted. Check recipes before reviewing the source again.",
    "Diese Vorschau ist abgelaufen oder der lokale Kern wurde neu gestartet. Prüfe Rezepte, bevor du die Quelle erneut kontrollierst.",
  ),
  "import_storage_failed": (
    "The local core could not confirm the import. Retry the unchanged request.",
    "Der lokale Kern konnte den Import nicht bestätigen. Wiederhole die unveränderte Anfrage.",
  ),
  "import_consent_required": (
    "Check source fields and confirm retrieval only if you agree. Check the device date if needed.",
    "Prüfe die Quellfelder und bestätige den Abruf nur mit deinem Einverständnis. Prüfe bei Bedarf das Gerätedatum.",
  ),
  "import_invalid_input": (
    "Check the source fields and device date before trying again.",
    "Prüfe Quellfelder und Gerätedatum vor einem erneuten Versuch.",
  ),
  "import_clock_invalid": (
    "Set a valid device date before retrieving this source.",
    "Stelle vor dem Abrufen dieser Quelle ein gültiges Gerätedatum ein.",
  ),
};
