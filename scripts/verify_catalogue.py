"""Verify the curated starter values against frozen, primary USDA API responses."""
from decimal import Decimal
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAPPING = {
    1008: ("energy_kcal", "kcal"), 1062: ("energy_kj", "kJ"),
    1003: ("protein", "g"), 1004: ("fat", "g"), 1005: ("carbohydrate", "g"),
    1079: ("fibre", "g"), 1087: ("calcium", "mg"), 1089: ("iron", "mg"),
    1090: ("magnesium", "mg"), 1092: ("potassium", "mg"), 1093: ("sodium", "mg"),
    1162: ("vitamin_c", "mg"), 1177: ("folate", "µg"),
}

def load(path):
    return json.loads(path.read_text(), parse_float=Decimal)

catalogue = load(ROOT / "data/catalogue.json")
for record in catalogue["ingredients"]:
    if record["id"] not in ("oats", "chia"):
        assert all(value["amount"] is None for value in record["nutrients"].values())
        continue
    source = load(ROOT / "data/sources" / f"usda-{record['id']}.json")
    assert str(source["fdcId"]) == record["source"]["record_id"]
    assert source["dataType"] == "SR Legacy"
    assert source["publicationDate"] == "4/1/2019"
    source_values = {n["nutrient"]["id"]: n for n in source["foodNutrients"]}
    for source_id, (name, unit) in MAPPING.items():
        reading = source_values[source_id]
        assert reading["nutrient"]["unitName"] == unit
        assert Decimal(record["nutrients"][name]["amount"]) == reading["amount"], (record["id"], name)
    for name in ("soluble_fibre", "beta_glucan"):
        assert record["nutrients"][name]["amount"] is None
print("Starter catalogue: 26 nutrient values match frozen USDA sources; unmeasured components remain unknown.")
