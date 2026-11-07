"""Golden pivot parts shared with the Rust copy of the case builder (T-4152).

The eval adapter writes a defined pivot's parts in Rust (crates/rustsheet-eval/src/pivot_definition.rs,
which mirrors pivot_definition.py) so the engine loads what Excel opened. Both builders must write the same
parts: this test pins the Python builder's parts for a few representative probe pivots to the golden files in
tests/pivot_parts_golden/, and crates/rustsheet-eval/src/pivot_definition/tests.rs pins the Rust builder's to
the same files. The files hold each part in a canonical form (`canonical`: one line per element, attributes
sorted, namespace declarations left out), as the builders order an author's attributes differently.

`RUSTSHEET_UPDATE_GOLDEN=1 uv run pytest tests/test_pivot_parts_golden.py` rewrites them from this builder.
"""

import json
import os
import xml.etree.ElementTree as ET
from pathlib import Path

import pytest
import xlsxwriter.utility

from rustsheet_tools.pivot import _source
from rustsheet_tools.pivot_definition import DefinedPivot

ROOT = Path(__file__).resolve().parents[2]
CASES = ROOT / "eval" / "suites" / "probes" / "cases"
GOLDEN = Path(__file__).resolve().parent / "pivot_parts_golden"
# (case stem, pivot sheet): grouping, custom subtotals, pages with hidden items, a calculated field with a
# raw `filters` slot, a pivot saved without refreshOnLoad with `area` and raw layout lines, an item order
# over mixed types, show-values-as with a number format, `cache_names`, an x14 show-as mode (the data field's
# extLst), `item_attrs`, a pivotField's raw extLst (T-4129), and a raw autoSortScope, a calculated item with a raw
# pivotArea and its own `f`, and page-field attributes (T-4130), an `item_order` beside an `order` (T-4164), and a
# discrete group with custom subtotals on its added field and three date levels (T-4170), and a raw groupItems
# slot and a raw record value with tuples (T-4767).
PIVOTS = [
    ("t4128-pivot-spike", "G"),
    ("t4622-pivot-count-subtotals", "B1"),
    ("t4137-pivot-captions", "K4"),
    ("t4128-pivot-spike", "K"),
    ("t4586-pivot-refresh-on-load", "A"),
    ("t4134-pivot-items", "I2"),
    ("t4123-pivot-capture", "T"),
    ("t4627-pivot-field-name-collisions", "P_Const"),
    ("t4129-pivot-show-as-x14", "RD"),
    ("t4129-pivot-item-flags", "IE"),
    ("t4129-pivot-layout-flags", "X14F"),
    ("t4130-pivot-sort", "SVC"),
    ("t4130-pivot-calc-items", "CIA0"),
    ("t4130-pivot-field-filters", "PC"),
    ("t4164-pivot-item-order", "IOO"),
    ("t4170-pivot-manual-groups", "MS"),
    ("t4170-pivot-date-levels", "DS"),
    ("t4767-pivot-group-items", "GEA"),
    ("t4767-pivot-record-tuples", "RE"),
]


def canonical(xml: str) -> str:
    """One line per element, depth-indented: its local name, then its attributes by local name, sorted,
    values unescaped; an element's non-blank text follows on its own `#text` line."""
    lines: list[str] = []

    def walk(element: ET.Element, depth: int) -> None:
        attrs = sorted((k.rsplit("}", 1)[-1], v) for k, v in element.attrib.items())
        lines.append("  " * depth + element.tag.rsplit("}", 1)[-1] + "".join(f" {k}={v}" for k, v in attrs))
        if element.text and element.text.strip():
            lines.append("  " * (depth + 1) + "#text " + element.text.strip())
        for child in element:
            walk(child, depth + 1)

    walk(ET.fromstring(xml), 0)  # noqa: S314 - the builder's own output, not a file from outside
    return "\n".join(lines)


def golden_text(stem: str, sheet_name: str) -> str:
    spec = json.loads((CASES / f"{stem}.case.json").read_text(encoding="utf-8"))
    sheet = next(s for s in spec["sheets"] if s["name"] == sheet_name)
    raw = sheet["pivot_tables"][0]
    source_sheet, source_ref, rows = _source(spec, sheet["name"], str(raw["source"]), strict=False)
    pivot = DefinedPivot(raw, rows[0], rows[1:])
    top, left = xlsxwriter.utility.xl_cell_to_rowcol(str(raw["location"]).replace("$", ""))
    parts = [
        ("cache_definition", pivot.cache_definition(source_sheet, source_ref)),
        ("cache_records", pivot.cache_records()),
        ("table", pivot.table_definition(1, top, left)),
    ]
    return "".join(f"== {name}\n{canonical(xml)}\n" for name, xml in parts)


@pytest.mark.parametrize(("stem", "sheet"), PIVOTS)
def test_the_builder_writes_the_golden_parts(stem: str, sheet: str) -> None:
    path = GOLDEN / f"{stem}__{sheet}.txt"
    text = golden_text(stem, sheet)
    if os.environ.get("RUSTSHEET_UPDATE_GOLDEN"):
        path.write_text(text, encoding="utf-8")
    assert path.read_text(encoding="utf-8") == text


def test_every_golden_file_is_listed() -> None:
    assert sorted(p.name for p in GOLDEN.glob("*.txt")) == sorted(f"{s}__{n}.txt" for s, n in PIVOTS)
