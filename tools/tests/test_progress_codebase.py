"""The progress page's `codebase` block and the history row's optional `codebase` (T-0078)."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from test_progress_history import row

from rustsheet_tools.progress_report import codebase, history
from rustsheet_tools.progress_report.model import ProgressReportError


def repo(tmp_path: Path, *, decisions: int = 2, semantics: tuple[str, ...] = ("a\nb\n", "c\n")) -> Path:
    q = tmp_path / "quality"
    q.mkdir()
    (q / "lint-debt.json").write_text(json.dumps({"a/Cargo.toml": ["x", "y"], "b/Cargo.toml": ["z"]}))
    (q / "duplication.json").write_text(json.dumps({"duplicated_lines": 120, "window": 10}))
    (q / "oversized-files.json").write_text(json.dumps({"f.rs": 1000, "g.rs": 2000, "h.rs": 3000}))
    (q / "evidence-floor.json").write_text(json.dumps({"floor": 20, "below_floor": 7}))
    d = tmp_path / "docs" / "decisions"
    d.mkdir(parents=True)
    for i in range(decisions):
        (d / f"{i:04d}-x.md").write_text("x")
    (d / "README.md").write_text("not a record")
    s = tmp_path / "docs" / "excel-semantics"
    s.mkdir()
    for i, text in enumerate(semantics):
        (s / f"{i}.md").write_text(text)
    return tmp_path


def test_each_count_is_read_from_its_file(tmp_path: Path) -> None:
    assert codebase.measure(repo(tmp_path)).counts == {
        "lint_allows": 3,
        "duplicated_lines": 120,
        "oversized_files": 3,
        "below_floor": 7,
        "decisions": 2,
        "semantics_lines": 3,
    }


def test_a_missing_ratchet_file_fails(tmp_path: Path) -> None:
    (r := repo(tmp_path))
    (r / "quality" / "duplication.json").unlink()
    with pytest.raises(ProgressReportError, match="duplication.json"):
        codebase.measure(r)


def test_block_shows_the_change_against_the_last_row(tmp_path: Path) -> None:
    now = codebase.measure(repo(tmp_path))
    last = row("2026-10-18")
    last["codebase"] = {**now.counts, "duplicated_lines": 130, "decisions": 1}
    text = codebase.block(now, last)
    assert "| Duplicated lines | 120 | \N{MINUS SIGN}10 |" in text
    assert "| Decision records | 2 | +1 |" in text
    assert "| Oversized files | 3 | 0 |" in text


def test_block_without_an_earlier_codebase_row_has_no_change(tmp_path: Path) -> None:
    text = codebase.block(codebase.measure(repo(tmp_path)), row("2026-10-18"))
    assert "| Oversized files | 3 | n/a |" in text


def test_rows_without_a_codebase_object_still_load() -> None:
    older = row("2026-10-18")
    assert "codebase" not in older
    assert history.parse(history.dump([older])) == [older]


def test_a_codebase_object_is_checked_and_round_trips(tmp_path: Path) -> None:
    r = row("2026-10-18")
    r["codebase"] = codebase.row(codebase.measure(repo(tmp_path)))
    assert history.parse(history.dump([r])) == [r]
    del r["codebase"]["decisions"]
    with pytest.raises(ProgressReportError, match="decisions"):
        history.parse(history.dump([r]))
