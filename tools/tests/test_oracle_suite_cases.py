from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path
from typing import Any

import pytest

from rustsheet_tools import snapshot
from rustsheet_tools.oracle import cli


def test_suite_cases_come_from_the_harness(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    listing = [{"id": "s/a.xlsx", "path": "/c/a.xlsx", "sidecar": "/s/oracle/a.oracle.json"}]
    seen: list[list[str]] = []

    def fake_run(cmd: list[str], **_: Any) -> subprocess.CompletedProcess[str]:
        seen.append(cmd)
        return subprocess.CompletedProcess(cmd, 0, stdout=json.dumps(listing))

    monkeypatch.setattr(snapshot, "repo_root", lambda: tmp_path)
    monkeypatch.setattr(snapshot, "eval_binary", lambda: Path("rustsheet-eval"))
    monkeypatch.setattr(snapshot.subprocess, "run", fake_run)
    assert snapshot.suite_cases("s") == listing
    assert seen == [["rustsheet-eval", "--root", str(tmp_path), "cases", "s"]]


@pytest.mark.parametrize(
    ("entry", "include_cached", "skipped"),
    [
        ({"formula_cells": 0, "trusts_cached_values": False}, False, True),  # nothing to score
        ({"formula_cells": 3, "trusts_cached_values": True}, False, True),  # cached values trusted
        ({"formula_cells": 3, "trusts_cached_values": True}, True, False),  # calibration run
        ({"formula_cells": 3, "trusts_cached_values": False}, False, False),  # what the oracle is for
        ({"unreadable": "missing workbook rels"}, True, True),  # the harness cannot read it either
    ],
)
def test_skip_reason(entry: dict[str, Any], include_cached: bool, skipped: bool) -> None:
    assert (cli._skip_reason(entry, include_cached) is not None) is skipped


def _cached(app: str, a1: dict[str, Any], version: str = "16.0300") -> dict[str, Any]:
    cells = {k: {"formula": "=X()", "formula_kind": {"kind": "normal"}, "value": v} for k, v in a1.items()}
    return {"application": app, "app_version": version, "sheets": [{"name": "S", "cells": cells}]}


def test_web_values_that_contradict_desktop_excel_are_left_pending(monkeypatch: pytest.MonkeyPatch) -> None:
    desktop = {"A1": {"t": "s", "v": "|1e5|"}, "A2": {"t": "n", "v": 0.1}, "A3": {"t": "n", "v": 1.0}}
    monkeypatch.setattr(cli, "snapshot", lambda _: _cached("Microsoft Macintosh Excel", desktop))
    web = {
        "S": {
            "A1": {"t": "e", "v": "#VALUE!"},
            "A2": {"t": "n", "v": 0.1 + 1e-17},
            "A3": {"t": "n", "v": 2.0},
        }
    }
    assert cli._drop_desktop_conflicts(Path("x.xlsx"), web) == 2
    assert web == {"S": {"A2": {"t": "n", "v": 0.1}}}  # last-digit noise: the 16.x desktop value wins


def test_desktop_numbers_replace_web_below_1e9_and_pending_above(monkeypatch: pytest.MonkeyPatch) -> None:
    desktop = {"A1": {"t": "n", "v": 1.0}, "A2": {"t": "n", "v": 1.0}, "A3": {"t": "n", "v": 1.0}}
    monkeypatch.setattr(cli, "snapshot", lambda _: _cached("Microsoft Excel", desktop))
    web = {
        "S": {"A1": {"t": "n", "v": 1.0}, "A2": {"t": "n", "v": 1.0 + 1e-12}, "A3": {"t": "n", "v": 1.001}}
    }
    assert cli._drop_desktop_conflicts(Path("x.xlsx"), web) == 1
    assert web == {"S": {"A1": {"t": "n", "v": 1.0}, "A2": {"t": "n", "v": 1.0}}}


def test_pre_16_producers_never_replace_a_web_number(monkeypatch: pytest.MonkeyPatch) -> None:
    desktop = {"A1": {"t": "n", "v": 1.0}}
    monkeypatch.setattr(cli, "snapshot", lambda _: _cached("Microsoft Excel", desktop, "12.0000"))
    web = {"S": {"A1": {"t": "n", "v": 1.0 + 1e-12}}}
    assert cli._drop_desktop_conflicts(Path("x.xlsx"), web) == 0
    assert web == {"S": {"A1": {"t": "n", "v": 1.0 + 1e-12}}}


def test_non_excel_cached_values_never_conflict(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(cli, "snapshot", lambda _: _cached("Apache POI", {"A1": {"t": "n", "v": 0}}))
    web = {"S": {"A1": {"t": "n", "v": 5.0}}}
    assert cli._drop_desktop_conflicts(Path("x.xlsx"), web) == 0
    assert web == {"S": {"A1": {"t": "n", "v": 5.0}}}


def test_unreadable_workbook_is_never_filled_unchecked(monkeypatch: pytest.MonkeyPatch) -> None:
    def boom(_: Path) -> dict[str, Any]:
        raise subprocess.CalledProcessError(1, ["rustsheet-eval"], stderr="missing workbook rels")

    monkeypatch.setattr(cli, "snapshot", boom)
    with pytest.raises(cli.OracleError, match="missing workbook rels"):
        cli._drop_desktop_conflicts(Path("x.xlsx"), {"S": {}})


def test_undecoded_rich_errors_are_never_written_as_goldens() -> None:
    values = {"S": {"A1": dict(cli.UNDECODED), "A2": {"t": "n", "v": 2.0}}}
    cli._drop_undecoded(Path("x.xlsx"), values)
    assert values == {"S": {"A2": {"t": "n", "v": 2.0}}}


def test_undecoded_marker_matches_the_harness_constant() -> None:
    rust = Path(__file__).resolve().parents[2] / "crates/rustsheet-eval/src/xlsx_snapshot.rs"
    m = re.search(r'pub const UNDECODED_RICH_ERROR: &str = "([^"]+)";', rust.read_text())
    assert m and m.group(1) == cli.UNDECODED["v"]


def test_invariant_cells_that_vary_between_fills_stay_pending() -> None:
    values = {"S": {"A1": {"t": "n", "v": 5.0}, "A2": {"t": "n", "v": 0.25}, "A3": {"t": "n", "v": 0.5}}}
    again = {"S": {"A1": {"t": "n", "v": 5.0}, "A2": {"t": "n", "v": 0.75}, "A3": {"t": "n", "v": 0.9}}}
    cli._drop_varying_invariants(Path("c.case.json"), ["S!A1", "'S'!$A$2"], values, again)
    # A3 varies too but is not listed: the harness skips it as volatile whatever the sidecar holds.
    assert values == {"S": {"A1": {"t": "n", "v": 5.0}, "A3": {"t": "n", "v": 0.5}}}
