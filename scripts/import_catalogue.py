"""Build the English-base kitchen catalogue from reviewed, frozen source selections."""
from decimal import Decimal, InvalidOperation, ROUND_HALF_UP
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
BLS_MAPPING = {
    "ENERCJ": ("energy_kj", "kJ"), "ENERCC": ("energy_kcal", "kcal"),
    "PROT625": ("protein", "g"), "FAT": ("fat", "g"),
    "CHO": ("available_carbohydrate", "g"), "FIBT": ("fibre", "g"),
    "CA": ("calcium", "mg"), "FE": ("iron", "mg"), "MG": ("magnesium", "mg"),
    "K": ("potassium", "mg"), "NA": ("sodium", "mg"), "VITC": ("vitamin_c", "mg"),
    "FOLFD": ("food_folate", "µg"), "FIBSOL": ("soluble_fibre", "g"),
}
USDA_MAPPING = {
    1008: ("energy_kcal", "kcal"), 1062: ("energy_kj", "kJ"),
    1003: ("protein", "g"), 1004: ("fat", "g"), 1005: ("carbohydrate", "g"),
    1079: ("fibre", "g"), 1087: ("calcium", "mg"), 1089: ("iron", "mg"),
    1090: ("magnesium", "mg"), 1092: ("potassium", "mg"), 1093: ("sodium", "mg"),
    1162: ("vitamin_c", "mg"), 1177: ("folate", "µg"), 1187: ("food_folate", "µg"),
}
ACQUISITION = {
    "Analyse": "Analysis", "Literatur": "Literature", "Nährstoffdatenbank": "Nutrient database",
    "Formelberechnung": "Formula calculation", "Rezeptberechnung": "Recipe calculation",
    "Aggregation": "Aggregation", "Schätzung": "Estimate", "Logische Null": "Logical zero",
    "Berechnung": "Calculation",
}
DE_ALIASES = {
    "drinking-water": ["Wasser", "Leitungswasser"],
    "oat-drink-unsweetened": ["Hafermilch", "Haferdrink"],
    "oat-drink-unsweetened-fortified": ["Hafermilch angereichert"],
    "almond-drink-unsweetened": ["Mandelmilch", "Mandeldrink"],
    "almond-drink-unsweetened-fortified": ["Mandelmilch angereichert"],
    "soy-drink-unsweetened": ["Sojamilch", "Sojadrink"],
    "soy-drink-unsweetened-fortified": ["Sojamilch angereichert"],
    "rice-drink-unsweetened": ["Reismilch", "Reisdrink"],
    "quark-low-fat": ["Magerquark", "Quark mager"],
    "quark-20-fat-dry-matter": ["Speisequark", "Speisequark 20 %"],
    "quark-40-fat-dry-matter": ["Sahnequark", "Speisequark 40 %"],
    "zucchini-raw": ["Zucchini", "Zucchine"],
    "pepper-red-raw": ["Paprika rot", "Rote Paprika"],
    "pepper-green-raw": ["Paprika grün", "Grüne Paprika"],
    "pepper-yellow-raw": ["Paprika gelb", "Gelbe Paprika"],
    "beetroot-raw": ["Rote Beete", "Rote Bete"],
    "beetroot-cooked": ["Rote Beete gekocht", "Rote Bete gekocht"],
    "spring-onion-raw": ["Frühlingszwiebel", "Frühlingszwiebeln", "Lauchzwiebel"],
    "chicken-breast-fillet-raw": ["Hähnchenbrust", "Hähnchenbrustfilet", "Hähnchenfleisch"],
    "cottage-cheese-low-fat": ["Hüttenkäse", "Körniger Frischkäse"],
    "sour-milk-cheese-low-fat": ["Harzer Käse", "Handkäse"],
    "xylitol-powder": ["Xylit", "Xylitol", "Birkenzucker"],
    "margarine-full-fat-fortified": ["Margarine", "Magarine"],
    "soy-protein-isolate": ["Sojaprotein", "Sojaproteinisolat", "Proteinpulver Soja"],
    "tomato-passata": ["Passierte Tomaten", "Tomatenpüree", "Passata"],
    "pasta-egg-free-dry": ["Nudeln eifrei", "Pasta eifrei"],
    "pasta-wholemeal-egg-free-dry": ["Vollkornnudeln", "Vollkornpasta"],
}
EN_ALIASES = {"drinking-water": ["Water", "Tap water"], "quark-low-fat": ["Low-fat quark"],
              "zucchini-raw": ["Courgette"], "beetroot-raw": ["Beet"],
              "tomato-passata": ["Passata", "Sieved tomatoes"], "soy-protein-isolate": ["Soy protein isolate"]}


