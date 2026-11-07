// Demo-only presentation helpers. Canonical data and API amounts remain English/dot-decimal.
import en from './locales/en.mjs';
import de from './locales/de.mjs';

export const resources = { en, de };
export function resolveLocale(preferred, browser = 'en') {
  const supported = value => {
    if (typeof value !== 'string') return undefined;
    const base = value.trim().toLowerCase().split('-')[0];
    return Object.hasOwn(resources, base) ? base : undefined;
  };
  return supported(preferred) ?? supported(browser) ?? 'en';
}
export function hasMessage(key) { return Object.hasOwn(en, key); }
export function message(locale, key, values = {}) {
  const text = resources[resolveLocale(locale)][key] ?? en[key] ?? resources[resolveLocale(locale)].unavailable;
  return text.replace(/\{([a-z_]+)\}/gu, (match, name) => Object.hasOwn(values, name) ? String(values[name]) : match);
}
export function ingredientName(ingredient, locale) {
  return ingredient.translations?.[resolveLocale(locale)]?.name || ingredient.name;
}
export function ingredientForm(ingredient, locale) {
  return ingredient.translations?.[resolveLocale(locale)]?.form || ingredient.form;
}
export function matchesIngredient(ingredient, query) {
  const texts = [ingredient.name, ...ingredient.aliases];
  for (const translated of Object.values(ingredient.translations ?? {})) {
    texts.push(translated.name, ...(translated.aliases ?? []));
  }
  const combined = texts.join(' ').toLowerCase();
  return query.trim().toLowerCase().split(/\s+/u).every(term => combined.includes(term));
}
export function formatDecimal(value, locale) {
  if (!/^\d+(?:\.\d+)?$/u.test(value)) throw new TypeError('Expected a canonical decimal string');
  const [whole, fraction] = value.split('.');
  const language = resolveLocale(locale);
  const integer = new Intl.NumberFormat(language, {useGrouping:true}).format(BigInt(whole));
  return fraction === undefined ? integer : integer + (language === 'de' ? ',' : '.') + fraction;
}
export function normalizeAmount(value, locale) {
  const invalid = () => Object.assign(new RangeError('Invalid localized gram amount'), {code:'invalid_amount'});
  const language = resolveLocale(locale);
  if (typeof value !== 'string' || value.length > 32) throw invalid();
  const pattern = language === 'de' ? /^\d+(?:,\d{1,6})?$/u : /^\d+(?:\.\d{1,6})?$/u;
  if (!pattern.test(value)) throw invalid();
  const canonical = language === 'de' ? value.replace(',', '.') : value;
  const [whole, fraction = ''] = canonical.split('.');
  const atoms = BigInt(whole) * 1000000n + BigInt(fraction.padEnd(6, '0'));
  if (atoms <= 0n || atoms > 1000000000000000n) throw invalid();
  return canonical;
}
export function inputForLocale(canonical, locale) {
  return resolveLocale(locale) === 'de' ? canonical.replace('.', ',') : canonical;
}
