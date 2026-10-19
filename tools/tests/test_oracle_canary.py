from __future__ import annotations

import io
import math
import re
import zipfile
from pathlib import Path
from types import SimpleNamespace
from typing import Any

import pytest

from rustsheet_tools.oracle import canary, cli
from rustsheet_tools.oracle.backends import OracleError, OracleInfo, force_full_calc_on_load
from rustsheet_tools.oracle.canary import CELLS, SHEET, canary_problems, checked_values, embed_canary
from rustsheet_tools.snapshot import formula_values, snapshot
from rustsheet_tools.spec2xlsx import build_xlsx

INFO = OracleInfo(kind="fake", application="fake")
SPEC = {"sheets": [{"name": "Sheet1", "cells": {"A1": 2, "B1": "=A1*3"}}]}
# What Excel for the web computed for the canary (2026-10-04; A5 2026-10-11) (RAND is any value in [0, 1)).
EXCEL = {
    "999": "2",
    "7": "0.54520464311930006",
    "-1": "2.68",
    "0": repr(math.sqrt(2)),
    "-2": "1",
    "-3": "1.5",
}


def _parts(xlsx: bytes) -> dict[str, str]:
    zf = zipfile.ZipFile(io.BytesIO(xlsx))
    return {n: zf.read(n).decode() for n in zf.namelist()}


def _repack(parts: dict[str, str]) -> bytes:
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w") as zf:
        for name, text in parts.items():
            zf.writestr(name, text)
    return buf.getvalue()


class EchoBackend:
    """'Recalculates' by handing the upload back: the failure the canary exists for."""

    name = "echo"

    def __init__(self) -> None:
        self.calls = 0

    def available(self) -> tuple[bool, str]:
        return True, "test"

    def recalc(self, xlsx: bytes) -> tuple[bytes, OracleInfo]:
        self.calls += 1
        return xlsx, INFO


class ExcelLikeBackend(EchoBackend):
    """Writes Excel's answers into the canary and the case cell, like a real recalculation."""

    name = "excel-like"

    def recalc(self, xlsx: bytes) -> tuple[bytes, OracleInfo]:
        self.calls += 1
        parts = _parts(xlsx)
        canary = "xl/worksheets/rustsheet-canary.xml"
        parts[canary] = re.sub(r"<v>([^<]*)</v>", lambda m: f"<v>{EXCEL[m.group(1)]}</v>", parts[canary])
        parts["xl/worksheets/sheet1.xml"] = parts["xl/worksheets/sheet1.xml"].replace("<v>0</v>", "<v>6</v>")
        return _repack(parts), INFO


def _values(xlsx: bytes, tmp_path: Path) -> dict[str, dict[str, Any]]:
    path = tmp_path / "wb.xlsx"
    path.write_bytes(xlsx)
    return formula_values(snapshot(path))


def test_embedding_appends_a_poisoned_sheet_and_keeps_the_case(tmp_path: Path) -> None:
    raw = build_xlsx(SPEC)
    embedded = embed_canary(raw)
    before, after = _parts(raw), _parts(embedded)
    assert after["xl/worksheets/sheet1.xml"] == before["xl/worksheets/sheet1.xml"]
    sheets = re.findall(r'<sheet name="([^"]+)" sheetId="(\d+)"', after["xl/workbook.xml"])
    assert sheets == [("Sheet1", "1"), (SHEET, "2")]  # last, so sheet indexes don't shift
    values = _values(embedded, tmp_path)
    assert values["Sheet1"] == _values(raw, tmp_path)["Sheet1"]
    assert len(canary_problems(values)) == len(CELLS)


def test_uncalculated_workbook_is_refused(tmp_path: Path) -> None:
    path = tmp_path / "echo.xlsx"
    path.write_bytes(embed_canary(build_xlsx(SPEC)))
    with pytest.raises(OracleError, match="not recalculated") as e:
        checked_values(path)
    assert str(e.value).count("the uploaded cached value") == len(CELLS)


