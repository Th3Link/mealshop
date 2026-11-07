// Disposable API demo, not a product web client. All interface text uses English-base message IDs.
import {resolveLocale, message, hasMessage, ingredientName, ingredientForm, matchesIngredient,
  formatDecimal, normalizeAmount, inputForLocale} from './i18n.mjs';

const $ = id => document.getElementById(id);
let savedLocale;
try { savedLocale = localStorage.getItem('mealshop.demo.locale'); } catch { /* Storage is optional. */ }
let locale = resolveLocale(savedLocale, navigator.language);
let catalogue;
let definitions;
let curatedConversions = [];
let nextLine = 0;
let lastResult;
let loadedRecipe;
let savedRecipes = [];
let storageState = {key:'no_recipes', values:{}};
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
  renderStorageStatus();
  renderSavedChoices();
}
function clearScale() { $('scale-status').textContent = ''; $('scaled-lines').replaceChildren(); }
function changed(event) {
  clearScale();
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
  else if (previous && select.name.startsWith('ingredient-')) missingOption(select,previous);
}
function missingOption(select,id) {
  const option = node('option',t('missing_catalogue_ingredient',{id})); option.value = id;
  select.append(option); select.value = id;
}
function addLine(id = catalogue.ingredients[0]?.id, grams = '100', original = null, evidence = null) {
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
  if (catalogue.ingredients.some(i => i.id === id)) select.value = id;
  else missingOption(select,id);
  ingredientLabel.append(label, select);
  const massLabel = node('label');
  const massText = node('span', t('quantity_amount'));
  massText.dataset.rowLabel = 'quantity_amount';
  const input = node('input');
  input.name = `grams-${nextLine++}`;
  input.type = 'text';
  input.inputMode = 'decimal';
  input.required = true;
  input.value = inputForLocale(original?.amount ?? grams, locale);
  input.maxLength = 32;
  massLabel.append(massText, input);
  const remove = node('button', '×', 'remove');
  remove.type = 'button';
  remove.setAttribute('aria-label', t('remove_ingredient'));
  remove.addEventListener('click', () => { row.remove(); changed(); });
  const unit = node('select'); unit.dataset.unit = 'true'; unit.setAttribute('aria-label',t('quantity_unit'));
  for (const value of ['g','kg','ml','l','tsp','tbsp','piece']) { const option=node('option',t('unit_'+value)); option.dataset.i18n='unit_'+value; option.value=value; unit.append(option); }
  unit.value=original?.unit ?? 'g'; massLabel.append(unit);
  const context=node('details'); context.append(node('summary',t('conversion_evidence')));
  for (const key of ['grams_per_base_unit','source_url','version','uncertainty','spoon_standard','millilitres_per_spoon']) {
    const label=node('label'); const caption=node('span',t('conversion_'+key)); caption.dataset.i18n='conversion_'+key;
    const field=node('input'); field.dataset.conversion=key;
    field.value=evidence?.[key] ?? ''; label.append(caption,field); context.append(label);
  }
  const curatedLabel=node('label'); curatedLabel.append(node('span',t('curated_conversion')));
  const curated=node('select'); curated.dataset.curated='true';
  const manual=node('option',t('manual_conversion')); manual.value=''; curated.append(manual);
  const refreshCurated = () => {
  curated.replaceChildren(manual);
  for (const item of curatedConversions.filter(c=>c.evidence.ingredient_id===select.value)) {
    const option=node('option',item.label); option.value=item.id; curated.append(option);
  }
  };
  refreshCurated();
  if (evidence?.input_unit) { const saved=node('option',evidence.version); saved.value='saved'; curated.append(saved); curated.value='saved'; row.savedEvidence=evidence; }
  select.addEventListener('change',()=>{ refreshCurated(); for(const field of context.querySelectorAll('[data-conversion]')) field.value=''; row.savedEvidence=null; });
  curated.addEventListener('change',()=>{ const item=curatedConversions.find(c=>c.id===curated.value); if(item) unit.value=item.unit; changed(); });
  curatedLabel.append(curated); context.prepend(curatedLabel);
  row.append(ingredientLabel, massLabel, remove, context);
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
    article.append(node('h4', `${name(ingredient)} · ${displayMass(ingredient.grams)}`),
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
    const lines = recipeLines();
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
  clearScale();
  if ($('recipe-controls').disabled) return;
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
    curatedConversions = (await recipeRequest('/api/conversions')).entries;
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
    await refreshRecipes();
    status('ready', {count:catalogue.ingredients.length, nutrients:definitions.length});
  } catch { status('load_failed', {}, true); $('retry').hidden = false; }
}
function addStep(step = {instruction:'',active_seconds:0,passive_seconds:0}) {
  const row = node('fieldset');
  const label = node('label');
  const title = node('span',t('step_instruction')); title.dataset.i18n = 'step_instruction';
  const input = node('textarea'); input.rows = 2; input.maxLength = 4000; input.value = step.instruction;
  label.append(title,input); row.append(label);
  for (const key of ['active_seconds','passive_seconds']) {
    const label = node('label'); const title = node('span',t(key)); title.dataset.i18n = key;
    const input = node('input'); input.type = 'number'; input.min = '0'; input.max = '604800'; input.step = '1';
    input.required = true; input.value = step[key]; input.dataset.duration = key;
    label.append(title,input); row.append(label);
  }
  const remove = node('button',t('remove_step')); remove.dataset.i18n = 'remove_step'; remove.type = 'button';
  remove.addEventListener('click',() => { row.remove(); changed(); }); row.append(remove);
  $('recipe-steps').append(row);
}
function renderStorageStatus() {
  $('recipe-storage-status').textContent = t(storageState.key, storageState.values) +
    (loadedRecipe ? ' ' + t('loaded_revision', {id:loadedRecipe.id,revision:number(loadedRecipe.revision)}) : '');
}
function storageStatus(key, values = {}) { storageState = {key,values}; renderStorageStatus(); }
function renderSavedChoices() {
  const selected = $('saved-recipes').value;
  $('saved-recipes').replaceChildren(...savedRecipes.map(recipe => {
    const option = node('option', `${recipe.title} · ${recipe.id} · ${number(recipe.revision)}`);
    option.value = recipe.id;
    return option;
  }));
  if (!savedRecipes.length) $('saved-recipes').append(node('option', t('no_recipes')));
  if (savedRecipes.some(r => r.id === selected)) $('saved-recipes').value = selected;
  $('load-recipe').disabled = savedRecipes.length === 0;
}
async function recipeRequest(path, options = {}) {
  const response = await fetch(path, {...options, cache:'no-store', signal:AbortSignal.timeout(10000)});
  if (!response.ok) {
    let code = 'request_failed';
    try { code = (await response.json()).code; } catch { /* Preserve local input on every failure. */ }
    throw Object.assign(new Error('Recipe request failed'), {code});
  }
  return response.json();
}
async function refreshRecipes() {
  try {
    const result = await recipeRequest('/api/recipes');
    savedRecipes = result.recipes;
    renderSavedChoices();
    storageStatus('storage_ready', {count:number(savedRecipes.length)});
  } catch { storageStatus('storage_unavailable'); }
}
function recipeLines() {
  return [...$('lines').children].map(row => {
    const input = row.querySelector('input');
    if (input.dataset.invalidLocale) throw Object.assign(new Error('Invalid amount'), {code:'invalid_amount'});
    const id=row.querySelector('select').value;
    const quantity={amount:normalizeAmount(input.value,locale),unit:row.querySelector('[data-unit]').value};
    const result={ingredient_id:id,quantity};
    if (!['g','kg'].includes(quantity.unit)) {
      const conversion={ingredient_id:id,form:catalogue.ingredients.find(i=>i.id===id)?.form};
      for (const field of row.querySelectorAll('[data-conversion]')) conversion[field.dataset.conversion]=field.value || null;
      const selected=curatedConversions.find(c=>c.id===row.querySelector('[data-curated]').value);
      if (!selected && row.querySelector('[data-curated]').value!=='saved' && (!conversion.grams_per_base_unit || !conversion.source_url || !conversion.version || !conversion.uncertainty)) throw Object.assign(new Error('Missing conversion'),{code:'conversion_required'});
      if (['tsp','tbsp'].includes(quantity.unit) && !selected && row.querySelector('[data-curated]').value!=='saved' && (!conversion.spoon_standard || !conversion.millilitres_per_spoon)) throw Object.assign(new Error('Missing spoon context'),{code:'spoon_required'});
      result.conversion=row.querySelector('[data-curated]').value==='saved' ? row.savedEvidence : selected ? selected.evidence : conversion;
    }
    return result;
  });
}
function draft() {
  return {
    title:$('recipe-name').value,author:$('recipe-author').value,language:$('recipe-language').value,
    lines:recipeLines(),servings:Number($('servings').value),preparation:$('preparation').value,
    steps:[...$('recipe-steps').children].map(row => ({instruction:row.querySelector('textarea').value,
      active_seconds:Number(row.querySelector('[data-duration="active_seconds"]').value),
      passive_seconds:Number(row.querySelector('[data-duration="passive_seconds"]').value)})),
  };
}
async function saveRecipe() {
  if (!$('recipe-form').reportValidity()) return;
  let recipe;
  try {
    recipe = draft();
    if (!recipe.title.trim() || !recipe.author.trim()) { storageStatus('recipe_invalid'); return; }
  } catch (error) { storageStatus(hasMessage('error_' + error.code) ? 'error_' + error.code : 'recipe_invalid'); return; }
  $('recipe-controls').disabled = true;
  storageStatus('recipe_saving');
  try {
    const result = await recipeRequest(loadedRecipe ? '/api/recipes/' + loadedRecipe.id : '/api/recipes', {
      method:loadedRecipe ? 'PUT' : 'POST', headers:{'Content-Type':'application/json'},
      body:JSON.stringify(loadedRecipe ? {expected_revision:loadedRecipe.revision,recipe} : recipe),
    });
    loadedRecipe = result;
    lastResult = result.nutrition;
    renderResult(lastResult);
    status('estimate_ready');
    await refreshRecipes();
    $('saved-recipes').value = result.id;
    storageStatus('recipe_saved', {id:result.id,revision:number(result.revision)});
  } catch (error) { storageStatus(hasMessage('error_' + error.code) ? 'error_' + error.code : 'storage_unavailable'); }
  finally { $('recipe-controls').disabled = false; }
}
async function loadRecipe() {
  clearScale();
  const id = $('saved-recipes').value;
  if (!savedRecipes.some(r => r.id === id)) return;
  $('recipe-controls').disabled = true;
  try {
    const result = await recipeRequest('/api/recipes/' + id);
    loadedRecipe = result;
    const recipe = result.recipe;
    $('recipe-name').value = recipe.title;
    $('recipe-author').value = recipe.author;
    $('recipe-language').value = recipe.language;
    $('recipe-steps').replaceChildren();
    for (const step of recipe.steps) addStep(step);
    $('servings').value = recipe.servings;
    $('preparation').value = recipe.preparation;
    $('lines').replaceChildren(); nextLine = 0;
    const missing = recipe.lines.some(line => !catalogue.ingredients.some(i => i.id === line.ingredient_id));
    for (const line of recipe.lines) addLine(line.ingredient_id,line.grams,line.quantity,line.conversion);
    lastResult = result.nutrition;
    renderResult(lastResult);
    status('estimate_ready');
    storageStatus(missing ? 'recipe_missing_ingredients' : 'recipe_loaded', {id:result.id,revision:number(result.revision)});
  } catch (error) { storageStatus(hasMessage('error_' + error.code) ? 'error_' + error.code : 'storage_unavailable'); }
  finally { $('recipe-controls').disabled = false; }
}
function newRecipe() {
  loadedRecipe = undefined;
  $('recipe-name').value = ''; $('recipe-author').value = ''; $('recipe-steps').replaceChildren();
  $('recipe-language').value = locale;
  $('servings').value = '2'; $('preparation').value = 'as_recorded';
  $('lines').replaceChildren(); nextLine = 0; addLine(); changed();
  storageStatus('recipe_new');
}
$('add-step').addEventListener('click',() => { addStep(); changed(); });
$('scale-recipe').addEventListener('click', async () => {
  clearScale();
  if (!loadedRecipe) { $('scale-status').textContent = t('scale_load_first'); return; }
  const count = $('target-servings').value;
  try {
    const result = await recipeRequest(`/api/recipes/${loadedRecipe.id}/scale?revision=${loadedRecipe.revision}&servings=${encodeURIComponent(count)}`);
    $('scale-status').textContent = t('scale_summary', {original:number(result.original_servings),target:number(result.servings)});
    $('scaled-lines').replaceChildren(...result.lines.map(line => {
      const ingredient = catalogue.ingredients.find(i => i.id === line.ingredient_id);
      return node('p', `${ingredient ? name(ingredient) : line.ingredient_id}: ${displayMass(line.grams)}`);
    }));
    lastResult = result.nutrition; renderResult(lastResult); status('estimate_ready');
  } catch { $('scale-status').textContent = t('scale_error'); }
});
$('save-recipe').addEventListener('click', saveRecipe);
$('load-recipe').addEventListener('click', loadRecipe);
$('new-recipe').addEventListener('click', newRecipe);
$('refresh-recipes').addEventListener('click', refreshRecipes);
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

