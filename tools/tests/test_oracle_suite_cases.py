from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Any

import pytest

from rustsheet_tools.oracle import cli


def _suite(root: Path, toml: str, files: list[str]) -> None:
    sdir = root / "eval" / "suites" / "s"
    sdir.mkdir(parents=True)
    (sdir / "suite.toml").write_text(toml)
    for f in files:
        p = root / f
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(b"")


@pytest.fixture
def repo(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    monkeypatch.setattr(cli, "repo_root", lambda: tmp_path)
    return tmp_path


def test_local_suite_sidecar_sits_next_to_the_case(repo: Path) -> None:
    _suite(repo, 'name = "s"\nkind = "spec"\nroot = "cases"\n', ["eval/suites/s/cases/a.case.json"])
    _, cases = cli._suite_cases("s")
    case = repo / "eval/suites/s/cases/a.case.json"
    assert cases == [(case, case.with_name("a.oracle.json"))]


def test_fetched_suite_uses_the_cache_and_oracle_dir_mirror(repo: Path) -> None:
    toml = (
        'name = "s"\nkind = "xlsx"\nexclude = ["**/crash-*"]\noracle_dir = "oracle"\n'
        '[fetch]\nurl = "u"\nrev = "r"\n'
    )
    _suite(repo, toml, ["eval/.cache/suites/s/sub/b.xlsx", "eval/.cache/suites/s/crash-1.xlsx"])
    _, cases = cli._suite_cases("s")
    assert cases == [
        (repo / "eval/.cache/suites/s/sub/b.xlsx", repo / "eval/suites/s/oracle/sub/b.oracle.json")
    ]


def _snap(formula: bool, excel: bool) -> dict[str, Any]:
    cell = {"formula": "=1" if formula else None, "value": {"t": "n", "v": 1}}
    return {"excel_authored": excel, "sheets": [{"name": "S", "cells": {"A1": cell}}]}


@pytest.mark.parametrize(
    ("formula", "excel", "include_cached", "skipped"),
    [
        (False, False, False, True),  # nothing to score
        (True, True, False, True),  # cached values already trusted
        (True, True, True, False),  # calibration run
        (True, False, False, False),  # the pending case the oracle exists for
    ],
)
def test_skip_reason(
    monkeypatch: pytest.MonkeyPatch, formula: bool, excel: bool, include_cached: bool, skipped: bool
) -> None:
    monkeypatch.setattr(cli, "snapshot", lambda _: _snap(formula, excel))
    manifest = {"use_cached_values": True, "require_excel_authored": True}
    assert (cli._skip_reason(manifest, Path("x.xlsx"), include_cached) is not None) is skipped


def test_unreadable_workbook_is_skipped(monkeypatch: pytest.MonkeyPatch) -> None:

    def boom(_: Path) -> dict[str, Any]:
        raise subprocess.CalledProcessError(1, ["rustsheet-eval"], stderr="missing workbook rels")

    monkeypatch.setattr(cli, "snapshot", boom)
    reason = cli._skip_reason({}, Path("x.xlsx"), include_cached=False)
    assert reason is not None and "missing workbook rels" in reason
