"""Verify selected evidence directly against operator-supplied original public archives (no network)."""
import argparse
import hashlib
import io
import json
from pathlib import Path
from zipfile import ZipFile
from extract_bls_snapshot import rows


def verify(bls_path, usda_path):
    evidence = json.loads(Path('data/sources/nutrition-evidence-v1.json').read_text())
    selected = {r['id']: r for r in evidence['records']}
    for kind, path in [('bls', bls_path), ('usda', usda_path)]:
        assert hashlib.sha256(path.read_bytes()).hexdigest() == evidence['archives'][kind]['archive_sha256']
    checked = 0
    with ZipFile(bls_path) as archive:
        data = archive.read(next(n for n in archive.namelist() if n.endswith('BLS_4_0_Daten_2025_DE.xlsx')))
        assert hashlib.sha256(data).hexdigest() == evidence['archives']['bls']['workbook_sha256']
        iterator = rows(data)
        _, header = next(iterator)
        for _, values in iterator:
            record = selected.get('bls:' + values.get('A', ''))
            if not record:
                continue
            cells = {header[k]: v for k, v in values.items() if k in header}
            assert record['name'] == cells['Food name']
            for field in record['fields'].values():
                code = field['column'].split(' ')[0]
                assert field['raw'] == cells[field['column']]
                assert field['method'] == cells[code + ' Datenherkunft']
                assert field['reference'] == cells[code + ' Referenz']
                checked += 1
    decoder = json.JSONDecoder(parse_float=str)
    seen = set()
    with ZipFile(usda_path) as archive:
        with io.TextIOWrapper(archive.open(next(n for n in archive.namelist() if n.endswith('.json'))), encoding='utf8') as stream:
            buffer = stream.read(131072)
            buffer = buffer[buffer.index('[', buffer.index('"SRLegacyFoods"')) + 1:]
            while True:
                buffer = buffer.lstrip(' ,\r\n\t')
                if buffer.startswith(']'):
                    break
                try:
                    raw, end = decoder.raw_decode(buffer)
                except json.JSONDecodeError:
                    chunk = stream.read(131072)
                    if not chunk:
                        raise
                    buffer += chunk
                    continue
                buffer = buffer[end:]
                record = selected.get('usda:' + str(raw['fdcId']))
                if not record:
                    continue
                seen.add(record['id'])
                assert record['name'] == raw['description']
                nutrients = {n['nutrient']['id']: n for n in raw['foodNutrients']}
                for field in record['fields'].values():
                    cell = nutrients[field['source_nutrient_id']]
                    assert field['raw'] == str(cell['amount'])
                    assert field['unit'] == cell['nutrient']['unitName']
                    assert field['data_points'] == cell.get('dataPoints')
                    assert field['derivation'] == cell.get('foodNutrientDerivation')
                    checked += 1
    assert seen == {id for id in selected if id.startswith('usda:')}
    print(f'Verified both original archive hashes and {checked} selected raw numeric/missing facts with acquisition metadata.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bls', type=Path, required=True)
    parser.add_argument('--usda', type=Path, required=True)
    args = parser.parse_args()
    verify(args.bls, args.usda)
