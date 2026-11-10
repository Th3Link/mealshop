import { cookingEnglish, cookingGerman, cookingExplanations } from './cooking-messages'
import { shoppingEnglish, shoppingGerman } from './shopping-messages'
import { computed, ref, watch } from 'vue'
import { groups } from '../../../data/allergens.json'
const en = {
  finishAndWait: 'Done · start waiting time',
  moveToPart: 'Move to recipe part',
  production: 'Prepare once',
  productionServings: 'Production servings',
  timeLimits: 'Limit available time',
  prepareSlot: 'Prepare',
  allocations: 'Eat on',
  unallocated: 'Eating days not assigned',
  addEatingDay: 'Add eating day',
  allocatedPortions: 'Allocated portions',
  productionHelp:
    'Ingredients are counted once for the production serving count. Removing this entry removes all its eating days.',
  preparationTime: 'Preparation time',
  prepareOnly: 'Prepare without assigned diners',
  startsAfterParts: 'Start after prepared parts',
  waitingRange: 'Waiting time range',
  maximumWait: 'Longest waiting time',
  ingredientTemperature: 'Temperature (°C)',
  addTemperature: 'Add temperature',
  temperatureError:
    'Enter a temperature between −273.15 and 1000 °C with up to six decimal places.',
  originalLanguage: 'Original language',
  ...shoppingEnglish,
  ...cookingEnglish,
  searchRecipes: 'Search recipes',
  resetFilters: 'Reset filters',
  changesSaved: 'Changes saved.',
  saving: 'Saving…',
  discardChanges: 'Discard changes',
  profileNotes: 'Notes and excluded tags',
  editMeal: 'Meal details',
  planPrerequisites: 'Add a recipe and a dietary profile before planning meals.',

  noSearchResults: 'No matching results. Try another name.',
  createNamed: 'Create ingredient',
  identitySection: 'Ingredient details',
  identityHelp: 'Start with a name and form. Add values and details when you know them.',
  classificationSection: 'Classification and composition',
  provenanceSection: 'Source and recording',
  moreNutrients: 'Vitamins and minerals',
  foodConstituents: 'Additional food constituents',
  constituentsSummaryHelp:
    'Declared amounts are added across ingredients. Missing values remain unknown.',
  constituentsHelp:
    'Add only constituents you have information about. Amounts are per 100 g; missing values remain unknown.',
  addConstituent: 'Add constituent',
  constituentName: 'Constituent name',
  otherConstituent: 'Other constituent',
  chooseConstituent: 'Choose constituent',
  intoleranceTriggers: 'Intolerance triggers',
  sorbitol: 'Sorbitol',
  recipeParts: 'Recipe parts',
  partsHelp: 'Organize preparations into named parts, with their own ingredients and steps.',
  addPart: 'Add recipe part',
  partName: 'Part name',
  mainRecipe: 'Main recipe',
  belongsTo: 'Recipe part',
  usesParts: 'Uses prepared recipe parts',
  stepIngredients: 'Ingredients used in this step',
  stepParts: 'Prepared parts used in this step',
  removePartConfirm: 'Remove this part? Its ingredients and steps move to the main recipe.',
  removePart: 'Remove recipe part',
  recipeDetails: 'Recipe details',
  optionalDetails: 'Optional details',
  saveIngredient: 'Save ingredient',
  ingredientPending: 'Saving ingredient…',
  sourceHelp:
    'Your own values are household declarations. Keep a label or source note so they can be checked later.',
  allergenHelp: 'Declare known presence. An unselected box does not establish absence.',
  untitledLine: 'Ingredient row',
  nameRequired: 'Enter a name to continue.',

  ingredientGroup: 'Ingredient group',
  ingredientGroups: 'Groups',
  groupIngredient: 'This is a group',
  noGroup: 'No group',
  nutritionMode: 'Nutrient values per 100 g',
  referenceValues: 'Keep original reference / unknown',
  ownValues: 'Enter own values',
  inheritValues: 'Inherit group values',
  nutritionNote: 'Nutrient source / evidence note',
  nutritionSource: 'Nutrient source URL (optional, HTTPS)',
  nutrientQuantityError:
    'Use zero or a positive decimal value with at most six decimal places, without grouping separators.',
  unknownValue: 'Empty means unknown; 0 is an explicit zero. All values are per 100 g.',
  dietClass: 'Dietary classification',
  vegan: 'Vegan',
  vegetarian: 'Vegetarian',
  meat_fish: 'Contains meat/fish',
  originClass: 'Ingredient origin',
  plant: 'Plant-derived',
  animal: 'Animal-derived',
  mixed: 'Mixed origin',
  subtypes: 'Subtypes',
  inheritedFrom: 'Nutrient values inherited from',
  reloadSaved: 'Reload saved version',
  householdEvidence: 'Household declaration',
  excludedTags: 'Hard excluded ingredient tags',
  excludedTagsHelp:
    'Matching tags on ingredients or their components block this person’s meals. One exact tag per line; case is ignored.',
  optional: 'Optional',
  addIngredient: 'Create ingredient',
  ingredientName: 'Ingredient name',
  ingredientForm: 'Form or product description',
  tags: 'Tags',
  tagsHelp: 'One tag per line; descriptive tags do not establish allergy safety.',
  components: 'Contains ingredients',
  declaredAllergens: 'Declared allergens',
  sourceDeclaration: 'Household declaration · values may be incomplete',
  sourceOptional: 'Source URL (optional, HTTPS)',
  recordedOn: 'Recorded on',
  favorite: 'Household favourite',
  favoritesOnly: 'Favourites only',
  showArchived: 'Show archived ingredients',
  archiveIngredient: 'Archive ingredient',
  restoreIngredient: 'Restore ingredient',
  archiveHelp:
    'Archive this ingredient for new selections? Saved recipes and their sources remain available.',
  editIngredient: 'Edit ingredient',
  ingredientMetadata: 'Tags and composition',
  recipesUsingIngredient: 'Recipes with this ingredient',
  noRecipesForIngredient: 'No saved recipes use this ingredient.',
  image: 'Recipe image',
  imageHelp: 'JPEG, PNG or WebP; up to 5 MiB. Uploaded privately to this household.',
  removeImage: 'Remove image',
  imageTooLarge: 'Choose a supported image up to 5 MiB.',
  previousWeek: 'Previous week',
  nextWeek: 'Next week',
  today: 'Today',
  emptySlot: 'Plan a meal',
  eating: 'Eating',
  skipping: 'Skipping',
  attendanceHelp:
    'Select who is eating this meal. Unselected people skip it. Servings remain adjustable.',
  invitePerson: 'Person to invite',
  searchPeople: 'Search Nextcloud people',
  searchPeopleHelp: 'Search follows the same account visibility rules as file sharing.',
  archived: 'Archived',
  chooseIngredient: 'Search or select an ingredient',

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
  activeBudget: 'Active time available',
  elapsedBudget: 'Elapsed time available',
  advanceBudget: 'Advance preparation available',
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
  finishAndWait: 'Fertig · Ruhezeit starten',
  moveToPart: 'In Rezeptteil verschieben',
  production: 'Einmal zubereiten',
  productionServings: 'Zubereitete Portionen',
  timeLimits: 'Verfügbare Zeit begrenzen',
  prepareSlot: 'Vorbereiten',
  allocations: 'Essen am',
  unallocated: 'Esstage noch nicht zugeordnet',
  addEatingDay: 'Essenstag ergänzen',
  allocatedPortions: 'Verplante Portionen',
  productionHelp:
    'Zutaten werden einmal für die Zubereitungsmenge gerechnet. Entfernen löscht diesen Eintrag mit allen Essenstagen.',
  preparationTime: 'Vorbereitungszeit',
  prepareOnly: 'Ohne zugeordnete Personen vorbereiten',
  startsAfterParts: 'Nach vorbereiteten Teilen beginnen',
  waitingRange: 'Zeitspanne der Ruhezeit',
  maximumWait: 'Längste Ruhezeit',

  ...cookingGerman,
  ingredientTemperature: 'Temperatur (°C)',
  addTemperature: 'Temperatur ergänzen',
  temperatureError:
    'Gib eine Temperatur zwischen −273,15 und 1000 °C mit bis zu sechs Nachkommastellen ein.',
  originalLanguage: 'Originalsprache',
  ...shoppingGerman,
  searchRecipes: 'Rezepte suchen',
  resetFilters: 'Filter zurücksetzen',
  changesSaved: 'Änderungen gespeichert.',
  saving: 'Wird gespeichert…',
  discardChanges: 'Änderungen verwerfen',
  profileNotes: 'Notizen und ausgeschlossene Tags',
  editMeal: 'Mahlzeit bearbeiten',
  planPrerequisites: 'Lege zuerst ein Rezept und ein Personenprofil an, um Mahlzeiten zu planen.',
  noSearchResults: 'Keine passenden Ergebnisse. Versuche einen anderen Namen.',
  createNamed: 'Zutat anlegen',
  identitySection: 'Angaben zur Zutat',
  identityHelp:
    'Beginne mit Name und Form. Werte und weitere Angaben kannst du ergänzen, sobald du sie kennst.',
  classificationSection: 'Einordnung und Zusammensetzung',
  provenanceSection: 'Quelle und Erfassung',
  moreNutrients: 'Vitamine und Mineralstoffe',
  foodConstituents: 'Weitere Inhaltsstoffe',
  constituentsSummaryHelp:
    'Deklarierte Mengen werden über die Zutaten addiert. Fehlende Angaben bleiben unbekannt.',
  constituentsHelp:
    'Ergänze nur Stoffe, zu denen dir Angaben vorliegen. Mengen gelten je 100 g; fehlende Werte bleiben unbekannt.',
  addConstituent: 'Inhaltsstoff hinzufügen',
  constituentName: 'Name des Inhaltsstoffs',
  otherConstituent: 'Anderer Inhaltsstoff',
  chooseConstituent: 'Inhaltsstoff auswählen',
  intoleranceTriggers: 'Unverträglichkeitsauslöser',
  sorbitol: 'Sorbit',
  recipeParts: 'Teilrezepte',
  partsHelp:
    'Gliedere Vorbereitungen in benannte Teile mit eigenen Zutaten und Zubereitungsschritten.',
  addPart: 'Teilrezept hinzufügen',
  partName: 'Name des Teilrezepts',
  mainRecipe: 'Hauptrezept',
  belongsTo: 'Zuordnung zum Teilrezept',
  usesParts: 'Verwendet vorbereitete Teilrezepte',
  stepIngredients: 'Zutaten in diesem Schritt',
  stepParts: 'Vorbereitete Teilrezepte in diesem Schritt',
  removePartConfirm:
    'Teilrezept entfernen? Seine Zutaten und Schritte werden ins Hauptrezept verschoben.',
  removePart: 'Teilrezept entfernen',
  recipeDetails: 'Angaben zum Rezept',
  optionalDetails: 'Weitere Angaben',
  saveIngredient: 'Zutat speichern',
  ingredientPending: 'Zutat wird gespeichert…',
  sourceHelp:
    'Eigene Werte sind Haushaltsangaben. Halte Etikett oder Quelle fest, damit sie später nachvollziehbar bleiben.',
  allergenHelp:
    'Gib bekannte Vorkommen an. Ein nicht gesetztes Häkchen bestätigt keine Abwesenheit.',
  untitledLine: 'Zutatenzeile',
  nameRequired: 'Gib einen Namen ein, um fortzufahren.',

  ingredientGroup: 'Zutatengruppe',
  ingredientGroups: 'Gruppen',
  groupIngredient: 'Dies ist eine Gruppe',
  noGroup: 'Keine Gruppe',
  nutritionMode: 'Nährwerte je 100 g',
  referenceValues: 'Originalreferenz / unbekannt beibehalten',
  ownValues: 'Eigene Werte eingeben',
  inheritValues: 'Gruppenwerte erben',
  nutritionNote: 'Quelle / Beleg der Nährwerte',
  nutritionSource: 'Nährwertquelle als URL (optional, HTTPS)',
  nutrientQuantityError:
    'Verwende null oder einen positiven Dezimalwert mit höchstens sechs Nachkommastellen ohne Tausendertrennzeichen.',
  unknownValue:
    'Leer bedeutet unbekannt; 0 ist ein ausdrücklicher Nullwert. Alle Werte gelten je 100 g.',
  dietClass: 'Ernährungsweise',
  vegan: 'Vegan',
  vegetarian: 'Vegetarisch',
  meat_fish: 'Mit Fleisch/Fisch',
  originClass: 'Herkunft der Zutat',
  plant: 'Pflanzlich',
  animal: 'Tierisch',
  mixed: 'Gemischt',
  subtypes: 'Untertypen',
  inheritedFrom: 'Nährwerte geerbt von',
  reloadSaved: 'Gespeicherte Version neu laden',
  householdEvidence: 'Haushaltsangabe',
  excludedTags: 'Hart ausgeschlossene Zutaten-Tags',
  excludedTagsHelp:
    'Passende Tags an Zutaten oder ihren Bestandteilen blockieren Mahlzeiten für diese Person. Ein exakter Tag pro Zeile; Großschreibung wird ignoriert.',
  optional: 'Optional',
  addIngredient: 'Zutat anlegen',
  ingredientName: 'Name der Zutat',
  ingredientForm: 'Form oder Produktbeschreibung',
  tags: 'Tags',
  tagsHelp: 'Ein Tag pro Zeile; beschreibende Tags bestätigen keine Allergiesicherheit.',
  components: 'Enthaltene Zutaten',
  declaredAllergens: 'Deklarierte Allergene',
  sourceDeclaration: 'Haushaltsangabe · Werte können unvollständig sein',
  sourceOptional: 'Quellen-URL (optional, HTTPS)',
  recordedOn: 'Erfasst am',
  favorite: 'Haushaltsfavorit',
  favoritesOnly: 'Nur Favoriten',
  showArchived: 'Archivierte Zutaten anzeigen',
  archiveIngredient: 'Zutat archivieren',
  restoreIngredient: 'Zutat wiederherstellen',
  archiveHelp:
    'Diese Zutat für neue Auswahlen archivieren? Gespeicherte Rezepte und Quellen bleiben erhalten.',
  editIngredient: 'Zutat bearbeiten',
  ingredientMetadata: 'Tags und Zusammensetzung',
  recipesUsingIngredient: 'Rezepte mit dieser Zutat',
  noRecipesForIngredient: 'Kein gespeichertes Rezept verwendet diese Zutat.',
  image: 'Rezeptbild',
  imageHelp: 'JPEG, PNG oder WebP; bis 5 MiB. Wird privat in diesem Haushalt gespeichert.',
  removeImage: 'Bild entfernen',
  imageTooLarge: 'Wähle ein unterstütztes Bild bis 5 MiB.',
  previousWeek: 'Vorherige Woche',
  nextWeek: 'Nächste Woche',
  today: 'Heute',
  emptySlot: 'Mahlzeit planen',
  eating: 'Isst mit',
  skipping: 'Setzt aus',
  attendanceHelp:
    'Wähle, wer bei dieser Mahlzeit mitisst. Nicht ausgewählte Personen setzen aus. Portionen bleiben anpassbar.',
  invitePerson: 'Einzuladende Person',
  searchPeople: 'Nextcloud-Personen suchen',
  searchPeopleHelp: 'Die Suche verwendet dieselben Sichtbarkeitsregeln wie Dateifreigaben.',
  archived: 'Archiviert',
  chooseIngredient: 'Zutat suchen oder auswählen',

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
  activeBudget: 'Verfügbare Arbeitszeit',
  elapsedBudget: 'Verfügbare Gesamtdauer',
  advanceBudget: 'Verfügbare Vorbereitungszeit',
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
export function normalizeLocale(value: string): 'en' | 'de' {
  return value.toLowerCase().split(/[-_]/)[0] === 'de' ? 'de' : 'en'
}
export const locale = ref<'en' | 'de'>(
  normalizeLocale(typeof document !== 'undefined' ? document.documentElement.lang : 'en'),
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
  meal_allocations_exceed_production: {
    en: 'Allocate no more portions than you prepare. Increase production or reduce eating-day portions.',
    de: 'Verplane höchstens so viele Portionen, wie du zubereitest. Erhöhe die Zubereitungsmenge oder reduziere die Essenstage.',
  },
  invalid_meal_allocations: {
    en: 'Choose distinct eating days/meals and positive whole serving counts.',
    de: 'Wähle unterschiedliche Tage/Mahlzeiten und positive ganze Portionszahlen.',
  },
  meal_slot_conflict: {
    en: 'Another preparation already uses an eating slot. Choose another day or meal.',
    de: 'Ein anderer Eintrag belegt bereits einen Essenstermin. Wähle einen anderen Tag oder eine andere Mahlzeit.',
  },
  invalid_preparation_time: {
    en: 'Choose a valid preparation date and time, no later than the eating days.',
    de: 'Wähle einen gültigen Vorbereitungstag mit Uhrzeit, spätestens am ersten Essenstag.',
  },
  cooking_plan_source_changed: {
    en: 'The meal-plan recipe or production count changed. Reopen cooking from the saved plan.',
    de: 'Rezept oder Zubereitungsmenge im Speiseplan haben sich geändert. Öffne die Zubereitung erneut aus dem gespeicherten Plan.',
  },
  attendees_not_assigned: {
    en: 'Preparation has no assigned diners. Review dietary constraints before assigning eating days.',
    de: 'Der Vorbereitung sind keine Personen zugeordnet. Prüfe die Ernährungseinschränkungen vor dem Zuordnen von Essenstagen.',
  },

  ...cookingExplanations,
  shopping_barcode_invalid: {
    en: 'Enter a valid EAN/UPC/GTIN including check digit.',
    de: 'Gib einen gültigen EAN/UPC/GTIN mit Prüfziffer ein.',
  },
  shopping_match_required: {
    en: 'Confirm the ingredient match before saving.',
    de: 'Bestätige die Zutatenzuordnung vor dem Speichern.',
  },
  shopping_candidate_expired: {
    en: 'The lookup result expired. Look up the barcode again.',
    de: 'Das Suchergebnis ist abgelaufen. Schlage den Barcode erneut nach.',
  },
  shopping_product_invalid: {
    en: 'Review the product name, package quantity and package count.',
    de: 'Prüfe Produktname, Packungsmenge und Packungsanzahl.',
  },
  shopping_product_not_found: {
    en: 'Save this product before adding packages.',
    de: 'Speichere dieses Produkt vor dem Hinzufügen der Packungen.',
  },
  shopping_lookup_consent_required: {
    en: 'Confirm consent for this external barcode request.',
    de: 'Bestätige die Zustimmung zu dieser externen Barcode-Anfrage.',
  },
  shopping_lookup_throttled: {
    en: 'Wait a few seconds before another barcode lookup.',
    de: 'Warte einige Sekunden vor einer weiteren Barcode-Suche.',
  },
  shopping_lookup_unavailable: {
    en: 'The provider is unavailable. Enter the product manually.',
    de: 'Der Anbieter ist nicht erreichbar. Erfasse das Produkt manuell.',
  },
  shopping_lookup_invalid: {
    en: 'The provider returned unusable data. Enter the product manually.',
    de: 'Der Anbieter hat unbrauchbare Daten geliefert. Erfasse das Produkt manuell.',
  },

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
  shopping_shop_invalid: {
    en: 'Choose an existing shop and distinct first/second choices. Finish an active session before removing its shop.',
    de: 'Wähle ein vorhandenes Geschäft und unterschiedliche erste/zweite Wahl. Beende einen laufenden Einkauf vor dem Entfernen des Geschäfts.',
  },
  shopping_session_invalid: {
    en: 'Enable local learning, select a shop and finish the active session before starting another.',
    de: 'Aktiviere lokales Lernen, wähle ein Geschäft und beende den laufenden Einkauf vor dem nächsten.',
  },
  shopping_quantity_invalid: {
    en: 'Use a positive exact quantity up to 1,000,000,000 base g, ml or pieces per contribution.',
    de: 'Verwende eine positive exakte Menge bis 1.000.000.000 g, ml oder Stück je Beitrag.',
  },
  shopping_request_invalid: {
    en: 'Review the item name, quantity and selected source.',
    de: 'Prüfe den Artikelnamen, die Menge und die ausgewählte Quelle.',
  },
  shopping_source_not_found: {
    en: 'This list source is no longer available. Reload and review the list.',
    de: 'Diese Listenquelle ist nicht mehr verfügbar. Lade die Liste neu und prüfe sie.',
  },
  shopping_plan_invalid: {
    en: 'The selected meal slots are no longer current. Reload and review the selection.',
    de: 'Die ausgewählten Mahlzeiten sind nicht mehr aktuell. Lade sie neu und prüfe die Auswahl.',
  },
  shopping_capacity: {
    en: 'The shopping storage limit has been reached. Contact the operator before adding more entries.',
    de: 'Die Speichergrenze der Einkaufsliste ist erreicht. Kontaktiere den Betreiber vor weiteren Einträgen.',
  },
  shopping_label_invalid: {
    en: 'Enter an item name up to 200 bytes.',
    de: 'Gib einen Artikelnamen mit höchstens 200 Bytes ein.',
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
export function knownExplanation(code: string): string | undefined {
  return explanations[code]?.[locale.value]
}
export function explanation(code: string): string {
  return knownExplanation(code) ?? t('reviewRequired')
}

export function allergenName(id: string): string {
  const group = groups.find((g) => g.id === id)
  return (locale.value === 'de' ? group?.translations.de : undefined) ?? group?.name ?? id
}

export function compactExact(value: string | null): string {
  if (value === null) return t('unknown')
  return exact(value.includes('.') ? value.replace(/0+$/, '').replace(/\.$/, '') : value)
}

const nutrientGerman: Record<string, string> = {
  energy_kcal: 'Energie',
  energy_kj: 'Energie',
  protein: 'Eiweiß',
  fat: 'Fett',
  total_sugars: 'Zucker',
  saturated_fat: 'Gesättigte Fettsäuren',
  salt: 'Salz',
  carbohydrate: 'Kohlenhydrate (Differenzmethode, einschließlich Ballaststoffe)',
  available_carbohydrate: 'Verfügbare Kohlenhydrate (mit Polyolen, ohne Ballaststoffe)',
  fibre: 'Ballaststoffe',
  calcium: 'Calcium',
  iron: 'Eisen',
  magnesium: 'Magnesium',
  potassium: 'Kalium',
  sodium: 'Natrium',
  vitamin_c: 'Vitamin C',
  folate: 'Folat gesamt',
  food_folate: 'Nahrungsfolat (kein Gesamtfolat oder DFE)',
  beta_glucan: 'Beta-Glucan',
  soluble_fibre: 'Lösliche Ballaststoffe',
}
export function nutrientLabel(id: string, original: string): string {
  return locale.value === 'de' ? (nutrientGerman[id] ?? original) : original
}

export function parseNonnegative(input: string, language = locale.value): string | null {
  const zero = language === 'de' ? /^0(?:,0{1,6})?$/ : /^0(?:\.0{1,6})?$/
  return parseGrams(input, language) ?? (zero.test(input) ? input.replace(',', '.') : null)
}

export function servingCount(count: number): string {
  const unit =
    locale.value === 'de'
      ? count === 1
        ? 'Portion'
        : 'Portionen'
      : count === 1
        ? 'serving'
        : 'servings'
  return `${new Intl.NumberFormat(locale.value).format(count)} ${unit}`
}
export function mealName(value: string): string {
  return value === 'breakfast' || value === 'lunch' || value === 'snack' || value === 'dinner'
    ? t(value)
    : value
}
export function calendarDate(value: string): string {
  return new Intl.DateTimeFormat(locale.value, { dateStyle: 'medium' }).format(
    new Date(`${value}T12:00:00`),
  )
}

export function timestamp(seconds: number): string {
  return new Intl.DateTimeFormat(locale.value, { dateStyle: 'medium', timeStyle: 'short' }).format(
    new Date(seconds * 1000),
  )
}
export function shoppingUnit(unit: string, amount: string): string {
  return unit === 'piece'
    ? t(/^0*1(?:\.0+)?$/.test(amount) ? 'shoppingPiece' : 'shoppingPieces')
    : unit
}
