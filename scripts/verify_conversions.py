"""Check every curated reference portion against its frozen USDA source."""
import json
from decimal import Decimal
from pathlib import Path
root=Path(__file__).resolve().parents[1]
source=json.loads((root/'data/sources/usda-selected.json').read_text())
foods={str(f['fdcId']):f for f in source['records']}
catalogue={i['id']:i for i in json.loads((root/'data/catalogue.json').read_text())['ingredients']}
entries=json.loads((root/'data/conversions.json').read_text())['entries']
assert len({e['id'] for e in entries})==len(entries)
for entry in entries:
    e=entry['evidence']; food=foods[e['source_record_id']]; item=catalogue[e['ingredient_id']]
    portion=next(p for p in food['foodPortions'] if p['id']==entry['portion']['id'])
    assert portion==entry['portion']
    assert item['source']['record_id']==e['source_record_id'] and item['form']==e['form']
    assert e['input_unit']==entry['unit']
    assert portion['modifier']==entry['unit'] or (entry['unit']=='piece' and portion['modifier'] in ['packet','block','cake (0.6 oz)'])
    assert Decimal(e['grams_per_base_unit'])*Decimal(str(portion['amount']))==Decimal(str(portion['gramWeight']))
print(f'Verified {len(entries)} exact USDA reference portions; no inferred volume densities.')
