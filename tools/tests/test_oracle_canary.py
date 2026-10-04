from __future__ import annotations

import io
import math
import re
import zipfile
from pathlib import Path
from types import SimpleNamespace
from typing import Any

import pytest

from rustsheet_tools.oracle import cli
from rustsheet_tools.oracle.backends import OracleError, OracleInfo
from rustsheet_tools.oracle.canary import SHEET, canary_problems, checked_values, embed_canary
from rustsheet_tools.snapshot import formula_values, snapshot
from rustsheet_tools.spec2xlsx import build_xlsx

INFO = OracleInfo(kind="fake", application="fake")
SPEC = {"sheets": [{"name": "Sheet1", "cells": {"A1": 2, "B1": "=A1*3"}}]}
# What Excel for the web computed for the canary on 2026-10-04 (RAND is any value in [0, 1)).
EXCEL = {"999": "2", "7": "0.54520464311930006", "-1": "2.68", "0": repr(math.sqrt(2))}


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
    assert len(canary_problems(values)) == 4


def test_uncalculated_workbook_is_refused(tmp_path: Path) -> None:
    path = tmp_path / "echo.xlsx"
    path.write_bytes(embed_canary(build_xlsx(SPEC)))
    with pytest.raises(OracleError, match="not recalculated") as e:
        checked_values(path)
    assert str(e.value).count("the uploaded cached value") == 4


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


def test_fill_writes_no_sidecar_when_the_canary_fails(
    cache_root: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    del cache_root
    case = tmp_path / "x.case.json"
    case.write_text('{"sheets": [{"name": "Sheet1", "cells": {"A1": "=1+1"}}]}')
    monkeypatch.setattr(cli, "GOLDEN_BACKENDS", {"echo"})
    monkeypatch.setattr(cli, "_backend", lambda _: EchoBackend())
    monkeypatch.setattr(cli, "_suite_cases", lambda _: ({}, [(case, tmp_path / "x.oracle.json")]))
    args = SimpleNamespace(backend="echo", suite="s", force=False, limit=0, include_cached=False)
    assert cli.cmd_fill(args) == 1
    assert not (tmp_path / "x.oracle.json").exists()


@pytest.mark.parametrize("formula", ["=SHEETS()", "=_xlfn.SHEETS(Sheet1:Sheet1)", '=info("numfile")'])
def test_sheet_count_functions_are_refused(formula: str) -> None:
    spec = {"sheets": [{"name": "Sheet1", "cells": {"A1": formula}}]}
    with pytest.raises(OracleError, match=r"SHEETS\(\) or INFO\(\)"):
        embed_canary(build_xlsx(spec))


def test_new_sheet_id_follows_the_largest_and_names_cannot_collide() -> None:
    parts = _parts(build_xlsx(SPEC))
    parts["xl/workbook.xml"] = parts["xl/workbook.xml"].replace('sheetId="1"', 'sheetId="7"')
    assert f'name="{SHEET}" sheetId="8"' in _parts(embed_canary(_repack(parts)))["xl/workbook.xml"]
    taken = build_xlsx({"sheets": [{"name": SHEET, "cells": {"A1": 1}}]})
    with pytest.raises(OracleError, match="already has a sheet"):
        embed_canary(taken)


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
    assert len(canary_problems(_values(embedded, tmp_path))) == 4
