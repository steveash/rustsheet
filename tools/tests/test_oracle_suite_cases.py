from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any

import pytest

from rustsheet_tools.oracle import cli


def test_suite_cases_come_from_the_harness(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    listing = [{"id": "s/a.xlsx", "path": "/c/a.xlsx", "sidecar": "/s/oracle/a.oracle.json"}]
    seen: list[list[str]] = []

    def fake_run(cmd: list[str], **_: Any) -> subprocess.CompletedProcess[str]:
        seen.append(cmd)
        return subprocess.CompletedProcess(cmd, 0, stdout=json.dumps(listing))

    monkeypatch.setattr(cli, "repo_root", lambda: tmp_path)
    monkeypatch.setattr(cli, "eval_binary", lambda: Path("rustsheet-eval"))
    monkeypatch.setattr(cli.subprocess, "run", fake_run)
    assert cli._suite_cases("s") == listing
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


def _cached(app: str, a1: dict[str, Any]) -> dict[str, Any]:
    cells = {k: {"formula": "=X()", "formula_kind": {"kind": "normal"}, "value": v} for k, v in a1.items()}
    return {"application": app, "sheets": [{"name": "S", "cells": cells}]}


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
    assert web == {"S": {"A2": {"t": "n", "v": 0.1 + 1e-17}}}  # last-digit noise is kept


def test_non_excel_cached_values_never_conflict(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(cli, "snapshot", lambda _: _cached("Apache POI", {"A1": {"t": "n", "v": 0}}))
    web = {"S": {"A1": {"t": "n", "v": 5.0}}}
    assert cli._drop_desktop_conflicts(Path("x.xlsx"), web) == 0
    assert web == {"S": {"A1": {"t": "n", "v": 5.0}}}