def load(path):
    return json.loads(path.read_text(), parse_float=Decimal)


def decimal_amount(value):
    """Keep numeric source facts; do not replace missing/below-resolution values with zero."""
    try:
        original = Decimal(str(value))
    except InvalidOperation:
        return None
    if not original.is_finite() or original < 0:
        return None
    rounded = original.quantize(Decimal("0.000001"), rounding=ROUND_HALF_UP)
    if original > 0 and rounded == 0:
        return None
    return format(rounded.normalize(), "f")


def form_text(name, category):
    low = name.lower()
    if "raw" in low:
        return "raw, edible portion", "roh, essbarer Anteil"
    if any(word in low for word in ["boiled", "cooked", "roasted", "baked"]):
        return "prepared state as named by the source, edible portion", "zubereiteter Zustand wie von der Quelle bezeichnet, essbarer Anteil"
    if "fresh" in low:
        return "fresh, edible portion", "frisch, essbarer Anteil"
    if "frozen" in low:
        return "frozen, edible portion", "tiefgefroren, essbarer Anteil"
    if "dried" in low or category in ["grains_flour", "pasta", "herbs_spices"]:
        return "dry/as sold unless another state is named by the source", "trocken/wie verkauft, sofern die Quelle keinen anderen Zustand nennt"
    return "as sold, edible portion; preserve the source-specific variant", "wie verkauft, essbarer Anteil; genaue Quellenvariante beachten"


