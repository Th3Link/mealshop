import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {resources, resolveLocale, message, ingredientName, ingredientForm, matchesIngredient,
  formatDecimal, normalizeAmount, inputForLocale} from '../src/web/i18n.mjs';

const html = readFileSync(new URL('../src/web/index.html', import.meta.url), 'utf8');
const catalogue = JSON.parse(readFileSync(new URL('../data/catalogue.json', import.meta.url), 'utf8'));

test('English is the complete base and German resources preserve identifiers/placeholders', () => {
  assert.deepEqual(Object.keys(resources.de).sort(), Object.keys(resources.en).sort());
  for (const key of Object.keys(resources.en)) {
    assert.equal(typeof resources.de[key], 'string');
    assert.ok(resources.de[key].trim().length > 0, key);
    assert.deepEqual(resources.en[key].match(/\{[a-z_]+\}/gu), resources.de[key].match(/\{[a-z_]+\}/gu), key);
  }
  for (const [,key] of html.matchAll(/data-i18n="([a-z_]+)"/gu)) assert.ok(Object.hasOwn(resources.en,key),key);
  assert.equal(message('de', 'ingredient_number', {number:2}), 'Zutat 2');
  assert.equal(message('xx', 'ingredient'), resources.en.ingredient);
  assert.equal(message('de', 'nonexistent_identifier'), resources.de.unavailable);
});

test('regional and saved locale preferences resolve predictably', () => {
  assert.equal(resolveLocale('de-AT','en'), 'de');
  assert.equal(resolveLocale('en-GB','de'), 'en');
  assert.equal(resolveLocale('unsupported','de-DE'), 'de');
  assert.equal(resolveLocale(undefined,'fr-FR'), 'en');
  assert.equal(resolveLocale(null,null), 'en');
});

test('decimal display remains exact beyond JavaScript safe integer precision', () => {
  assert.equal(formatDecimal('1000000000000000000.01','de'), '1.000.000.000.000.000.000,01');
  assert.equal(formatDecimal('1000000000000000000.01','en'), '1,000,000,000,000,000,000.01');
  assert.equal(formatDecimal('0.000001','de'), '0,000001');
  assert.equal(formatDecimal('0.00','de'), '0,00');
  assert.throws(() => formatDecimal('1,25','de'));
});

test('localized input converts losslessly and does not guess grouping', () => {
  assert.equal(normalizeAmount('00100,500000','de'), '00100.500000');
  assert.equal(normalizeAmount('0.000001','en'), '0.000001');
  for (const canonical of ['100.500000','0.000001','1000000000','001.2300']) {
    const german = inputForLocale(canonical,'de');
    assert.equal(normalizeAmount(german,'de'),canonical);
    assert.equal(inputForLocale(normalizeAmount(german,'de'),'en'),canonical);
  }
  for (const [locale,value] of [['de','1.000'],['en','1,000'],['de','1.000,50'],['de','0,0000001'],
    ['de','1000000000,000001'],['de','0'],['en','-1'],['en','1e2'],['de',' 1,5 ']]) {
    assert.throws(() => normalizeAmount(value,locale), error => error.code === 'invalid_amount', `${locale}: ${value}`);
  }
});

test('ingredient identity and English base survive localized display and multilingual search', () => {
  const flour = catalogue.ingredients.find(i => i.id === 'wheat-flour-type-405');
  assert.equal(catalogue.language, 'en');
  assert.equal(ingredientName(flour,'en'), 'Wheat flour, Type 405');
  assert.equal(ingredientName(flour,'de'), 'Weizen Mehl, Type 405');
  assert.equal(flour.id, 'wheat-flour-type-405');
  assert.ok(matchesIngredient(flour,'Weizen 405'));
  assert.ok(matchesIngredient(flour,'wheat 405'));
  assert.ok(!matchesIngredient(flour,'rye 405'));
  assert.ok(ingredientForm(flour,'de').length > 0);
  const custom = {id:'wheat-flour-type-405',name:'Custom English ingredient',form:'custom form',aliases:[]};
  assert.equal(ingredientName(custom,'de'), custom.name); // No demo hardcoding by a colliding ID.
  assert.equal(ingredientForm(custom,'de'), custom.form);
});

test('every ingredient has a German display name and keeps untranslated source content', () => {
  assert.ok(catalogue.ingredients.length >= 300);
  for (const ingredient of catalogue.ingredients) {
    assert.ok(ingredient.translations.de.name.length > 0, ingredient.id);
    assert.equal(ingredientName(ingredient,'en'), ingredient.name);
    assert.equal(ingredientName(ingredient,'fr'), ingredient.name);
    assert.ok(ingredient.uncertainty.length > 0);
  }
});
