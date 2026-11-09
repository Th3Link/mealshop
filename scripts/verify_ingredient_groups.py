"""Verify organizational coverage without inventing ingredient composition or source values."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def verify():
    catalogue = json.loads((ROOT / 'data/catalogue.json').read_text())
    mapping = json.loads((ROOT / 'data/ingredient-groups.json').read_text())
    assert set(mapping) == {'version', 'groups'} and mapping['version'] == 'kitchen-organization-1'
    expected = {item['id'] for item in catalogue['ingredients']}
    seen, groups = set(), set()
    for group in mapping['groups']:
        assert set(group) == {'id', 'name', 'translations', 'ingredient_ids'}
        assert group['id'].startswith('custom-group-') and group['id'] not in groups
        assert group['id'] not in expected and len(group['id']) <= 100
        assert group['name'].strip() and group['translations']['de']['name'].strip()
        assert group['ingredient_ids']
        groups.add(group['id'])
        for identity in group['ingredient_ids']:
            assert identity in expected and identity not in seen, identity
            seen.add(identity)
    assert seen == expected, expected - seen
    for family in ['wheat', 'rye', 'spelt']:
        group = next(g for g in mapping['groups'] if g['id'] == f'custom-group-{family}-flour')
        assert set(group['ingredient_ids']) == {i for i in expected if i.startswith(f'{family}-flour-type-') or i == f'{family}-wholemeal-flour'}
    print(f'Verified {len(seen)} exact ingredient IDs in {len(groups)} bilingual organizational groups; no nutrient profiles supplied.')

if __name__ == '__main__':
    verify()
