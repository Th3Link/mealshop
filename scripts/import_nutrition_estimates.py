"""Reproduce versioned nutrition evidence offline; never rewrite original source readings."""
from copy import deepcopy
from decimal import Decimal, InvalidOperation, ROUND_HALF_UP, ROUND_CEILING
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
INPUT = ROOT / 'data/sources/nutrition-evidence-v1.json'
OUTPUT = ROOT / 'data/nutrition-estimates-v1.json'
CORE = ('energy_kcal', 'energy_kj', 'protein', 'fat', 'available_carbohydrate', 'carbohydrate')
GROUP_FIELDS = (*CORE, 'fibre', 'total_sugars', 'saturated_fat', 'salt')
UNITS = {**{k: 'g' for k in GROUP_FIELDS}, 'energy_kcal': 'kcal', 'energy_kj': 'kJ',
         **{k: 'mg' for k in ('calcium', 'iron', 'magnesium', 'potassium', 'sodium', 'vitamin_c')},
         'folate': 'µg', 'food_folate': 'µg'}


def bounds(cell, nutrient):
    if nutrient not in UNITS or cell['unit'].lower() != UNITS[nutrient].lower():
        raise ValueError(f'Unexpected source unit for {nutrient}')
    raw = cell['raw'].strip()
    try:
        threshold = raw.startswith('<') and not raw.startswith('<=')
        value = Decimal(raw[1:].strip() if threshold else raw)
    except InvalidOperation:
        return None  # Unknown/LOD without a numeric limit is never zero.
    if not value.is_finite() or value < 0:
        raise ValueError('Invalid source amount')
    normalized = value.quantize(Decimal('0.000001'), rounding=ROUND_CEILING if threshold else ROUND_HALF_UP)
    if value > 0 and normalized == 0:
        return None
    number = format(normalized.normalize(), 'f')
    return {'lower': '0' if threshold else number, 'upper': number}


def envelope(refs, fields):
    result = {}
    for key in fields:
        available = [r['values'][key] for r in refs if key in r['values']]
        if available:
            result[key] = {'lower': min((v['lower'] for v in available), key=Decimal),
                           'upper': max((v['upper'] for v in available), key=Decimal)}
    return result


def generate():
    evidence = json.loads(INPUT.read_text())
    catalogue = json.loads((ROOT / 'data/catalogue.json').read_text())
    foods = {i['id']: i for i in catalogue['ingredients']}
    records = {r['id']: r for r in evidence['records']}
    if len(records) != len(evidence['records']):
        raise ValueError('Duplicate evidence identity')
    refs = {}
    for key, record in records.items():
        values = {n: b for n, c in record['fields'].items() if (b := bounds(c, n)) is not None}
        refs[key] = {k: deepcopy(record[k]) for k in ('id', 'name', 'form', 'source')}
        refs[key]['values'] = values
    output = {'version': evidence['version'], 'profiles': {}}
    effective = {id: {n: {'lower': v['amount'], 'upper': v['amount']}
                      for n, v in item['nutrients'].items() if v['amount'] is not None}
                 for id, item in foods.items()}
    for selection in evidence['selections']:
        id = selection['ingredient_id']
        item = foods[id]
        candidates = [deepcopy(refs[k]) for k in selection['references']]
        if len(set(selection['references'])) != len(candidates):
            raise ValueError('Duplicate comparison record')
        fields = set(selection.get('nutrients', UNITS)) - set(effective[id])
        values = envelope(candidates, fields)
        if not values:
            continue
        for ref in candidates:
            ref['values'] = {n: v for n, v in ref['values'].items() if n in values}
        profile = {'version': evidence['version'], 'method': selection['method'],
                   'rationale': selection['rationale'], 'references': candidates, 'values': values}
        entry = output['profiles'].setdefault(id, {'source_name': item['source']['name'], 'source_record_id': item['source']['record_id'],
            'source_version': item['source']['version'], 'form': item['form'], 'profiles': []})
        entry['profiles'].append(profile)
        effective[id].update(values)
    groups = json.loads((ROOT / 'data/ingredient-groups.json').read_text())
    for group in groups['groups']:
        candidates = {}
        for id in group['ingredient_ids']:
            item = foods[id]
            values = {n: {'lower': v['amount'], 'upper': v['amount']}
                      for n, v in item['nutrients'].items() if v['amount'] is not None and n in GROUP_FIELDS}
            if values:
                candidates['reported:' + id] = {'id': 'reported:' + id, 'name': item['name'],
                    'form': item['form'], 'source': {'kind': 'primary', **item['source']}, 'values': values}
            for profile in output['profiles'].get(id, {}).get('profiles', []):
                for ref in profile['references']:
                    existing = candidates.setdefault(ref['id'], deepcopy(ref))
                    for nutrient, value in ref['values'].items():
                        if nutrient in existing['values'] and existing['values'][nutrient] != value:
                            raise ValueError('Conflicting facts for the identical primary source record')
                        existing['values'][nutrient] = deepcopy(value)
        all_refs = list(candidates.values())
        values = envelope(all_refs, GROUP_FIELDS)
        # Keep every extremum, not repeated interior records; this preserves the exact envelope.
        selected = set()
        for n, v in values.items():
            for edge in ('lower', 'upper'):
                selected.add(next(r['id'] for r in all_refs if r['values'].get(n, {}).get(edge) == v[edge]))
        chosen = [r for r in all_refs if r['id'] in selected]
        for ref in chosen:
            ref['values'] = {n: v for n, v in ref['values'].items() if n in values}
        if values:
            output['profiles'][group['id']] = {'source_name': 'Catalogue organization', 'source_record_id': group['id'],
                'members': group['ingredient_ids'],
                'source_version': groups['version'], 'form': 'Ingredient group', 'profiles': [{
                    'version': evidence['version'], 'method': 'group_envelope',
                    'rationale': 'Observed envelope of explicitly assigned group members and their frozen evidence. The unspecified subtype/state is unverified; select a subtype for more relevant values. No constituents or allergy safety are inferred.',
                    'references': chosen, 'values': values}]}
    for id, values in effective.items():
        if not all(n in values for n in ('energy_kcal', 'protein', 'fat')) or not any(n in values for n in ('available_carbohydrate', 'carbohydrate')):
            raise ValueError(f'Missing core coverage: {id}')
    return output


if __name__ == '__main__':
    generated = json.dumps(generate(), ensure_ascii=False, sort_keys=True, separators=(',', ':')) + '\n'
    if '--check' in sys.argv:
        if not OUTPUT.exists() or OUTPUT.read_text() != generated:
            raise SystemExit('Nutrition evidence is stale; run scripts/import_nutrition_estimates.py')
    else:
        OUTPUT.write_text(generated)
    print(f'Nutrition evidence reproducible: {len(generate()["profiles"])} profiled food/group identities; {len(generated.encode())} bytes')
