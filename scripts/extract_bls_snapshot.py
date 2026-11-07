"""Extract reviewed BLS records from the official XLSX archive without changing numeric cell text."""
import argparse
import hashlib
from io import BytesIO
import json
from pathlib import Path
from xml.etree import ElementTree as ET
from zipfile import ZipFile
from import_catalogue import BLS_MAPPING, ROOT

NS = {"m": "http://schemas.openxmlformats.org/spreadsheetml/2006/main"}


def rows(content):
    with ZipFile(BytesIO(content)) as workbook:
        shared = []
        if "xl/sharedStrings.xml" in workbook.namelist():
            shared = ["".join(t.text or "" for t in item.findall(".//m:t", NS))
                      for item in ET.fromstring(workbook.read("xl/sharedStrings.xml"))]
        with workbook.open("xl/worksheets/sheet1.xml") as sheet:
            for _, row in ET.iterparse(sheet, events=["end"]):
                if row.tag != "{" + NS["m"] + "}row":
                    continue
                cells = {}
                for cell in row:
                    value = cell.find("m:v", NS)
                    text = value.text if value is not None else "".join(t.text or "" for t in cell.findall(".//m:t", NS))
                    if cell.get("t") == "s":
                        text = shared[int(text)]
                    column = "".join(c for c in cell.get("r") if c.isalpha())
                    cells[column] = text
                yield int(row.get("r")), cells
                row.clear()


def extract(archive):
    selection = json.loads((ROOT / "data/catalogue-selection.json").read_text())["entries"]
    selected = {r["record_id"] for r in selection if r["provider"] == "bls"}
    with ZipFile(archive) as package:
        data_name = next(n for n in package.namelist() if n.endswith("BLS_4_0_Daten_2025_DE.xlsx"))
        component_name = next(n for n in package.namelist() if n.endswith("BLS_4_0_Components_DE_EN.xlsx"))
        data = package.read(data_name)
        iterator = rows(data)
        _, headers = next(iterator)
        wanted = {column: name for column, name in headers.items()
                  if name in ["BLS Code", "Lebensmittelbezeichnung", "Food name"] or name.split(" ")[0] in BLS_MAPPING}
        records = []
        for index, values in iterator:
            if values.get("A") in selected:
                records.append({"sheet_row": index, **{name: values.get(column, "") for column, name in wanted.items()}})
        iterator = rows(package.read(component_name))
        _, header = next(iterator)
        components = []
        for _, values in iterator:
            record = {header[column]: value for column, value in values.items() if column in header}
            if record.get("Nährstoffcode / Component code") in BLS_MAPPING:
                components.append(record)
    if {r["BLS Code"] for r in records} != selected:
        raise ValueError("Some reviewed BLS source identities were not present in the archive")
    return {"publisher": "Max Rubner-Institut", "dataset": "Bundeslebensmittelschlüssel (BLS), Version 4.0 — Deutsche Nährstoffdatenbank",
        "release": "2025", "doi": "10.25826/Data20251217-134202-0", "license": "CC-BY-4.0",
        "license_url": "https://creativecommons.org/licenses/by/4.0/", "download_page": "https://blsdb.de/download",
        "retrieved_on": "2026-11-07", "archive_sha256": hashlib.sha256(archive.read_bytes()).hexdigest(),
        "workbook": data_name, "workbook_sha256": hashlib.sha256(data).hexdigest(), "sheet": "BLS_4_0_Daten_2025_DE",
        "components": components, "records": records}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("--output", type=Path, default=ROOT / "data/sources/bls-4.0-selected.json")
    args = parser.parse_args()
    snapshot = extract(args.archive)
    args.output.write_text(json.dumps(snapshot, indent=2, ensure_ascii=False) + "\n")
    print(f"Extracted {len(snapshot['records'])} BLS rows with source sheet/row, values, acquisition methods and references.")
