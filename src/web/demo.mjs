// Disposable API demo, not a product web client. All interface text uses English-base message IDs.
import {resolveLocale, message, hasMessage, ingredientName, ingredientForm, matchesIngredient,
  formatDecimal, normalizeAmount, inputForLocale} from './i18n.mjs';

const $ = id => document.getElementById(id);
let savedLocale;
try { savedLocale = localStorage.getItem('mealshop.demo.locale'); } catch { /* Storage is optional. */ }
let locale = resolveLocale(savedLocale, navigator.language);
let catalogue;
let definitions;
let nextLine = 0;
let lastResult;
let statusState = {key:'loading', values:{}, error:false, preserved:false};
const t = (key, values) => message(locale, key, values);
const number = value => formatDecimal(String(value), locale);
const name = ingredient => ingredientName(ingredient, locale);
const nutrientLabel = nutrient => hasMessage('nutrient_' + nutrient.id) ? t('nutrient_' + nutrient.id) : nutrient.label;

function node(tag, text, className) {
  const element = document.createElement(tag);
  if (text !== undefined) element.textContent = text;
  if (className) element.className = className;
  return element;
}
function link(label, url) {
  const result = node('a', label);
  const parsed = new URL(url);
  if (parsed.protocol !== 'https:') throw new TypeError('Source link must use HTTPS');
  result.href = parsed.href;
  result.target = '_blank';
  result.rel = 'noopener noreferrer';
  return result;
}
function renderStatus() {
  const values = Object.fromEntries(Object.entries(statusState.values).map(([key,value]) => [key,typeof value === 'number' ? number(value) : value]));
  $('status').textContent = t(statusState.key, values) + (statusState.preserved ? ' ' + t('preserved') : '');
  $('status').className = statusState.error ? 'error' : '';
}
function status(key, values = {}, error = false, preserved = false) {
  statusState = {key, values, error, preserved};
  renderStatus();
}
function translateStatic() {
  document.documentElement.lang = locale;
  document.title = t('demo_title');
  for (const element of document.querySelectorAll('[data-i18n]')) element.textContent = t(element.dataset.i18n);
  $('language').value = locale;
  $('search').placeholder = t('search_hint');
  renderStatus();
}
function changed(event) {
  if (event?.target?.name?.startsWith('grams-')) delete event.target.dataset.invalidLocale;
  lastResult = undefined;
  $('results').hidden = true;
  status('changed');
}
function choices(select, ingredients) {
  const previous = select.value;
  select.replaceChildren(...[...ingredients].sort((a,b) => name(a).localeCompare(name(b), locale)).map(ingredient => {
    const option = node('option', name(ingredient));
    option.value = ingredient.id;
    return option;
  }));
  if (ingredients.some(ingredient => ingredient.id === previous)) select.value = previous;
}
function addLine(id = catalogue.ingredients[0]?.id, grams = '100') {
  const row = node('div', undefined, 'line');
  const ingredientLabel = node('label');
  const label = node('span', t('ingredient'));
  label.dataset.rowLabel = 'ingredient';
  const select = node('select');
  select.name = `ingredient-${nextLine}`;
  select.dataset.number = String(nextLine + 1);
  select.setAttribute('aria-label', t('ingredient_number', {number:nextLine + 1}));
  select.required = true;
  choices(select, catalogue.ingredients);
  select.value = id;
  ingredientLabel.append(label, select);
  const massLabel = node('label');
  const massText = node('span', t('grams'));
  massText.dataset.rowLabel = 'grams';
  const input = node('input');
  input.name = `grams-${nextLine++}`;
  input.type = 'text';
  input.inputMode = 'decimal';
  input.required = true;
  input.value = inputForLocale(grams, locale);
  input.maxLength = 32;
  massLabel.append(massText, input);
  const remove = node('button', '×', 'remove');
  remove.type = 'button';
  remove.setAttribute('aria-label', t('remove_ingredient'));
  remove.addEventListener('click', () => { row.remove(); changed(); });
  row.append(ingredientLabel, massLabel, remove);
  $('lines').append(row);
}
function education(note, ingredient) {
  const article = node('article', undefined, 'evidence');
  article.append(node('h4', `${name(ingredient)} · ${note.title}`), node('p', note.summary),
    node('p', t('conditions', {text:note.conditions})), node('p', t('limits', {text:note.limitations}), 'subtle'),
    link(note.evidence_reference, note.source_url));
  return article;
}
function showDetail() {
  const ingredient = catalogue.ingredients.find(i => i.id === $('ingredient-detail').value);
  const detail = $('detail');
  detail.replaceChildren();
  if (!ingredient) { detail.append(node('p', t('no_match'))); return; }
  const aliases = ingredient.translations?.[locale]?.aliases ?? ingredient.aliases;
  detail.append(node('h3', name(ingredient)), node('p', `${ingredientForm(ingredient, locale)} · ${t('basis')}`),
    node('p', t('english_name', {name:ingredient.name}), 'subtle'),
    node('p', t('aliases', {names:aliases.join(', ') || t('no_aliases')}), 'subtle'),
    link(ingredient.source.name, ingredient.source.url),
    node('p', `${ingredient.source.record_id} · ${ingredient.source.version} · ${ingredient.source.license}`, 'source'),
    node('p', t('original_text', {text:ingredient.uncertainty}), 'subtle'));
  const composition = node('details');
  composition.append(node('summary', t('composition')));
  for (const definition of definitions) {
    const reading = ingredient.nutrients[definition.id];
    const value = reading?.amount == null ? t('unknown') : `${number(reading.amount)} ${definition.unit}`;
    composition.append(node('p', `${nutrientLabel(definition)}: ${value}`));
  }
  detail.append(composition);
  for (const note of ingredient.education) detail.append(education(note, ingredient));
}
function quantity(nutrient, field) {
  return nutrient[field] === null ? t('unknown') : `${number(nutrient[field])} ${nutrient.unit}`;
}
function renderResult(result) {
  const serving = t(result.servings === 1 ? 'serving_one' : 'serving_other', {number:number(result.servings)});
  $('result-context').textContent = `${serving} · ${result.catalogue_version}`;
  $('macro-cards').replaceChildren();
  const find = id => result.nutrients.find(nutrient => nutrient.id === id);
  const available = find('available_carbohydrate');
  const byDifference = find('carbohydrate');
  // These source definitions are deliberately never combined or converted into one another.
  const carbohydrate = available?.status === 'complete' ? available : byDifference?.status === 'complete' ? byDifference : available ?? byDifference;
  for (const nutrient of [find('energy_kcal'), find('protein'), carbohydrate, find('fat')].filter(Boolean)) {
    const card = node('article', undefined, 'card');
    card.append(node('p', nutrientLabel(nutrient)), node('strong', quantity(nutrient, 'per_serving')));
    const note = nutrient.status === 'complete' ? t('per_serving') : nutrient.status === 'partial' ?
      t('known_subtotal', {amount:`${number(nutrient.known_per_serving)} ${nutrient.unit}`}) : t('no_values');
    card.append(node('p', note, 'subtle'));
    $('macro-cards').append(card);
  }
  $('warnings').replaceChildren(...result.warnings.map((original,index) => {
    const key = 'warning_' + result.warning_codes?.[index];
    return node('p', hasMessage(key) ? t(key) : t('original_text', {text:original}));
  }));
  $('nutrients').replaceChildren();
  for (const nutrient of result.nutrients) {
    const row = node('tr');
    const label = node('th', nutrientLabel(nutrient));
    label.scope = 'row';
    const missingNames = nutrient.missing_ingredients.map(id => {
      const ingredient = result.ingredients.find(i => i.ingredient_id === id);
      return ingredient ? name(ingredient) : id;
    });
    const coverage = nutrient.status === 'complete' ? t('complete') :
      t('missing', {status:t(nutrient.status === 'partial' ? 'partial' : 'unknown'), names:missingNames.join(', ')});
    const total = node('td', quantity(nutrient, 'total'));
    const portion = node('td', quantity(nutrient, 'per_serving'));
    if (nutrient.status === 'partial') {
      total.append(node('p', t('known_subtotal', {amount:`${number(nutrient.known_total)} ${nutrient.unit}`}), 'subtle'));
      portion.append(node('p', t('known_subtotal', {amount:`${number(nutrient.known_per_serving)} ${nutrient.unit}`}), 'subtle'));
    }
    const info = node('td', coverage, nutrient.status === 'complete' ? 'subtle' : 'partial');
    const details = node('details');
    details.append(node('summary', t('uncertainty')));
    for (const uncertainty of nutrient.uncertainty) details.append(node('p', uncertainty));
    info.append(details);
    row.append(label, total, portion, info);
    $('nutrients').append(row);
  }
  $('provenance').replaceChildren();
  $('education').replaceChildren();
  const seen = new Set();
  for (const ingredient of result.ingredients) {
    const article = node('article');
    article.append(node('h4', `${name(ingredient)} · ${number(ingredient.grams)} g`),
      node('p', `${ingredientForm(ingredient, locale)} · ${t('basis')}`),
      link(ingredient.source.name, ingredient.source.url),
      node('p', `${ingredient.source.record_id} · ${ingredient.source.version} · ${t('retrieved', {date:ingredient.source.retrieved_on})} · ${ingredient.source.license}`, 'source'),
      node('p', t('original_text', {text:ingredient.uncertainty})));
    $('provenance').append(article);
    if (!seen.has(ingredient.ingredient_id)) {
      for (const note of ingredient.education) $('education').append(education(note, ingredient));
      seen.add(ingredient.ingredient_id);
    }
  }
  if (!$('education').childElementCount) $('education').append(node('p', t('no_evidence')));
  $('rounding').textContent = t('rounding');
  $('results').hidden = false;
}
async function calculate(event) {
  event?.preventDefault();
  $('results').hidden = true;
  lastResult = undefined;
  try {
    const lines = [...$('lines').children].map(row => {
      const input = row.querySelector('input');
      if (input.dataset.invalidLocale) throw Object.assign(new RangeError('Invalid localized gram amount'), {code:'invalid_amount'});
      return {ingredient_id:row.querySelector('select').value, grams:normalizeAmount(input.value, locale)};
    });
    const request = {lines, servings:Number($('servings').value), preparation:$('preparation').value};
    $('recipe-controls').disabled = true;
    status('calculating');
    const response = await fetch('/api/nutrition/calculate', {
      method:'POST', headers:{'Content-Type':'application/json'}, body:JSON.stringify(request), signal:AbortSignal.timeout(10000),
    });
    const result = await response.json();
    if (!response.ok) throw Object.assign(new Error('Calculation request failed'), {code:result.code});
    lastResult = result;
    renderResult(result);
    status('estimate_ready');
  } catch (error) {
    const key = 'error_' + error.code;
    status(hasMessage(key) ? key : 'error_request_failed', {}, true, true);
  } finally { $('recipe-controls').disabled = false; }
}
function filterNotebook() {
  choices($('ingredient-detail'), catalogue.ingredients.filter(i => matchesIngredient(i, $('search').value)));
  showDetail();
}
function switchLanguage() {
  const previous = locale;
  locale = resolveLocale($('language').value);
  let invalid = false;
  for (const row of $('lines').children) {
    const input = row.querySelector('input');
    try {
      if (input.dataset.invalidLocale) throw new RangeError('Previously invalid gram amount');
      input.value = inputForLocale(normalizeAmount(input.value, previous), locale);
    } catch { input.dataset.invalidLocale = 'true'; invalid = true; }
    for (const label of row.querySelectorAll('[data-row-label]')) label.textContent = t(label.dataset.rowLabel);
    const select = row.querySelector('select');
    choices(select, catalogue.ingredients);
    select.setAttribute('aria-label', t('ingredient_number', {number:select.dataset.number}));
    row.querySelector('button').setAttribute('aria-label', t('remove_ingredient'));
  }
  try { localStorage.setItem('mealshop.demo.locale', locale); } catch { /* A session-only choice still works. */ }
  translateStatic();
  if (catalogue) filterNotebook();
  if (invalid) { lastResult = undefined; $('results').hidden = true; status('switch_invalid', {}, true); }
  else if (lastResult) renderResult(lastResult);
}
async function initialize() {
  $('retry').hidden = true;
  status('loading');
  try {
    const response = await fetch('/api/ingredients', {signal:AbortSignal.timeout(10000)});
    if (!response.ok) throw new Error('Catalogue request failed');
    catalogue = await response.json();
    definitions = catalogue.nutrient_definitions;
    if (!catalogue.ingredients.length) { status('empty'); return; }
    choices($('ingredient-detail'), catalogue.ingredients);
    $('search').disabled = false;
    $('ingredient-detail').disabled = false;
    $('recipe-controls').disabled = false;
    $('lines').replaceChildren(); nextLine = 0;
    if (catalogue.ingredients.some(i => i.id === 'oats') && catalogue.ingredients.some(i => i.id === 'chia')) {
      addLine('oats', '100'); addLine('chia', '25');
    } else addLine(catalogue.ingredients[0].id);
    showDetail();
    status('ready', {count:catalogue.ingredients.length, nutrients:definitions.length});
  } catch { status('load_failed', {}, true); $('retry').hidden = false; }
}
$('recipe-form').addEventListener('submit', calculate);
$('recipe-form').addEventListener('input', changed);
$('recipe-form').addEventListener('change', changed);
$('add-line').addEventListener('click', () => { addLine(); changed(); });
$('ingredient-detail').addEventListener('change', showDetail);
$('search').addEventListener('input', filterNotebook);
$('language').addEventListener('change', switchLanguage);
$('retry').addEventListener('click', initialize);
translateStatic();
initialize();