function displayMass(grams) {
  if ($('unit-preference').value === 'kg' || ($('unit-preference').value === 'g_kg' && BigInt(grams.split('.')[0]) >= 1000n)) {
    const [whole,fraction='']=grams.split('.');
    const digits=(BigInt(whole)*1000000n+BigInt(fraction.padEnd(6,'0'))).toString().padStart(10,'0');
    return number(digits.slice(0,-9)+'.'+digits.slice(-9))+' kg';
  }
  return number(grams)+' g';
}
try { $('unit-preference').value=['g','kg','g_kg'].includes(localStorage.getItem('mealshop.demo.massUnit'))?localStorage.getItem('mealshop.demo.massUnit'):'g'; } catch {}
$('unit-preference').addEventListener('change',()=>{ clearScale(); try {localStorage.setItem('mealshop.demo.massUnit',$('unit-preference').value);} catch {} });

let household = {revision:0,members:[]};
let householdAllergens=[];
async function householdRequest(path,options={}) {
  const response=await fetch('/api/household/'+path,{...options,cache:'no-store',headers:{'Content-Type':'application/json','Authorization':'Bearer '+$('household-token').value},signal:AbortSignal.timeout(10000)});
  if (!response.ok) throw new Error('household request failed');
  return response.json();
}
function householdEditor() {
  const member=household.members.find(m=>m.id===$('household-member').value);
  $('household-name').value=member?.name ?? '';
  $('household-allergies').value=(member?.allergies ?? []).join('\n');
  $('household-intolerances').value=(member?.intolerances ?? []).join('\n');
  $('household-allergen-ids').replaceChildren(...householdAllergens.map(a=>{const option=node('option',locale==='de'?a.translations.de:a.name);option.value=a.id;option.selected=member?.allergen_ids?.includes(a.id) ?? false;return option;}));
  for (const [element,field] of [['household-excluded','excluded_ingredients'],['household-disliked','disliked_ingredients']]) {
    const select=$(element); select.replaceChildren(...catalogue.ingredients.map(i=>{const option=node('option',name(i));option.value=i.id;option.selected=member?.[field].includes(i.id) ?? false;return option;}));
  }
}
function householdRender() {
  const selected=$('household-member').value;
  const fresh=node('option',t('household_new'));fresh.value='';
  $('household-member').replaceChildren(fresh,...household.members.map(m=>{const o=node('option',m.name);o.value=m.id;return o;}));
  if(household.members.some(m=>m.id===selected)) $('household-member').value=selected;
  householdEditor();
}
async function householdAction(action) {
  try {await action();} catch {$('household-status').textContent=t('household_failed');}
}
$('household-load').addEventListener('click',()=>householdAction(async()=>{householdAllergens=(await householdRequest('allergens')).groups;household=await householdRequest('profiles');householdRender();$('household-status').textContent=t('household_loaded');}));
$('household-member').addEventListener('change',householdEditor);
async function householdWrite(members) {
  household=await householdRequest('profiles',{method:'PUT',body:JSON.stringify({expected_revision:household.revision,members})});
  householdRender();$('household-status').textContent=t('household_saved');
}
$('household-save').addEventListener('click',()=>householdAction(async()=>{
  const id=$('household-member').value || 'member-'+crypto.randomUUID();
  const member={id,name:$('household-name').value,allergen_ids:[...$('household-allergen-ids').selectedOptions].map(o=>o.value),intolerances:$('household-intolerances').value.split('\n').filter(s=>s.trim()),allergies:$('household-allergies').value.split('\n').filter(s=>s.trim()),excluded_ingredients:[...$('household-excluded').selectedOptions].map(o=>o.value),disliked_ingredients:[...$('household-disliked').selectedOptions].map(o=>o.value)};
  const members=household.members.filter(m=>m.id!==id);members.push(member);await householdWrite(members);
}));
$('household-remove').addEventListener('click',()=>householdAction(async()=>{const id=$('household-member').value;if(id)await householdWrite(household.members.filter(m=>m.id!==id));}));
$('household-evaluate').addEventListener('click',()=>householdAction(async()=>{
  const result=await householdRequest('evaluate',{method:'POST',body:JSON.stringify({member_ids:household.members.map(m=>m.id),ingredient_ids:recipeLines().map(l=>l.ingredient_id)})});
  $('household-status').textContent=t('household_'+result.status)+' '+result.excluded_ingredients.map(id=>name(catalogue.ingredients.find(i=>i.id===id))).join(', ');
}));
