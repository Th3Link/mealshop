"""Independently check catalogue identities, units and amounts against frozen primary records."""
from decimal import Decimal, InvalidOperation
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
USDA = {
    "energy_kcal": (1008, "kcal"), "energy_kj": (1062, "kJ"), "protein": (1003, "g"),
    "fat": (1004, "g"), "carbohydrate": (1005, "g"), "fibre": (1079, "g"),
    "calcium": (1087, "mg"), "iron": (1089, "mg"), "magnesium": (1090, "mg"),
    "potassium": (1092, "mg"), "sodium": (1093, "mg"), "vitamin_c": (1162, "mg"),
    "folate": (1177, "µg"), "food_folate": (1187, "µg"),
}
BLS = {
    "energy_kcal": ("ENERCC", "kcal"), "energy_kj": ("ENERCJ", "kJ"),
    "protein": ("PROT625", "g"), "fat": ("FAT", "g"),
    "available_carbohydrate": ("CHO", "g"), "fibre": ("FIBT", "g"),
    "calcium": ("CA", "mg"), "iron": ("FE", "mg"), "magnesium": ("MG", "mg"),
    "potassium": ("K", "mg"), "sodium": ("NA", "mg"), "vitamin_c": ("VITC", "mg"),
    "food_folate": ("FOLFD", "µg"), "soluble_fibre": ("FIBSOL", "g"),
}


def load(path):
    return json.loads(path.read_text(), parse_float=Decimal)


def numeric(value):
    try:
        result = Decimal(str(value))
        return result if result.is_finite() and result >= 0 else None
    except InvalidOperation:
        return None


catalogue = load(ROOT / "data/catalogue.json")
selection = load(ROOT / "data/catalogue-selection.json")["entries"]
assert catalogue["language"] == "en"
assert len(catalogue["ingredients"]) == len(selection) + 3
assert len({i["id"] for i in catalogue["ingredients"]}) == len(catalogue["ingredients"])
bls_snapshot = load(ROOT / "data/sources/bls-4.0-selected.json")
bls = {r["BLS Code"]: r for r in bls_snapshot["records"]}
usda = {str(r["fdcId"]): r for r in load(ROOT / "data/sources/usda-selected.json")["records"]}
for id in ("oats", "chia"):
    raw = load(ROOT / "data/sources" / f"usda-{id}.json")
    usda[str(raw["fdcId"])] = raw
manifest = {i["id"]: i for i in selection}
values = profiles = identities = 0
for record in catalogue["ingredients"]:
    source = record["source"]
    assert record["translations"]["de"]["name"]
    assert record["category"].isascii() and record["category"].islower()
    assert record["basis"] == "per_100g_edible_portion"
    if source["name"].startswith("Max Rubner-Institut"):
        assert source["license"] == "CC-BY-4.0"
        raw = bls[source["record_id"]]
        assert record["name"] == raw["Food name"], record["id"]
        assert record["translations"]["de"]["name"] == raw["Lebensmittelbezeichnung"]
        assert manifest[record["id"]]["record_id"] == source["record_id"]
        assert "carbohydrate" not in record["nutrients"]  # Do not confuse CHO with USDA by-difference.
        assert "folate" not in record["nutrients"]       # FOLFD is not total folate/DFE.
        expected = {}
        for name, (code, unit) in BLS.items():
            column = next(k for k in raw if k.startswith(code + " ") and "[" in k)
            assert f"[{unit}/100g]" in column
            expected[name] = numeric(raw[column])
    elif source["name"] == "USDA FoodData Central":
        assert source["license"] == "CC0-1.0"
        raw = usda[source["record_id"]]
        assert raw["dataType"] == "SR Legacy"
        assert source["version"] == "SR Legacy / 2019-04-01"
        assert "available_carbohydrate" not in record["nutrients"]
        source_values = {r["nutrient"]["id"]: r for r in raw["foodNutrients"]}
        expected = {}
        for name, (source_id, unit) in USDA.items():
            if source_id in source_values:
                reading = source_values[source_id]
                assert reading["nutrient"]["unitName"] == unit
                expected[name] = numeric(reading.get("amount"))
    else:
        assert source["name"] == "Mealshop identity-only record"
        assert all(r["amount"] is None for r in record["nutrients"].values())
        identities += 1
        continue
    profiles += 1
    for name, original in expected.items():
        amount = record["nutrients"].get(name, {}).get("amount")
        if original is None:
            assert amount is None, (record["id"], name)
        elif amount is not None:
            actual = Decimal(amount)
            assert abs(actual - original) <= Decimal("0.0000005"), (record["id"], name, original, actual)
            assert actual > 0 or original == 0  # Never turn tiny positive data into a claimed zero.
            values += 1
        else:
            assert original < Decimal("0.0000005"), (record["id"], name, "missing mapped value")
    for name, reading in record["nutrients"].items():
        if name not in expected:
            assert reading["amount"] is None, (record["id"], name, "unsourced value")
print(f"Verified {len(catalogue['ingredients'])} English-base ingredients, {profiles} source profiles, {identities} identity-only records and {values} mapped nutrient values.")

from verify_ingredient_groups import verify as verify_groups
verify_groups()
