"""The progress page's `codebase` block (SPEC-0022 later phase, T-0078).

Reads the quality ratchet files under quality/, the decision records and the Excel-semantics notes,
all read only, and reports each count with its change since the last complete history row. The
ratchets themselves are enforced by scripts/check_*.py; this module only counts what they store.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from rustsheet_tools.progress_report.markdown import signed, table
from rustsheet_tools.progress_report.model import ProgressReportError, count, docs_dir, field, load_object

QUALITY = "quality"
DECISIONS = "decisions"
SEMANTICS = "excel-semantics"
# The history row's `codebase` keys with the label of each count, in the order the block shows them.
COUNTS = (
    ("lint_allows", "Lint-debt allow entries"),
    ("duplicated_lines", "Duplicated lines"),
    ("oversized_files", "Oversized files"),
    ("below_floor", "Functions below the evidence floor"),
    ("decisions", "Decision records"),
    ("semantics_lines", "Excel-semantics lines"),
)


def _lint_allows(data: dict[str, Any], where: str) -> int:
    total = 0
    for manifest, lints in data.items():
        if not isinstance(lints, list):
            msg = f"{where}: {manifest} is not a list of lints"
            raise ProgressReportError(msg)
        total += len(lints)
    return total


def _records(directory: Path) -> list[Path]:
    """The markdown files of `directory`; a missing directory is an error rather
    than a count of zero."""
    if not directory.is_dir():
        msg = f"{directory} is not a directory"
        raise ProgressReportError(msg)
    return sorted(directory.glob("*.md"))


def _lines(path: Path) -> int:
    try:
        return len(path.read_text(encoding="utf-8").splitlines())
    except (OSError, UnicodeDecodeError) as e:
        msg = f"{path}: {e}"
        raise ProgressReportError(msg) from e


def measure(repo: Path) -> dict[str, int]:
    """The counts of the repository at `repo`."""
    docs = docs_dir(repo)
    quality = repo / QUALITY
    debt = quality / "lint-debt.json"
    dup, floor = quality / "duplication.json", quality / "evidence-floor.json"
    return {
        "lint_allows": _lint_allows(load_object(debt), str(debt)),
        "duplicated_lines": count(load_object(dup), "duplicated_lines", str(dup)),
        "oversized_files": len(load_object(quality / "oversized-files.json")),
        "below_floor": count(load_object(floor), "below_floor", str(floor)),
        "decisions": sum(1 for p in _records(docs / DECISIONS) if p.name.startswith("0")),
        "semantics_lines": sum(_lines(p) for p in _records(docs / SEMANTICS)),
    }


def check_row(obj: Any, where: str) -> None:
    """Check a history row's optional `codebase` object; rows without one are valid."""
    for key, _ in COUNTS:
        count(obj, key, where)


def block(counts: dict[str, int], last: dict[str, Any] | None) -> str:
    """The block's Markdown. `last` is the last complete history row; the change column is blank
    for a row from before the block existed."""
    before = last.get("codebase") if last else None
    rows = []
    for key, label in COUNTS:
        now = counts[key]
        change = signed(now - field(before, key, "history codebase")) if before else "n/a"
        rows.append([label, f"{now:,}", change])
    since = f" since {last['date']}" if before and last else ""
    return (
        "## Codebase health\n\n"
        f"Counts of the quality ratchets and knowledge files, with the change{since}. "
        "Lower is better for the first four rows.\n\n" + table(["Measure", "Now", "Change"], rows) + "\n"
    )
