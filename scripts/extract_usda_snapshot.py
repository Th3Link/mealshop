"""Extract reviewed SR Legacy records; preserve numeric JSON tokens as strings in the snapshot."""
import argparse
import hashlib
import json
from pathlib import Path
from zipfile import ZipFile
from import_catalogue import ROOT

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("--output", type=Path, default=ROOT / "data/sources/usda-selected.json")
    args = parser.parse_args()
    selected = {r["record_id"] for r in json.loads((ROOT / "data/catalogue-selection.json").read_text())["entries"] if r["provider"] == "usda"}
    with ZipFile(args.archive) as archive:
        with archive.open("FoodData_Central_sr_legacy_food_json_2018-04.json") as stream:
            data = json.load(stream, parse_float=str)
    records = [r for r in data["SRLegacyFoods"] if str(r["fdcId"]) in selected]
    if {str(r["fdcId"]) for r in records} != selected:
        raise ValueError("Some reviewed USDA source identities were not present in the archive")
    output = {"dataset": "USDA SR Legacy", "release": "2018-04",
        "download_url": "https://fdc.nal.usda.gov/fdc-datasets/FoodData_Central_sr_legacy_food_json_2018-04.zip",
        "archive_sha256": hashlib.sha256(args.archive.read_bytes()).hexdigest(), "license": "CC0-1.0",
        "retrieved_on": "2026-11-07", "extraction": "JSON fractional-number token spellings retained as strings; full selected records, no nutrient inference.",
        "records": records}
    args.output.write_text(json.dumps(output, indent=2, ensure_ascii=False) + "\n")
    print(f"Extracted {len(records)} USDA records with preserved numeric tokens.")
