export const peopleEnglish = {
  personRules: 'Dietary baseline and availability',
  unrestricted: 'No dietary preset',
  ruleExceptions: 'Explicit exceptions',
  rulePriority:
    'Exceptions only relax the dietary baseline. Allergies, intolerances and explicit exclusions always take priority. Unknown declarations still require review.',
  householdProduced: 'Household-produced',
  householdOnly: 'Only household-produced',
  productionClass: 'How the ingredient is obtained',
  purchasedDeclaration: 'Purchased',
  maxPersonActive: 'Maximum active preparation time (seconds, optional)',
  availabilityWindows: 'Meal availability',
  availabilityHelp:
    'No windows means every meal is possible. Added windows restrict attendance to these weekdays and meals.',
  addAvailability: 'Add availability window',
  ruleWeekdays: 'Weekdays (Monday = 1)',
  ruleDays: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'].join(' · '),
  addRuleException: 'Add ingredient exception',
}
export const peopleGerman: Record<keyof typeof peopleEnglish, string> = {
  personRules: 'Ernährungsgrundlage und Verfügbarkeit',
  unrestricted: 'Keine Ernährungsform',
  ruleExceptions: 'Ausdrückliche Ausnahmen',
  rulePriority:
    'Ausnahmen ändern nur die Ernährungsgrundlage. Allergien, Unverträglichkeiten und ausdrückliche Ausschlüsse haben immer Vorrang. Unbekannte Angaben müssen weiterhin geprüft werden.',
  householdProduced: 'Selbst hergestellt',
  householdOnly: 'Nur selbst hergestellt',
  productionClass: 'Herstellung / Einkauf der Zutat',
  purchasedDeclaration: 'Gekauft',
  maxPersonActive: 'Maximale aktive Zubereitungszeit (Sekunden, optional)',
  availabilityWindows: 'Verfügbarkeit für Mahlzeiten',
  availabilityHelp:
    'Ohne Zeitfenster sind alle Mahlzeiten möglich. Eingetragene Zeitfenster begrenzen die Teilnahme auf diese Wochentage und Mahlzeiten.',
  addAvailability: 'Zeitfenster hinzufügen',
  ruleWeekdays: 'Wochentage (Montag = 1)',
  ruleDays: 'Mo · Di · Mi · Do · Fr · Sa · So',
  addRuleException: 'Zutatenausnahme hinzufügen',
}
export const peopleExplanations = {
  person_active_time_exceeded: {
    en: 'Active preparation exceeds a selected person’s limit.',
    de: 'Die aktive Zubereitung überschreitet die Grenze einer ausgewählten Person.',
  },
  person_unavailable: {
    en: 'A selected person is unavailable for this weekday and meal.',
    de: 'Eine ausgewählte Person ist für diesen Wochentag und diese Mahlzeit nicht verfügbar.',
  },
  invalid_person_rule_date: {
    en: 'Choose a valid meal date.',
    de: 'Wähle ein gültiges Datum für die Mahlzeit.',
  },
}
