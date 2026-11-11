"""Independently verify frozen nutrition comparisons, original precedence and full core coverage."""
from decimal import Decimal, InvalidOperation
import json
import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

BLS_CODES = dict(zip(['energy_kcal','energy_kj','protein','fat','available_carbohydrate','fibre','calcium','iron','magnesium','potassium','sodium','vitamin_c','food_folate','total_sugars','saturated_fat','salt'], ['ENERCC','ENERCJ','PROT625','FAT','CHO','FIBT','CA','FE','MG','K','NA','VITC','FOLFD','SUGAR','FASAT','NACL']))
USDA_CODES = dict(zip(['energy_kcal','energy_kj','protein','fat','carbohydrate','fibre','calcium','iron','magnesium','potassium','sodium','vitamin_c','folate','food_folate','total_sugars','saturated_fat'], [1008,1062,1003,1004,1005,1079,1087,1089,1090,1092,1093,1162,1177,1187,2000,1258]))

def load(name):
    return json.loads((ROOT / name).read_text())

def verify():
    # Published v1 remains reproducible for legacy recipe analysis; release new evidence as v2.
    assert hashlib.sha256((ROOT / 'data/nutrition-estimates-v1.json').read_bytes()).hexdigest() == 'c43c10bab7fe3e4a8954e01727418276bd4c32e4b25e62ada2d1757f30f97ef6', 'Published v1 is immutable; add a new evidence release instead of rewriting history'
    assert hashlib.sha256((ROOT / 'data/sources/nutrition-evidence-v1.json').read_bytes()).hexdigest() == '848f7a14413802c07033648fde08e02330c8a3ce7dd6372ac0c812fd82810946', 'Published v1 is immutable; add a new evidence release instead of rewriting history'
    evidence = load('data/sources/nutrition-evidence-v1.json')
    runtime = load('data/nutrition-estimates-v1.json')
    foods = {i['id']: i for i in load('data/catalogue.json')['ingredients']}
    groups = load('data/ingredient-groups.json')
    records = {r['id']: r for r in evidence['records']}
    assert len(records) == len(evidence['records'])
    assert evidence['basis'] == 'per_100g_edible_portion'
    assert runtime['version'] == evidence['version'] == 'nutrition-evidence-v1'
    for key, record in records.items():
        for nutrient, field in record['fields'].items():
            if key.startswith('bls:'): assert field['column'].split(' ')[0] == BLS_CODES[nutrient]
            if key.startswith('usda:'): assert field['source_nutrient_id'] == USDA_CODES[nutrient]
    selections = {(s['ingredient_id'], s['method']): s for s in evidence['selections']}
    assert len(selections) == len(evidence['selections'])
    group_ids = {g['id']: g for g in groups['groups']}
    assert set(runtime['profiles']) == set(foods) | set(group_ids)
    values = comparisons = 0
    for id, entry in runtime['profiles'].items():
        item = foods.get(id)
        if item:
            assert entry['source_name'] == item['source']['name']
            assert entry['source_record_id'] == item['source']['record_id']
            assert entry['source_version'] == item['source']['version']
            assert entry['form'] == item['form']
        else:
            assert entry['source_name'] == 'Catalogue organization'
            assert entry['members'] == group_ids[id]['ingredient_ids']
            assert entry['source_version'] == groups['version']
            assert entry['form'] == 'Ingredient group'
        covered = {n for n, v in item['nutrients'].items() if v['amount'] is not None} if item else set()
        for profile in entry['profiles']:
            assert profile['version'] == evidence['version'] and profile['rationale'].strip()
            refs = profile['references']
            assert 1 <= len(refs) <= 64 and len({r['id'] for r in refs}) == len(refs)
            assert not (covered & set(profile['values'])), (id, 'overrides original/earlier values')
            if profile['method'] != 'group_envelope':
                selection = selections[id, profile['method']]
                assert set(r['id'] for r in refs) == set(selection['references'])
                assert selection['rationale'] == profile['rationale']
                assert len(refs) == 1 if profile['method'] == 'source_supplement' else len(refs) >= 2
                comparisons += profile['method'] == 'comparison_envelope'
            else:
                allowed = set()
                for member in group_ids[id]['ingredient_ids']:
                    allowed.add('reported:' + member)
                    for p in runtime['profiles'][member]['profiles']:
                        allowed.update(r['id'] for r in p['references'])
                assert {r['id'] for r in refs} <= allowed
            for ref in refs:
                source = ref['source']
                assert source['kind'] == 'primary' and source['url'].startswith('https://')
                assert all(source[k].strip() for k in ('name','record_id','version','license','retrieved_on'))
                if profile['method'] == 'source_supplement':
                    assert all(source[k] == item['source'][k] for k in ('name','record_id','version'))
                for nutrient, b in ref['values'].items():
                    assert nutrient not in ('beta_glucan', 'soluble_fibre')
                    low, high = Decimal(b['lower']), Decimal(b['upper'])
                    assert 0 <= low <= high and low.is_finite() and high.is_finite()
                    if ref['id'] in records:
                        cell = records[ref['id']]['fields'][nutrient]
                        assert ref['source'] == records[ref['id']]['source']
                        raw = cell['raw'].strip()
                        limit = raw.startswith('<')
                        original = Decimal(raw[1:] if limit else raw)
                        assert abs(high - original) < Decimal('0.000001') if limit else abs(high - original) <= Decimal('0.0000005')
                        if limit: assert high >= original
                        assert low == 0 if limit else low == high
                        assert high > 0 or original == 0
                    else:
                        original = foods[ref['id'].removeprefix('reported:')]
                        assert low == high == Decimal(original['nutrients'][nutrient]['amount'])
                        assert source['record_id'] == original['source']['record_id']
            for nutrient, bounds in profile['values'].items():
                samples = [r['values'][nutrient] for r in refs if nutrient in r['values']]
                assert samples
                assert Decimal(bounds['lower']) == min(Decimal(v['lower']) for v in samples)
                assert Decimal(bounds['upper']) == max(Decimal(v['upper']) for v in samples)
                values += 1
            covered.update(profile['values'])
        assert {'energy_kcal','protein','fat'} <= covered and {'carbohydrate','available_carbohydrate'} & covered, id
    # Independently recompute every group bound from effective members, not the selected extrema.
    for group in groups['groups']:
        for nutrient, span in runtime['profiles'][group['id']]['profiles'][0]['values'].items():
            lower, upper = [], []
            for member in group['ingredient_ids']:
                amount = foods[member]['nutrients'].get(nutrient, {}).get('amount')
                if amount is not None:
                    lower.append(Decimal(amount)); upper.append(Decimal(amount))
                    continue
                for profile in runtime['profiles'][member]['profiles']:
                    if nutrient in profile['values']:
                        value = profile['values'][nutrient]
                        lower.append(Decimal(value['lower'])); upper.append(Decimal(value['upper']))
                        break
            assert Decimal(span['lower']) == min(lower), (group['id'], nutrient, 'lower')
            assert Decimal(span['upper']) == max(upper), (group['id'], nutrient, 'upper')
    # Neither estimates nor a documented zero may mutate allergy/composition data or source facts.
    assert all('beta_glucan' not in p['values'] and 'soluble_fibre' not in p['values']
               for e in runtime['profiles'].values() for p in e['profiles'])
    print(f'Verified {len(foods)} foods and {len(group_ids)} groups: complete core coverage; {comparisons} comparison profiles, {values} independently checked envelopes; original readings unchanged.')

if __name__ == '__main__':
    verify()
