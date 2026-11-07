"""Validate positive-only allergen selections against exact catalogue identities."""
import json
from pathlib import Path
root=Path(__file__).resolve().parents[1]
data=json.loads((root/'data/allergens.json').read_text())
foods={i['id']:i for i in json.loads((root/'data/catalogue.json').read_text())['ingredients']}
groups={g['id'] for g in data['groups']}
assert len(groups)==14
seen=set()
for entry in data['entries']:
    food=foods[entry['ingredient_id']]
    assert entry['form']==food['form'] and entry['source_record_id']==food['source']['record_id']
    assert entry['allergen_id'] in groups and entry['source_url']==data['source_url']
    key=(entry['ingredient_id'],entry['allergen_id']);assert key not in seen;seen.add(key)
    assert 'no absence' in entry['basis']
print(f'Validated {len(seen)} positive identity classifications; omissions remain unknown.')