@pytest.fixture
def cache_root(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    monkeypatch.setattr(cli, "repo_root", lambda: tmp_path)
    return tmp_path / "eval" / ".cache" / "oracle"


def test_recalculated_values_come_back_without_the_canary(cache_root: Path) -> None:
    backend = ExcelLikeBackend()
    values, _, _ = cli._recalc_cached(backend, build_xlsx(SPEC))
    assert values == {"Sheet1": {"B1": {"t": "n", "v": 6.0}}}
    assert cli._recalc_cached(backend, build_xlsx(SPEC))[0] == values
    assert backend.calls == 1  # second answer came from the verified cache
    assert len(list((cache_root / "excel-like").iterdir())) == 2


def test_failed_canary_is_not_cached(cache_root: Path) -> None:
    with pytest.raises(OracleError, match="not recalculated"):
        cli._recalc_cached(EchoBackend(), build_xlsx(SPEC))
    assert list((cache_root / "echo").iterdir()) == []


def test_cache_entry_without_a_passing_canary_is_recomputed(cache_root: Path) -> None:
    raw = build_xlsx(SPEC)
    backend = ExcelLikeBackend()
    sha = cli._recalc_cached(backend, raw)[2]
    (cache_root / backend.name / f"{sha}.xlsx").write_bytes(raw)  # e.g. cached before canaries
    cli._recalc_cached(backend, raw)
    assert backend.calls == 2


def test_refusals_apply_to_results_cached_before_them(
    cache_root: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    del cache_root
    raw = build_xlsx({"sheets": [{"name": "Sheet1", "cells": {"A1": '=CELL("width",A:A)'}}]})
    backend = ExcelLikeBackend()
    with monkeypatch.context() as m:
        m.setattr(canary, "_REFUSALS", ())  # cached while CELL("width") was still accepted
        cli._recalc_cached(backend, raw)
    with pytest.raises(OracleError, match="width"):
        cli._recalc_cached(backend, raw)
    assert backend.calls == 1


def test_fill_writes_no_sidecar_when_the_canary_fails(
    cache_root: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    del cache_root
    case = tmp_path / "x.case.json"
    case.write_text('{"sheets": [{"name": "Sheet1", "cells": {"A1": "=1+1"}}]}')
    monkeypatch.setattr(cli, "GOLDEN_BACKENDS", {"echo"})
    monkeypatch.setattr(cli, "_backend", lambda _: EchoBackend())
    entry = {"path": str(case), "sidecar": str(tmp_path / "x.oracle.json")}
    monkeypatch.setattr(cli, "suite_cases", lambda _: [entry])
    args = SimpleNamespace(backend="echo", suite="s", force=False, limit=0, case=[], include_cached=False)
    assert cli.cmd_fill(args) == 1
    assert not (tmp_path / "x.oracle.json").exists()


@pytest.mark.parametrize("formula", ["=SHEETS()", "=_xlfn.SHEETS(Sheet1:Sheet1)", '=info("numfile")'])
def test_sheet_count_functions_are_refused(formula: str) -> None:
    spec = {"sheets": [{"name": "Sheet1", "cells": {"A1": formula}}]}
    with pytest.raises(OracleError, match=r"SHEETS\(\) or INFO\(\)"):
        embed_canary(build_xlsx(spec))


@pytest.mark.parametrize("formula", ['=CELL("width",A:A )', '=cell( "Width",B1)'])
def test_web_unsupported_cell_width_is_refused(formula: str) -> None:
    spec = {"sheets": [{"name": "Sheet1", "cells": {"A1": formula}}]}
    with pytest.raises(OracleError, match=r"CELL\(\"width\"\)"):
        embed_canary(build_xlsx(spec))


def test_cell_with_other_info_types_is_not_refused() -> None:
    embed_canary(build_xlsx({"sheets": [{"name": "Sheet1", "cells": {"A1": '=CELL("row",A1)'}}]}))


def test_new_sheet_id_follows_the_largest_and_names_cannot_collide() -> None:
    parts = _parts(build_xlsx(SPEC))
    parts["xl/workbook.xml"] = parts["xl/workbook.xml"].replace('sheetId="1"', 'sheetId="7"')
    assert f'name="{SHEET}" sheetId="8"' in _parts(embed_canary(_repack(parts)))["xl/workbook.xml"]
    taken = build_xlsx({"sheets": [{"name": SHEET, "cells": {"A1": 1}}]})
    with pytest.raises(OracleError, match="already has a sheet"):
        embed_canary(taken)


def _move_sheet(parts: dict[str, str], new: str) -> dict[str, str]:
    """The same package with its worksheet stored at `new`, which only the workbook rels point at."""
    old = "xl/worksheets/sheet1.xml"
    moved = {(new if n == old else n): t for n, t in parts.items()}
    moved["xl/_rels/workbook.xml.rels"] = moved["xl/_rels/workbook.xml.rels"].replace(
        "worksheets/sheet1.xml", new.removeprefix("xl/")
    )
    moved["[Content_Types].xml"] = moved["[Content_Types].xml"].replace(f"/{old}", f"/{new}")
    return moved


def test_refusal_scan_follows_the_workbook_rels_to_a_nonstandard_sheet_path() -> None:
    spec = {"sheets": [{"name": "Sheet1", "cells": {"A1": "=SHEETS()"}}]}
    moved = _move_sheet(_parts(build_xlsx(spec)), "xl/other/data.xml")
    with pytest.raises(OracleError, match=r"xl/other/data.xml uses SHEETS\(\)"):
        embed_canary(_repack(moved))


def _rels_path() -> str:
    return "xl/_rels/workbook.xml.rels"


def test_refusal_scan_reads_single_quoted_and_entity_escaped_rels() -> None:
    spec = {"sheets": [{"name": "Sheet1", "cells": {"A1": "=SHEETS()"}}]}
    parts = _move_sheet(_parts(build_xlsx(spec)), "xl/other/data.xml")
    rels = parts[_rels_path()]
    parts[_rels_path()] = rels.replace("other/data.xml", "other/dat&#97;.xml").replace('"', "'")
    with pytest.raises(OracleError, match=r"xl/other/data.xml uses SHEETS\(\)"):
        embed_canary(_repack(parts))


def test_a_relationship_to_a_missing_sheet_part_is_refused() -> None:
    parts = _parts(build_xlsx(SPEC))
    parts[_rels_path()] = parts[_rels_path()].replace("worksheets/sheet1.xml", "worksheets/missing.xml")
    with pytest.raises(OracleError, match="not in the package"):
        embed_canary(_repack(parts))


def test_unparseable_workbook_rels_are_refused() -> None:
    parts = _parts(build_xlsx(SPEC))
    parts[_rels_path()] = "<Relationships><Relationship"
    with pytest.raises(OracleError, match=r"unreadable xl/_rels/workbook.xml.rels"):
        embed_canary(_repack(parts))


def test_workbook_rels_with_a_dtd_are_refused() -> None:
    parts = _parts(build_xlsx(SPEC))
    parts[_rels_path()] = '<!DOCTYPE r [<!ENTITY a "b">]>' + parts[_rels_path()]
    with pytest.raises(OracleError, match="declares a DTD"):
        embed_canary(_repack(parts))


def test_a_sheet_in_the_worksheets_directory_is_scanned_even_without_a_relationship() -> None:
    parts = _parts(build_xlsx(SPEC))
    parts["xl/worksheets/orphan.xml"] = (
        "<worksheet><sheetData><row><c><f>SHEETS()</f></c></row></sheetData></worksheet>"
    )
    with pytest.raises(OracleError, match=r"xl/worksheets/orphan.xml uses SHEETS\(\)"):
        embed_canary(_repack(parts))


def test_a_zero_byte_directory_entry_under_worksheets_is_skipped() -> None:
    parts = _parts(build_xlsx(SPEC))
    parts["xl/worksheets/"] = ""
    assert SHEET in _parts(embed_canary(_repack(parts)))["xl/workbook.xml"]


def test_a_directory_named_entry_with_data_is_still_refused() -> None:
    parts = _parts(build_xlsx(SPEC))
    parts["xl/worksheets/"] = "not xml"
    with pytest.raises(OracleError, match="xl/worksheets/"):
        embed_canary(_repack(parts))


def test_a_clean_sheet_at_a_nonstandard_path_is_embedded() -> None:
    moved = _move_sheet(_parts(build_xlsx(SPEC)), "xl/other/data.xml")
    assert SHEET in _parts(embed_canary(_repack(moved)))["xl/workbook.xml"]


def test_xlm_get_workbook_in_a_defined_name_is_refused() -> None:
    parts = _parts(build_xlsx(SPEC))
    names = '<definedNames><definedName name="Sheets">GET.WORKBOOK(1)</definedName></definedNames>'
    parts["xl/workbook.xml"] = parts["xl/workbook.xml"].replace("</sheets>", f"</sheets>{names}")
    with pytest.raises(OracleError, match=r"GET\.WORKBOOK\(\)"):
        embed_canary(_repack(parts))


def _with_part(xlsx: bytes, name: str, data: bytes) -> bytes:
    """`xlsx` with part `name` replaced by raw `data` (for encodings `_repack` cannot write)."""
    src = zipfile.ZipFile(io.BytesIO(xlsx))
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w") as zf:
        for item in src.infolist():
            zf.writestr(item.filename, data if item.filename == name else src.read(item))
    return buf.getvalue()


def test_a_utf16_rels_part_with_a_dtd_is_refused() -> None:
    raw = build_xlsx(SPEC)
    rels = _parts(raw)[_rels_path()].split("?>", 1)[1]
    doc = '<?xml version="1.0" encoding="UTF-16"?><!DOCTYPE r [<!ENTITY a "b">]>' + rels
    with pytest.raises(OracleError, match="declares a DTD"):
        embed_canary(_with_part(raw, _rels_path(), doc.encode("utf-16")))


@pytest.mark.parametrize(
    "formula",
    [
        "SHEETS&#40;)",
        "SHE<!-- -->ETS()",
        "<![CDATA[SHE]]>ETS()",
        "_xlfn.SHEETS&#x28;)",
        "INFO&#40;&quot;x&quot;)",
    ],
)
def test_refusal_scan_reads_unescaped_text(formula: str) -> None:
    parts = _parts(build_xlsx(SPEC))
    parts["xl/worksheets/sheet1.xml"] = parts["xl/worksheets/sheet1.xml"].replace(
        "</sheetData>", f"<row r='9'><c r='A9'><f>{formula}</f></c></row></sheetData>"
    )
    with pytest.raises(OracleError, match=r"SHEETS\(\) or INFO\(\)"):
        embed_canary(_repack(parts))


def test_refusal_scan_reads_a_utf16_sheet() -> None:
    raw = build_xlsx({"sheets": [{"name": "Sheet1", "cells": {"A1": "=SHEETS()"}}]})
    sheet = _parts(raw)["xl/worksheets/sheet1.xml"].split("?>", 1)[1]
    doc = '<?xml version="1.0" encoding="UTF-16"?>' + sheet
    with pytest.raises(OracleError, match=r"SHEETS\(\)"):
        embed_canary(_with_part(raw, "xl/worksheets/sheet1.xml", doc.encode("utf-16")))


def test_refusal_scan_covers_formulas_outside_cells() -> None:
    parts = _parts(build_xlsx(SPEC))
    validation = (
        '<dataValidations count="1"><dataValidation type="whole" sqref="B1">'
        "<formula1>SHEETS()</formula1></dataValidation></dataValidations>"
    )
    parts["xl/worksheets/sheet1.xml"] = parts["xl/worksheets/sheet1.xml"].replace(
        "</sheetData>", f"</sheetData>{validation}"
    )
    with pytest.raises(OracleError, match=r"SHEETS\(\)"):
        embed_canary(_repack(parts))


def test_an_unparseable_sheet_part_is_refused() -> None:
    parts = _parts(build_xlsx(SPEC))
    parts["xl/worksheets/sheet1.xml"] = "<worksheet><sheetData>"
    with pytest.raises(OracleError, match=r"unreadable xl/worksheets/sheet1\.xml"):
        embed_canary(_repack(parts))


def test_duplicate_part_names_are_refused() -> None:
    raw = build_xlsx(SPEC)
    buf = io.BytesIO(raw)
    with pytest.warns(UserWarning, match="Duplicate name"), zipfile.ZipFile(buf, "a") as zf:
        zf.writestr("xl/worksheets/sheet1.xml", "<worksheet/>")
    with pytest.raises(OracleError, match="duplicate part names"):
        embed_canary(buf.getvalue())


@pytest.mark.parametrize("has_calc_pr", [False, True])
def test_force_full_calc_on_load_sets_the_flag_once(has_calc_pr: bool) -> None:
    parts = _parts(build_xlsx(SPEC))
    if has_calc_pr:
        parts["xl/workbook.xml"] = parts["xl/workbook.xml"].replace(
            "</workbook>", '<calcPr calcId="1"/></workbook>'
        )
    text = _parts(force_full_calc_on_load(_repack(parts)))["xl/workbook.xml"]
    assert text.count('fullCalcOnLoad="1"') == 1
    assert (
        _parts(force_full_calc_on_load(_repack({**parts, "xl/workbook.xml": text})))["xl/workbook.xml"]
        == text
    )


def test_not_an_xlsx_is_an_oracle_error() -> None:
    with pytest.raises(OracleError, match="unreadable"):
        embed_canary(b"not a zip")


@pytest.mark.parametrize(
    ("a1", "got"),
    [
        ("A1", {"t": "n", "v": 3.0}),
        ("A2", {"t": "n", "v": 1.0}),  # RAND() is < 1
        ("A3", {"t": "n", "v": 2.67}),  # binary-float rounding, not Excel's
        ("A4", {"t": "n", "v": 1.4142135623731}),  # 15-digit JSON precision, not the file's 17
        ("A5", {"t": "n", "v": 31.0}),  # day-first locale: 1 Feb 2001
        ("A6", {"t": "e", "v": "#VALUE!"}),  # comma-decimal locale
        ("A1", {"t": "s", "v": "2"}),
        ("A1", None),
    ],
)
def test_each_wrong_canary_value_is_reported(a1: str, got: Any) -> None:
    good: dict[str, Any] = {
        "A1": {"t": "n", "v": 2.0},
        "A2": {"t": "n", "v": 0.5},
        "A3": {"t": "n", "v": 2.68},
        "A4": {"t": "n", "v": math.sqrt(2)},
        "A5": {"t": "n", "v": 1.0},
        "A6": {"t": "n", "v": 1.5},
    }
    assert canary_problems({SHEET: good}) == []
    good[a1] = got
    problems = canary_problems({SHEET: {k: v for k, v in good.items() if v is not None}})
    assert len(problems) == 1
    assert problems[0].startswith(a1)


def test_namespace_prefixed_package_parts(tmp_path: Path) -> None:
    # Some writers (e.g. POI's 58760.xlsx) prefix the relationships namespace; Excel accepts it.
    parts = _parts(build_xlsx(SPEC))
    rels = "xl/_rels/workbook.xml.rels"
    parts[rels] = (
        parts[rels]
        .replace("<Relationships xmlns=", "<ns0:Relationships xmlns:ns0=")
        .replace("<Relationship ", "<ns0:Relationship ")
        .replace("</Relationships>", "</ns0:Relationships>")
    )
    embedded = embed_canary(_repack(parts))
    assert '<ns0:Relationship Id="rIdRustSheetCanary"' in _parts(embedded)[rels]
    assert len(canary_problems(_values(embedded, tmp_path))) == len(CELLS)