def build():
    base = load(ROOT / "data/catalogue-base.json")
    bls = load(ROOT / "data/sources/bls-4.0-selected.json")
    bls_rows = {r["BLS Code"]: r for r in bls["records"]}
    usda = load(ROOT / "data/sources/usda-selected.json")
    usda_rows = {str(r["fdcId"]): r for r in usda["records"]}
    components = {r["Nährstoffcode / Component code"]: r for r in bls["components"]}
    entries = load(ROOT / "data/catalogue-selection.json")["entries"]
    records = base["ingredients"]
    for record in records:
        if record["id"] in ("oats", "chia"):
            raw = load(ROOT / "data/sources" / f"usda-{record['id']}.json")
            folate = next(n for n in raw["foodNutrients"] if n["nutrient"]["id"] == 1187)
            record["nutrients"]["food_folate"] = {"amount": decimal_amount(folate["amount"]), "uncertainty": "USDA food folate, distinct from total folate and DFE; generic source composition."}
    for selected in entries:
        id = selected["id"]
        category = selected["category"]
        provider = selected["provider"]
        nutrients = {}
        if provider == "bls":
            raw = bls_rows[selected["record_id"]]
            name = raw["Food name"]
            translated = {"name": raw["Lebensmittelbezeichnung"], "aliases": DE_ALIASES.get(id, [])}
            source = {"name": "Max Rubner-Institut · Bundeslebensmittelschlüssel", "url": "https://blsdb.de/download",
                      "record_id": selected["record_id"], "version": "BLS 4.0 / 2025", "retrieved_on": "2026-11-07", "license": "CC-BY-4.0"}
            uncertainty = "BLS reference composition for the named state, not a measurement of your product. Available carbohydrate excludes fibre; fat-in-dry-matter grades are not grams of fat per 100 g."
            for code, (nutrient, unit) in BLS_MAPPING.items():
                assert components[code]["Einheit / Unit"] == unit
                column = next(k for k in raw if k.startswith(code + " ") and "[" in k)
                assert f"[{unit}/100g]" in column, column
                acquisition = ACQUISITION.get(raw.get(code + " Datenherkunft"), "Source method recorded in snapshot")
                amount = decimal_amount(raw[column])
                detail = f"BLS: {acquisition}. Original value/method/reference retained in source snapshot."
                if amount is None:
                    detail = "Not reported or not representable at six decimal places; original retained in BLS snapshot."
                elif Decimal(amount) != Decimal(raw[column]):
                    detail += " Rounded to six decimals on import."
                nutrients[nutrient] = {"amount": amount, "uncertainty": detail}
        elif provider == "usda":
            raw = usda_rows[selected["record_id"]]
            name = raw["description"]
            translated = selected["de"].copy()
            source = {"name": "USDA FoodData Central", "url": f"https://fdc.nal.usda.gov/food-details/{raw['fdcId']}/nutrients",
                      "record_id": str(raw["fdcId"]), "version": "SR Legacy / 2019-04-01", "retrieved_on": "2026-11-07", "license": "CC0-1.0"}
            uncertainty = "USDA SR Legacy reference composition, not your specific product. Carbohydrate by difference includes fibre. Preparation and formulation must match the source."
            if "whey" in id:
                name += " (USDA reference formulation)"
                uncertainty += " Formulated reference powder, not a universal unflavoured/pure whey isolate specification; brands can differ substantially."
            for reading in raw["foodNutrients"]:
                mapped = USDA_MAPPING.get(reading["nutrient"]["id"])
                if not mapped:
                    continue
                nutrient, unit = mapped
                assert reading["nutrient"]["unitName"] == unit
                nutrients[nutrient] = {"amount": decimal_amount(reading.get("amount")),
                    "uncertainty": (f"USDA reference; {reading['dataPoints']} reported data points. No confidence interval supplied." if "dataPoints" in reading else "USDA reference; sample count and confidence interval not reported.")}
        else:
            name = selected["name"]
            translated = selected["de"].copy()
            source = {"name": "Mealshop identity-only record", "url": "https://github.com/Th3Link/mealshop/blob/main/data/README.md",
                      "record_id": id, "version": "2026-11-07", "retrieved_on": "2026-11-07", "license": "AGPL-3.0-only"}
            uncertainty = "Identity only: no matching verified composition source selected. Product composition must be supplied; no nearest-food or flour-to-pasta substitution is made."
        form, de_form = form_text(name, category)
        if id.endswith("-dry"):
            form, de_form = "dry/uncooked, as sold; edible portion", "trocken/ungekocht, wie verkauft; essbarer Anteil"
        translated["form"] = de_form
        for prefix, english, german in [("oat-drink", "Oat milk", "Hafermilch"), ("almond-drink", "Almond milk", "Mandelmilch"), ("soy-drink", "Soy milk", "Sojamilch"), ("rice-drink", "Rice milk", "Reismilch")]:
            if id.startswith(prefix):
                translated["aliases"] = sorted(set(translated.get("aliases", []) + [german]))
        aliases = list(EN_ALIASES.get(id, []))
        for prefix, alias in [("oat-drink", "Oat milk"), ("almond-drink", "Almond milk"), ("soy-drink", "Soy milk"), ("rice-drink", "Rice milk")]:
            if id.startswith(prefix): aliases.append(alias)
        families = [
            (("chickpeas-",), "Chickpeas", "Kichererbsen"),
            (("lentils-", "red-lentils-"), "Lentils", "Linsen"),
            (("peas-",), "Peas", "Erbsen"),
            (("potato-",), "Potatoes", "Kartoffeln"),
            (("tomato-", "tomatoes-"), "Tomatoes", "Tomaten"),
            (("onion-",), "Onions", "Zwiebeln"),
            (("egg-whole-",), "Eggs", "Eier"),
            (("egg-white-",), "Egg whites", "Eiklar"),
            (("chicken-",), "Chicken meat", "Hähnchenfleisch"),
            (("pasta-",), "Pasta", "Nudeln"),
        ]
        for prefixes, english, german in families:
            if id.startswith(prefixes):
                aliases.append(english)
                translated["aliases"] = translated.get("aliases", []) + [german]
        if category == "cheese":
            aliases.append("Cheese")
            translated["aliases"] = translated.get("aliases", []) + ["Käse"]
        if id.startswith("milk-") and "lactose-free" in id:
            aliases.append("Lactose-free milk")
            translated["aliases"] = translated.get("aliases", []) + ["Laktosefreie Milch"]
        aliases = sorted(set(aliases))
        translated["aliases"] = sorted(set(translated.get("aliases", [])))
        records.append({"id": id, "category": category, "name": name, "aliases": aliases, "form": form,
            "basis": "per_100g_edible_portion", "source": source, "uncertainty": uncertainty,
            "nutrients": nutrients, "education": [], "translations": {"de": translated}})
    assert len(records) == len({r["id"] for r in records})
    return {"schema_version": 1, "language": "en", "version": "mealshop-kitchen-2026-11-07", "ingredients": records}


if __name__ == "__main__":
    output = json.dumps(build(), indent=2, ensure_ascii=False) + "\n"
    path = ROOT / "data/catalogue.json"
    if "--check" in sys.argv:
        if path.read_text() != output:
            raise SystemExit("Run python3 scripts/import_catalogue.py and commit data/catalogue.json")
    else:
        path.write_text(output)
    print(f"Kitchen catalogue: {len(json.loads(output)['ingredients'])} records; {len(output.encode())} bytes; English base with German translations.")
