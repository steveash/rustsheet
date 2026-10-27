"""The progress page's `codebase` block (SPEC-0022 later phase, T-0078).

Reads the quality ratchet files under quality/, the decision records and the Excel-semantics notes,
all read only, and reports each count with its change since the last complete history row. The
ratchets themselves are enforced by scripts/check_*.py; this module only counts what they store.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from rustsheet_tools.progress_report.markdown import signed, table
from rustsheet_tools.progress_report.model import ProgressReportError, count, field

QUALITY = "quality"
DECISIONS = "docs/decisions"
SEMANTICS = "docs/excel-semantics"
# The history row's `codebase` keys with the label of each count, in the order the block shows them.
MEASURES = (
    ("lint_allows", "Lint-debt allow entries"),
    ("duplicated_lines", "Duplicated lines"),
    ("oversized_files", "Oversized files"),
    ("below_floor", "Functions below the evidence floor"),
    ("decisions", "Decision records"),
    ("semantics_lines", "Excel-semantics lines"),
)


@dataclass(frozen=True)
class Codebase:
    """The counts of one run, keyed as in `MEASURES`."""

    counts: dict[str, int]


def _json(path: Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        msg = f"{path}: {e}"
        raise ProgressReportError(msg) from e
    if not isinstance(data, dict):
        msg = f"{path} is not a JSON object"
        raise ProgressReportError(msg)
    return data


def _lint_allows(data: dict[str, Any], where: str) -> int:
    total = 0
    for manifest, lints in data.items():
        if not isinstance(lints, list):
            msg = f"{where}: {manifest} is not a list of lints"
            raise ProgressReportError(msg)
        total += len(lints)
    return total


def _lines(path: Path) -> int:
    try:
        return len(path.read_text(encoding="utf-8").splitlines())
    except (OSError, UnicodeDecodeError) as e:
        msg = f"{path}: {e}"
        raise ProgressReportError(msg) from e


def measure(repo: Path) -> Codebase:
    """The counts of the repository at `repo`."""
    quality = repo / QUALITY
    debt = quality / "lint-debt.json"
    dup, floor = quality / "duplication.json", quality / "evidence-floor.json"
    where = f"{dup}"
    counts = {
        "lint_allows": _lint_allows(_json(debt), str(debt)),
        "duplicated_lines": count(_json(dup), "duplicated_lines", where),
        "oversized_files": len(_json(quality / "oversized-files.json")),
        "below_floor": count(_json(floor), "below_floor", str(floor)),
        "decisions": len(list((repo / DECISIONS).glob("0*.md"))),
        "semantics_lines": sum(_lines(p) for p in sorted((repo / SEMANTICS).glob("*.md"))),
    }
    return Codebase(counts)


def row(c: Codebase) -> dict[str, int]:
    """The history row's `codebase` object."""
    return dict(c.counts)


def check_row(obj: Any, where: str) -> None:
    """Check a history row's optional `codebase` object; rows without one are valid."""
    for key, _ in MEASURES:
        count(obj, key, where)


def block(c: Codebase, last: dict[str, Any] | None) -> str:
    """The block's Markdown. `last` is the last complete history row; the change column is blank
    for a row from before the block existed."""
    before = last.get("codebase") if last else None
    rows = []
    for key, label in MEASURES:
        now = c.counts[key]
        change = signed(now - field(before, key, "history codebase")) if before else "n/a"
        rows.append([label, f"{now:,}", change])
    since = f" since {last['date']}" if before and last else ""
    return (
        "## Codebase health\n\n"
        f"Counts of the quality ratchets and knowledge files, with the change{since}. "
        "Lower is better for the first four rows.\n\n" + table(["Measure", "Now", "Change"], rows) + "\n"
    )
