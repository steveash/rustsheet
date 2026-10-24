"""GETPIVOTDATA in generated cases: the case's pivot and the calls drawn from it (T-2396)."""

from __future__ import annotations

import io
import re
import zipfile
from typing import Any

from xlsxwriter.utility import xl_cell_to_rowcol, xl_rowcol_to_cell

from rustsheet_tools import repo_root
from rustsheet_tools.gen.generator import Catalog, generate_case
from rustsheet_tools.gen.pivots import CAPTIONS, OUTSIDE, SHEET, UNKNOWN_FIELDS, UNKNOWN_ITEMS
from rustsheet_tools.spec2xlsx import build_xlsx

ARGUMENT = re.compile(r'"(?:[^"]|"")*"|[^,]+')


def _catalog() -> Catalog:
    return Catalog.load(repo_root() / "eval" / "catalog" / "functions.yaml")


def _focus(seed: int, formulas: int = 40) -> dict[str, Any]:
    return generate_case(
        _catalog(), seed, milestone="M6", formulas=formulas, focus=True, functions=["GETPIVOTDATA"]
    )


def _arguments(formula: str) -> list[str]:
    inner = formula.removeprefix("=GETPIVOTDATA(").removesuffix(")")
    return [a.strip('"').replace('""', '"') if a.startswith('"') else a for a in ARGUMENT.findall(inner)]


def _fold(texts: str | set[str]) -> Any:
    """Case-folded: GETPIVOTDATA ignores case in names and items."""
    return texts.casefold() if isinstance(texts, str) else {t.casefold() for t in texts}


def test_a_getpivotdata_case_builds_with_its_pivot() -> None:
    spec = _focus(0)
    assert [s["name"] for s in spec["sheets"]] == ["Data", "Calc", SHEET]
    (pivot,) = spec["sheets"][2]["pivot_tables"]
    assert pivot["rendered"]
    with zipfile.ZipFile(io.BytesIO(build_xlsx(spec))) as package:
        names = package.namelist()
    assert "xl/pivotTables/pivotTable1.xml" in names
    assert "xl/pivotCache/pivotCacheRecords1.xml" in names


def test_calls_name_only_the_pivots_fields_items_and_cells_or_a_deliberate_miss() -> None:
    misses = 0
    for seed in range(12):
        spec = _focus(seed)
        sheet = spec["sheets"][2]
        pivot = sheet["pivot_tables"][0]
        header = {v for a1, v in sheet["cells"].items() if a1[1:] == "1"}
        axis = {pivot["row"], pivot.get("column")} - {None}
        items = {str(v) for a1, v in sheet["cells"].items() if a1[1:] != "1" and isinstance(v, str)}
        data_fields = {pivot["value"], f"{CAPTIONS[pivot['function']]} of {pivot['value']}"}
        # Every cell of the pivot's rectangle, blank intersections included.
        corners = [xl_cell_to_rowcol(a1) for a1 in pivot["rendered"]]
        inside = {
            f"{SHEET}!{xl_rowcol_to_cell(r, c, row_abs=True, col_abs=True)}"
            for r in range(min(r for r, _ in corners), max(r for r, _ in corners) + 1)
            for c in range(min(c for _, c in corners), max(c for _, c in corners) + 1)
        }
        for formula in spec["sheets"][1]["cells"].values():
            assert formula.startswith("=GETPIVOTDATA(")
            data_field, pivot_ref, *pairs = _arguments(formula)
            wrong = _fold(data_field) not in _fold(data_fields)
            assert _fold(data_field.strip()) in _fold(header | data_fields | set(UNKNOWN_FIELDS)) or _fold(
                data_field
            ).endswith(_fold(f" of {pivot['value']}"))
            first = pivot_ref.split(":")[0]
            wrong += first not in inside
            assert first in inside or first in {f"{SHEET}!{a1}" for a1 in OUTSIDE}
            # An odd count is only ever (data_field, pivot_table, field): Excel for the web refuses 5 or more.
            assert len(pairs) % 2 == 0 or len(pairs) == 1
            wrong += len(pairs) % 2
            names = [p.casefold() for p in pairs[::2]]
            wrong += len(names) != len(set(names))
            for name, item in zip(pairs[::2], pairs[1::2], strict=False):
                assert _fold(name) in _fold(header | set(UNKNOWN_FIELDS))
                wrong += _fold(name) not in _fold(axis)
                if item.startswith(f"{SHEET}!"):
                    assert item in inside
                else:
                    assert _fold(item.strip()) in _fold(items | set(UNKNOWN_ITEMS))
                    wrong += _fold(item) not in _fold(items)
            assert wrong <= 1, formula
            misses += wrong
    assert misses > 0


def test_cases_without_getpivotdata_have_no_pivot_sheet() -> None:
    spec = generate_case(_catalog(), 0, milestone="M5", formulas=40, focus=True)
    assert [s["name"] for s in spec["sheets"]] == ["Data", "Calc"]


def test_the_pivot_comes_from_its_own_stream() -> None:
    # The pivot depends on the seed only, not on how many draws the formulas took before the first call.
    a, b = _focus(5, formulas=3), _focus(5, formulas=30)
    assert a["sheets"][2] == b["sheets"][2]
