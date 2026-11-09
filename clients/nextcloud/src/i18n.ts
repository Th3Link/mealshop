import { computed, ref, watch } from 'vue'
import { groups } from '../../../data/allergens.json'
const en = {
  timingEstimate: 'Preparation estimate',
  referenceServings: 'Reference servings',
  elapsedSeconds: 'Elapsed time (seconds)',
  advanceSeconds: 'Advance preparation (seconds)',
  timingWarning:
    'Times refer to the saved reference batch. Changing servings can change the work and waiting time; review them before applying.',
  attendanceOverridden: 'Calendar attendance has been overridden.',
  someRejected: 'Some recipes were excluded by the dietary or time constraints.',
  refreshAccess: 'Refresh access',
  originalInput: 'Original input',
  leaveHousehold: 'Leave household',
  overrideAttendance: 'Override calendar attendance with selected attendees',
  displayUnit: 'Recipe mass display',
  unit: 'Unit',
  conversion: 'Conversion evidence',
  conversionHelp:
    'Volume and pieces require exact ingredient/state-specific evidence. No automatic ml-to-g conversion is used.',
  manual: 'Declare your own evidence',
  density: 'Grams per ml or piece',
  sourceUrl: 'HTTPS source URL',
  spoonStandard: 'Named spoon standard',
  spoonVolume: 'Millilitres per spoon',
  exportProfiles: 'Export profiles',
  recipes: 'Recipes',
  ingredients: 'Ingredients',
  plan: 'Meal plan',
  household: 'Household',
  households: 'Your households',
  language: 'Language',
  loading: 'Loading…',
  retry: 'Try again',
  cancel: 'Cancel',
  save: 'Save changes',
  create: 'Create',
  back: 'Back',
  search: 'Search ingredients',
  noResults: 'No matching ingredients',
  clearSearch: 'Clear search',
  all: 'All ingredients',
  catalogueIntro: 'Explore ingredient identities, nutrient values and their original sources.',
  original: 'Original content (English)',
  unknown: 'Unknown',
  nutrients: 'Nutrient values',
  nutrient: 'Nutrient',
  amount: 'Amount',
  basis: 'Per 100 g edible portion',
  source: 'Source',
  record: 'Record',
  version: 'Version',
  licence: 'Licence',
  retrieved: 'Retrieved',
  uncertainty: 'Uncertainty',
  sourceNotice:
    'Values describe this exact food record. Unknown values are not zero and do not establish suitability for an allergy.',
  openSource: 'Open primary source',
  form: 'Food form',
  results: 'ingredients',
  name: 'Name',
  details: 'View details',
  householdsIntro:
    'Keep recipes, dietary profiles and plans together. Access is limited to accepted household members.',
  createHousehold: 'Create household',
  householdName: 'Household name',
  noHouseholds: 'Your kitchen starts here',
  noHouseholdsText:
    'Create a household or accept an invitation to start cooking and planning together.',
  invitations: 'Invitations',
  accept: 'Accept invitation',
  decline: 'Decline',
  invitationNotice:
    'Accepting gives you access to the household’s recipes, plans and dietary profiles, including allergy information.',
  owner: 'Owner',
  editor: 'Editor',
  reader: 'Reader',
  readOnly: 'You have read-only access to this household.',
  members: 'Access and invitations',
  invite: 'Invite a person',
  userId: 'Exact Nextcloud user ID',
  role: 'Role',
  revoke: 'Revoke access',
  inviteHelp:
    'The person must accept in Mealshop. User IDs are case-sensitive; no account directory is searched.',
  pending: 'Pending invitations',
  noInvitations: 'No pending invitations',
  transfer: 'Transfer ownership',
  profiles: 'Dietary profiles',
  profileIntro:
    'Keep allergies and hard exclusions separate from preferences. Unknown data always requires review.',
  newMember: 'Add dietary profile',
  remove: 'Remove',
  allergies: 'Allergy notes',
  intolerances: 'Intolerance notes',
  allergenGroups: 'Hard allergen exclusions',
  excluded: 'Excluded ingredients',
  disliked: 'Disliked ingredients',
  onePerLine: 'One entry per line',
  deleteProfile: 'Remove this dietary profile?',
  confirm: 'Confirm',
  recipeIntro: 'Your household’s recipes, with precise quantities and pinned source records.',
  newRecipe: 'Create recipe',
  noRecipes: 'No recipes yet',
  noRecipesText: 'Create the first recipe for your household.',
  title: 'Recipe title',
  author: 'Author',
  servings: 'Servings',
  preparation: 'Preparation',
  asRecorded: 'As recorded',
  cooking: 'Cooking effects not modelled',
  lines: 'Ingredients and quantities',
  addLine: 'Add ingredient',
  grams: 'Grams',
  selectIngredient: 'Choose ingredient',
  steps: 'Preparation steps',
  addStep: 'Add step',
  instruction: 'Instruction',
  activeSeconds: 'Active time (seconds)',
  passiveSeconds: 'Waiting time (seconds)',
  edit: 'Edit recipe',
  revision: 'Revision',
  previewServings: 'Preview servings',
  preview: 'Preview',
  perServing: 'Per serving',
  total: 'Total',
  knownSubtotal: 'Known subtotal',
  nutritionLimits:
    'Estimates retain the recipe’s source snapshots. No unmodelled cooking losses or yield corrections are applied.',
  emptyPlan: 'Plan your next meal',
  emptyPlanText: 'Add a saved recipe, choose attendees and review quantities before applying.',
  newMeal: 'Plan a meal',
  date: 'Date',
  meal: 'Meal',
  breakfast: 'Breakfast',
  lunch: 'Lunch',
  dinner: 'Dinner',
  snack: 'Snack',
  attendees: 'Attendees',
  activeBudget: 'Active time available (seconds)',
  elapsedBudget: 'Elapsed time available (seconds)',
  advanceBudget: 'Advance preparation available (seconds)',
  suggestions: 'Find suitable recipes',
  review: 'Review meal',
  apply: 'Apply reviewed plan',
  shopping: 'Shopping quantity changes',
  conflicts: 'Conflicts',
  reviewRequired: 'Review required',
  acknowledge: 'I reviewed the unknown dietary data and serving/time uncertainty.',
  noSuggestions: 'No feasible recipes found. Review the listed dietary and time constraints.',
  calendar: 'Calendar attendance snapshot',
  consent: 'I consent to using this read-only calendar snapshot.',
  calendarSource: 'Calendar source',
  eventId: 'Event ID',
  calendarHelp:
    'An imported snapshot proposes attendance; selected attendees override it. The external calendar is never changed.',
  error: 'The request could not be completed. Your input has been retained. Try again.',
  unauthorized: 'Your Nextcloud session has expired. Sign in again and reload this page.',
  forbidden:
    'You no longer have access to this household, or your role does not permit this action.',
  conflict:
    'The data changed in another session. Reload and review the current revision before saving.',
  invalid: 'Check the required fields, exact quantities and selected people.',
  storage: 'Storage is unavailable. Contact the instance operator before retrying a write.',
  quantityError:
    'Use a positive decimal amount with at most six decimal places, without grouping separators.',
  required: 'Complete the required fields.',
  unsaved: 'Discard your unsaved changes?',
  noProfiles: 'No dietary profiles yet',
  selectHousehold: 'Switch household',
  accessHelp:
    'Owners manage access. Editors change recipes, profiles and plans. Readers can view, export and review them.',
} satisfies Record<string, string>
const de: Record<keyof typeof en, string> = {
  timingEstimate: 'Geschätzte Zubereitungszeit',
  referenceServings: 'Referenzportionen',
  elapsedSeconds: 'Gesamtdauer (Sekunden)',
  advanceSeconds: 'Vorbereitung (Sekunden)',
  timingWarning:
    'Die Zeiten beziehen sich auf die gespeicherte Referenzmenge. Andere Portionen können Arbeits- und Wartezeiten verändern; prüfe sie vor dem Übernehmen.',
  attendanceOverridden: 'Die Kalender-Anwesenheit wurde überschrieben.',
  someRejected: 'Einige Rezepte wurden wegen Ernährungs- oder Zeitkonflikten ausgeschlossen.',
  refreshAccess: 'Zugriffe aktualisieren',
  originalInput: 'Originaleingabe',
  leaveHousehold: 'Haushalt verlassen',
  overrideAttendance: 'Kalender-Anwesenheit durch ausgewählte Personen ersetzen',
  displayUnit: 'Mengenanzeige im Rezept',
  unit: 'Einheit',
  conversion: 'Umrechnungsbeleg',
  conversionHelp:
    'Volumen und Stückzahlen benötigen einen Beleg für genau diese Zutat und Form. Es gibt keine automatische ml-zu-g-Umrechnung.',
  manual: 'Eigenen Beleg angeben',
  density: 'Gramm je ml oder Stück',
  sourceUrl: 'HTTPS-Quellenadresse',
  spoonStandard: 'Benannter Löffelstandard',
  spoonVolume: 'Milliliter je Löffel',
  exportProfiles: 'Profile exportieren',
  recipes: 'Rezepte',
  ingredients: 'Zutaten',
  plan: 'Speiseplan',
  household: 'Haushalt',
  households: 'Deine Haushalte',
  language: 'Sprache',
  loading: 'Wird geladen…',
  retry: 'Erneut versuchen',
  cancel: 'Abbrechen',
  save: 'Änderungen speichern',
  create: 'Erstellen',
  back: 'Zurück',
  search: 'Zutaten suchen',
  noResults: 'Keine passenden Zutaten',
  clearSearch: 'Suche leeren',
  all: 'Alle Zutaten',
  catalogueIntro: 'Zutaten, Nährwerte und ihre Originalquellen erkunden.',
  original: 'Originalinhalt (Englisch)',
  unknown: 'Unbekannt',
  nutrients: 'Nährwerte',
  nutrient: 'Nährstoff',
  amount: 'Menge',
  basis: 'Je 100 g essbarer Anteil',
  source: 'Quelle',
  record: 'Datensatz',
  version: 'Version',
  licence: 'Lizenz',
  retrieved: 'Abgerufen',
  uncertainty: 'Unsicherheit',
  sourceNotice:
    'Die Werte beschreiben genau diesen Lebensmitteldatensatz. Unbekannt bedeutet nicht null und bestätigt keine Allergieverträglichkeit.',
  openSource: 'Primärquelle öffnen',
  form: 'Lebensmittelform',
  results: 'Zutaten',
  name: 'Name',
  details: 'Details ansehen',
  householdsIntro:
    'Rezepte, Ernährungsprofile und Pläne gemeinsam verwalten. Zugriff erhalten nur bestätigte Haushaltsmitglieder.',
  createHousehold: 'Haushalt erstellen',
  householdName: 'Haushaltsname',
  noHouseholds: 'Hier beginnt deine Küche',
  noHouseholdsText:
    'Erstelle einen Haushalt oder nimm eine Einladung an, um gemeinsam zu kochen und zu planen.',
  invitations: 'Einladungen',
  accept: 'Einladung annehmen',
  decline: 'Ablehnen',
  invitationNotice:
    'Mit der Annahme erhältst du Zugriff auf Rezepte, Pläne und Ernährungsprofile einschließlich Allergieangaben.',
  owner: 'Besitzer',
  editor: 'Bearbeiter',
  reader: 'Leser',
  readOnly: 'Du hast Lesezugriff auf diesen Haushalt.',
  members: 'Zugriff und Einladungen',
  invite: 'Person einladen',
  userId: 'Exakte Nextcloud-Benutzerkennung',
  role: 'Rolle',
  revoke: 'Zugriff entziehen',
  inviteHelp:
    'Die Person muss in Mealshop zustimmen. Groß- und Kleinschreibung beachten; es wird kein Benutzerverzeichnis durchsucht.',
  pending: 'Ausstehende Einladungen',
  noInvitations: 'Keine ausstehenden Einladungen',
  transfer: 'Besitz übertragen',
  profiles: 'Ernährungsprofile',
  profileIntro:
    'Allergien und harte Ausschlüsse getrennt von Vorlieben erfassen. Unbekannte Angaben müssen immer geprüft werden.',
  newMember: 'Ernährungsprofil hinzufügen',
  remove: 'Entfernen',
  allergies: 'Allergiehinweise',
  intolerances: 'Unverträglichkeitshinweise',
  allergenGroups: 'Harte Allergenausschlüsse',
  excluded: 'Ausgeschlossene Zutaten',
  disliked: 'Ungern gegessene Zutaten',
  onePerLine: 'Ein Eintrag je Zeile',
  deleteProfile: 'Dieses Ernährungsprofil entfernen?',
  confirm: 'Bestätigen',
  recipeIntro:
    'Die Rezepte deines Haushalts mit exakten Mengen und festgehaltenen Quelldatensätzen.',
  newRecipe: 'Rezept erstellen',
  noRecipes: 'Noch keine Rezepte',
  noRecipesText: 'Erstelle das erste Rezept für deinen Haushalt.',
  title: 'Rezepttitel',
  author: 'Autor',
  servings: 'Portionen',
  preparation: 'Zubereitung',
  asRecorded: 'Wie erfasst',
  cooking: 'Kocheffekte nicht modelliert',
  lines: 'Zutaten und Mengen',
  addLine: 'Zutat hinzufügen',
  grams: 'Gramm',
  selectIngredient: 'Zutat auswählen',
  steps: 'Zubereitungsschritte',
  addStep: 'Schritt hinzufügen',
  instruction: 'Anweisung',
  activeSeconds: 'Arbeitszeit (Sekunden)',
  passiveSeconds: 'Wartezeit (Sekunden)',
  edit: 'Rezept bearbeiten',
  revision: 'Version',
  previewServings: 'Portionen vorschauen',
  preview: 'Vorschau',
  perServing: 'Je Portion',
  total: 'Gesamt',
  knownSubtotal: 'Bekannte Zwischensumme',
  nutritionLimits:
    'Schätzungen behalten die Quellenstände des Rezepts bei. Nicht modellierte Kochverluste oder Ausbeutekorrekturen werden nicht angewendet.',
  emptyPlan: 'Plane deine nächste Mahlzeit',
  emptyPlanText:
    'Wähle ein gespeichertes Rezept und die Mitessenden; prüfe vor dem Übernehmen die Mengen.',
  newMeal: 'Mahlzeit planen',
  date: 'Datum',
  meal: 'Mahlzeit',
  breakfast: 'Frühstück',
  lunch: 'Mittagessen',
  dinner: 'Abendessen',
  snack: 'Zwischenmahlzeit',
  attendees: 'Mitessende',
  activeBudget: 'Verfügbare Arbeitszeit (Sekunden)',
  elapsedBudget: 'Verfügbare Gesamtdauer (Sekunden)',
  advanceBudget: 'Verfügbare Vorbereitungszeit (Sekunden)',
  suggestions: 'Passende Rezepte finden',
  review: 'Mahlzeit prüfen',
  apply: 'Geprüften Plan übernehmen',
  shopping: 'Änderungen der Einkaufsmengen',
  conflicts: 'Konflikte',
  reviewRequired: 'Prüfung erforderlich',
  acknowledge:
    'Ich habe unbekannte Ernährungsdaten und die Unsicherheit bei Portionen und Zeiten geprüft.',
  noSuggestions:
    'Keine passenden Rezepte gefunden. Prüfe die aufgeführten Ernährungs- und Zeitkonflikte.',
  calendar: 'Kalender-Anwesenheit als Momentaufnahme',
  consent: 'Ich stimme der Verwendung dieser schreibgeschützten Kalender-Momentaufnahme zu.',
  calendarSource: 'Kalenderquelle',
  eventId: 'Ereigniskennung',
  calendarHelp:
    'Ein importierter Stand schlägt Mitessende vor; deine Auswahl hat Vorrang. Der externe Kalender bleibt unverändert.',
  error:
    'Die Anfrage konnte nicht abgeschlossen werden. Deine Eingaben bleiben erhalten. Versuche es erneut.',
  unauthorized:
    'Deine Nextcloud-Sitzung ist abgelaufen. Melde dich erneut an und lade die Seite neu.',
  forbidden:
    'Du hast keinen Zugriff mehr auf diesen Haushalt oder deine Rolle erlaubt diese Aktion nicht.',
  conflict:
    'Die Daten wurden in einer anderen Sitzung geändert. Lade den aktuellen Stand und prüfe ihn vor dem Speichern.',
  invalid: 'Prüfe Pflichtfelder, exakte Mengen und ausgewählte Personen.',
  storage:
    'Der Speicher ist nicht verfügbar. Kontaktiere vor weiteren Schreibversuchen den Betreiber.',
  quantityError:
    'Verwende eine positive Dezimalmenge mit höchstens sechs Nachkommastellen ohne Tausendertrennzeichen.',
  required: 'Fülle die erforderlichen Felder aus.',
  unsaved: 'Ungespeicherte Änderungen verwerfen?',
  noProfiles: 'Noch keine Ernährungsprofile',
  selectHousehold: 'Haushalt wechseln',
  accessHelp:
    'Besitzer verwalten Zugriffe. Bearbeiter ändern Rezepte, Profile und Pläne. Leser können diese ansehen, exportieren und prüfen.',
}
export const locale = ref<'en' | 'de'>(
  typeof document !== 'undefined' && document.documentElement.lang.toLowerCase().startsWith('de')
    ? 'de'
    : 'en',
)
try {
  const saved = localStorage.getItem('mealshop.locale')
  if (saved === 'de' || saved === 'en') locale.value = saved
} catch {
  /* Storage can be unavailable in a private browser session. */
}
watch(locale, (value) => {
  try {
    localStorage.setItem('mealshop.locale', value)
  } catch {
    /* In-memory language selection remains usable. */
  }
})
export function t(key: keyof typeof en): string {
  return (locale.value === 'de' ? de : en)[key]
}
export const decimalSeparator = computed(() => (locale.value === 'de' ? ',' : '.'))
export function exact(value: string | null): string {
  return value === null ? t('unknown') : value.replace('.', decimalSeparator.value)
}
export function parseGrams(input: string, language = locale.value): string | null {
  if (input.length > 32) return null
  const pattern =
    language === 'de' ? /^(?:0|[1-9]\d*)(?:,\d{1,6})?$/ : /^(?:0|[1-9]\d*)(?:\.\d{1,6})?$/
  if (!pattern.test(input)) return null
  const canonical = input.replace(',', '.')
  const [whole = '', fraction = ''] = canonical.split('.')
  const atoms = BigInt(whole) * 1000000n + BigInt(fraction.padEnd(6, '0'))
  return atoms > 0n && atoms <= 1000000000000000n ? canonical : null
}
const explanations: Record<string, { en: string; de: string }> = {
  dietary_exclusion: {
    en: 'An attendee has a hard dietary exclusion for this recipe.',
    de: 'Für eine mitessende Person enthält dieses Rezept einen harten Ernährungsausschluss.',
  },
  preparation_time_unknown: {
    en: 'This recipe has no preparation steps with known durations.',
    de: 'Für dieses Rezept fehlen Zubereitungsschritte mit bekannten Zeiten.',
  },
  active_time_exceeded: {
    en: 'Active preparation exceeds the available time.',
    de: 'Die Arbeitszeit überschreitet die verfügbare Zeit.',
  },
  elapsed_time_exceeded: {
    en: 'The recipe needs more elapsed or advance preparation time.',
    de: 'Das Rezept benötigt mehr Gesamtdauer oder Vorbereitungszeit.',
  },
  allergen_coverage_incomplete: {
    en: 'Allergen coverage is incomplete. Check every ingredient and product label.',
    de: 'Die Allergendaten sind unvollständig. Prüfe jede Zutat und die Produktkennzeichnung.',
  },
  cross_contamination_unmodelled: {
    en: 'Cross-contamination is not modelled; this review cannot establish allergy safety.',
    de: 'Kreuzkontamination wird nicht berücksichtigt; diese Prüfung bestätigt keine Allergiesicherheit.',
  },
  owner_transfer_required: {
    en: 'Transfer ownership to another accepted member before leaving.',
    de: 'Übertrage den Besitz vor dem Verlassen an ein bestätigtes Mitglied.',
  },
  already_member: {
    en: 'This person already belongs to the household.',
    de: 'Diese Person gehört bereits zum Haushalt.',
  },
  membership_capacity: {
    en: 'The household or membership limit has been reached.',
    de: 'Die Grenze für Haushalte oder Mitgliedschaften wurde erreicht.',
  },
  recipe_revision_not_found: {
    en: 'The selected recipe revision is unavailable. Select an existing revision.',
    de: 'Die gewählte Rezeptversion ist nicht verfügbar. Wähle eine vorhandene Version.',
  },
}
export function explanation(code: string): string {
  return explanations[code]?.[locale.value] ?? t('reviewRequired')
}

export function allergenName(id: string): string {
  const group = groups.find((g) => g.id === id)
  return (locale.value === 'de' ? group?.translations.de : undefined) ?? group?.name ?? id
}
