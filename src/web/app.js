'use strict';

const $ = id => document.getElementById(id);
let catalogue;
let definitions;
let nextLine = 0;

function node(tag, text, className) {
  const element = document.createElement(tag);
  if (text !== undefined) element.textContent = text;
  if (className) element.className = className;
  return element;
}
function link(label, url) {
  const result = node('a', label);
  const parsed = new URL(url);
  if (parsed.protocol !== 'https:') throw new Error('Source link must use HTTPS');
  result.href = parsed.href;
  result.target = '_blank';
  result.rel = 'noopener noreferrer';
  return result;
}
function status(text, error = false) {
  $('status').textContent = text;
  $('status').className = error ? 'error' : '';
}
function changed() {
  $('results').hidden = true;
  status('Inputs changed. Calculate to see the updated estimate.');
}
function choices(select, ingredients) {
  select.replaceChildren(...ingredients.map(i => {
    const option = node('option', i.name);
    option.value = i.id;
    return option;
  }));
}
function addLine(id = catalogue.ingredients[0]?.id, grams = '100') {
  const row = node('div', undefined, 'line');
  const ingredientLabel = node('label', 'Ingredient');
  const select = node('select');
  select.name = `ingredient-${nextLine}`;
  select.setAttribute('aria-label', `Ingredient ${nextLine + 1}`);
  select.required = true;
  choices(select, catalogue.ingredients);
  select.value = id;
  ingredientLabel.append(select);
  const massLabel = node('label', 'Grams');
  const input = node('input');
  input.name = `grams-${nextLine++}`;
  input.type = 'text';
  input.inputMode = 'decimal';
  input.required = true;
  input.value = grams;
  input.maxLength = 32;
  massLabel.append(input);
  const remove = node('button', '×', 'remove');
  remove.type = 'button';
  remove.setAttribute('aria-label', 'Remove ingredient line');
  remove.addEventListener('click', () => { row.remove(); changed(); });
  row.append(ingredientLabel, massLabel, remove);
  $('lines').append(row);
}
function education(note, ingredientName) {
  const article = node('article', undefined, 'evidence');
  article.append(node('h4', `${ingredientName} · ${note.title}`), node('p', note.summary),
    node('p', `Applies when: ${note.conditions}`), node('p', `Limits: ${note.limitations}`, 'subtle'),
    link(note.evidence_reference, note.source_url));
  return article;
}
function showDetail() {
  const ingredient = catalogue.ingredients.find(i => i.id === $('ingredient-detail').value);
  const detail = $('detail');
  detail.replaceChildren();
  if (!ingredient) { detail.append(node('p', 'No matching ingredients.')); return; }
  detail.append(node('h3', ingredient.name), node('p', `${ingredient.form} · per 100 g edible portion`),
    node('p', `Also known as: ${ingredient.aliases.join(', ') || 'No aliases recorded'}`, 'subtle'),
    link(ingredient.source.name, ingredient.source.url),
    node('p', `${ingredient.source.record_id} · ${ingredient.source.version} · ${ingredient.source.license}`, 'source'),
    node('p', ingredient.uncertainty, 'subtle'));
  const composition = node('details');
  composition.append(node('summary', 'Composition per 100 g'));
  for (const definition of definitions) {
    const reading = ingredient.nutrients[definition.id];
    const value = reading?.amount == null ? 'Unknown' : `${reading.amount} ${definition.unit}`;
    composition.append(node('p', `${definition.label}: ${value}`));
  }
  detail.append(composition);
  for (const note of ingredient.education) detail.append(education(note, ingredient.name));
}
function quantity(nutrient, field) {
  return nutrient[field] === null ? 'Unknown' : `${nutrient[field]} ${nutrient.unit}`;
}
function renderResult(result) {
  $('result-context').textContent = `${result.servings} serving(s) · ${result.catalogue_version}`;
  $('macro-cards').replaceChildren();
  for (const id of ['energy_kcal', 'protein', 'carbohydrate', 'fat']) {
    const nutrient = result.nutrients.find(n => n.id === id);
    const card = node('article', undefined, 'card');
    card.append(node('p', nutrient.label), node('strong', quantity(nutrient, 'per_serving')));
    card.append(node('p', nutrient.status === 'complete' ? 'per serving' :
      `${nutrient.status === 'partial' ? 'Known subtotal' : 'No measured values'}${nutrient.status === 'partial' ? `: ${nutrient.known_per_serving} ${nutrient.unit}` : ''}`, 'subtle'));
    $('macro-cards').append(card);
  }
  $('warnings').replaceChildren(...result.warnings.map(w => node('p', w)));
  $('nutrients').replaceChildren();
  for (const nutrient of result.nutrients) {
    const row = node('tr');
    const label = node('th', nutrient.label);
    label.scope = 'row';
    let coverage = 'Complete source coverage; source uncertainty still applies.';
    if (nutrient.status !== 'complete') coverage = `${nutrient.status === 'partial' ? 'Partial' : 'Unknown'}; missing: ${nutrient.missing_ingredients.join(', ')}`;
    const total = node('td', quantity(nutrient, 'total'));
    const portion = node('td', quantity(nutrient, 'per_serving'));
    if (nutrient.status === 'partial') {
      total.append(node('p', `Known subtotal: ${nutrient.known_total} ${nutrient.unit}`, 'subtle'));
      portion.append(node('p', `Known subtotal: ${nutrient.known_per_serving} ${nutrient.unit}`, 'subtle'));
    }
    const info = node('td', coverage, nutrient.status === 'complete' ? 'subtle' : 'partial');
    const details = node('details');
    details.append(node('summary', 'Source uncertainty'));
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
    article.append(node('h4', `${ingredient.name} · ${ingredient.grams} g`),
      node('p', `${ingredient.form} · per 100 g edible portion`),
      link(ingredient.source.name, ingredient.source.url),
      node('p', `${ingredient.source.record_id} · ${ingredient.source.version} · retrieved ${ingredient.source.retrieved_on} · ${ingredient.source.license}`, 'source'),
      node('p', ingredient.uncertainty));
    $('provenance').append(article);
    if (!seen.has(ingredient.ingredient_id)) {
      for (const note of ingredient.education) $('education').append(education(note, ingredient.name));
      seen.add(ingredient.ingredient_id);
    }
  }
  if (!$('education').childElementCount) $('education').append(node('p', 'No evidence notes recorded for these ingredients.'));
  $('rounding').textContent = result.rounding;
  $('results').hidden = false;
}
async function calculate(event) {
  event?.preventDefault();
  const lines = [...$('lines').children].map(row => ({
    ingredient_id: row.querySelector('select').value,
    grams: row.querySelector('input').value,
  }));
  const request = { lines, servings: Number($('servings').value), preparation: $('preparation').value };
  $('recipe-controls').disabled = true;
  $('results').hidden = true;
  status('Calculating…');
  try {
    const response = await fetch('/api/nutrition/calculate', {
      method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify(request),
      signal: AbortSignal.timeout(10000),
    });
    const result = await response.json();
    if (!response.ok) throw new Error(result.error || `Calculation failed (${response.status})`);
    renderResult(result);
    status('Estimate ready. Review source coverage and preparation limits below.');
  } catch (error) {
    status(`Could not calculate: ${error.message.replace(/[.!?]+$/u, '')}. Your inputs have been kept.`, true);
  } finally { $('recipe-controls').disabled = false; }
}
async function initialize() {
  $('retry').hidden = true;
  status('Loading ingredient catalogue…');
  try {
    const response = await fetch('/api/ingredients', {signal: AbortSignal.timeout(10000)});
    if (!response.ok) throw new Error(`Catalogue request failed (${response.status})`);
    catalogue = await response.json();
    definitions = catalogue.nutrient_definitions;
    if (!catalogue.ingredients.length) { status('This catalogue is empty. Configure ingredients before calculating.'); return; }
    choices($('ingredient-detail'), catalogue.ingredients);
    $('search').disabled = false;
    $('ingredient-detail').disabled = false;
    $('recipe-controls').disabled = false;
    $('lines').replaceChildren();
    addLine(catalogue.ingredients[0].id);
    if (catalogue.ingredients.some(i => i.id === 'oats') && catalogue.ingredients.some(i => i.id === 'chia')) {
      $('lines').replaceChildren(); nextLine = 0; addLine('oats', '100'); addLine('chia', '25');
    }
    showDetail();
    status(`Ready · ${catalogue.ingredients.length} ingredients · ${definitions.length} nutrients/components`);
  } catch (error) { status(`Could not load catalogue: ${error.message}`, true); $('retry').hidden = false; }
}
$('recipe-form').addEventListener('submit', calculate);
$('recipe-form').addEventListener('input', changed);
$('recipe-form').addEventListener('change', changed);
$('add-line').addEventListener('click', () => { addLine(); changed(); });
$('ingredient-detail').addEventListener('change', showDetail);
$('search').addEventListener('input', () => {
  const query = $('search').value.trim().toLowerCase();
  const ingredients = catalogue.ingredients.filter(i => [i.name, ...i.aliases].some(s => s.toLowerCase().includes(query)));
  choices($('ingredient-detail'), ingredients);
  showDetail();
});
$('retry').addEventListener('click', initialize);
initialize();
