"""Oracle calibration: how often does the oracle agree with Excel's own cached values?

For an `xlsx` suite of Excel-authored workbooks (e.g. ironcalc-calc), every workbook is
recalculated by an oracle backend (canary-verified, cached like `fill`) and each non-volatile
formula cell is compared with the value Excel stored when it last saved the file. Comparison
rules, volatile skipping and function attribution come from `rustsheet-eval compare`, so the
numbers mean what eval scores mean. Report: docs/research/oracle-calibration.md (T-0004).
"""

from __future__ import annotations

import concurrent.futures as cf
import threading
from collections import Counter, defaultdict
from collections.abc import Callable, Sequence
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from rustsheet_tools.oracle.backends import OracleError

NO_FUNCTION = "(operators only)"


@dataclass
class FunctionStats:
    cells: int = 0
    verdicts: Counter[str] = field(default_factory=Counter)

    @property
    def disagreements(self) -> int:
        return self.cells - self.verdicts["pass"]


@dataclass
class Calibration:
    workbooks: int = 0
    refused: dict[str, str] = field(default_factory=dict)
    """Workbook → why no oracle result exists (canary refusal, unsupported workbook, ...)."""
    volatile_skipped: int = 0
    total: FunctionStats = field(default_factory=FunctionStats)
    by_function: dict[str, FunctionStats] = field(default_factory=lambda: defaultdict(FunctionStats))
    disagreements: list[dict[str, Any]] = field(default_factory=list)

    def add(self, workbook: str, comparison: dict[str, Any]) -> None:
        """Folds one workbook's `rustsheet-eval compare` output into the totals."""
        self.workbooks += 1
        self.volatile_skipped += comparison["volatile_skipped"]
        for cell in comparison["cells"]:
            verdict = cell["verdict"] or "missing"
            self.total.cells += 1
            self.total.verdicts[verdict] += 1
            for name in cell["functions"] or [NO_FUNCTION]:
                stats = self.by_function[name]
                stats.cells += 1
                stats.verdicts[verdict] += 1
            if verdict != "pass":
                self.disagreements.append({"workbook": workbook, **cell})


def calibrate(
    workbooks: Sequence[Path],
    recalculated: Callable[[Path], Path],
    compare: Callable[[Path, Path], dict[str, Any]],
    jobs: int = 1,
) -> Calibration:
    """Recalculates and compares every workbook; `recalculated` maps a workbook to the oracle's
    recalculated copy (raising OracleError when there is none)."""
    result = Calibration()
    lock = threading.Lock()

    def one(path: Path) -> None:
        try:
            comparison = compare(path, recalculated(path))
        except OracleError as e:
            with lock:
                result.refused[path.name] = str(e).splitlines()[0]
            return
        with lock:
            result.add(path.name, comparison)

    with cf.ThreadPoolExecutor(max_workers=max(1, jobs)) as pool:
        list(pool.map(one, workbooks))
    result.disagreements.sort(key=lambda d: (d["workbook"], d["sheet"], d["cell"]))
    return result


def _pct(part: int, whole: int) -> str:
    return f"{100 * part / whole:.2f}%" if whole else "n/a"


def to_markdown(cal: Calibration, examples: int = 50) -> str:
    """Tables for the calibration report: totals, verdicts, functions with disagreements."""
    t = cal.total
    lines = [
        f"- Workbooks compared: {cal.workbooks}; refused: {len(cal.refused)}",
        f"- Formula cells compared: {t.cells}; agree bit-exact: {t.verdicts['pass']} "
        f"({_pct(t.verdicts['pass'], t.cells)}); volatile cells skipped: {cal.volatile_skipped}",
        "",
        "| verdict | cells |",
        "|---|---|",
        *(f"| {v} | {n} |" for v, n in sorted(t.verdicts.items(), key=lambda kv: -kv[1])),
        "",
        "Functions with disagreements (a cell counts once for each function it calls):",
        "",
        "| function | cells | disagree | verdicts |",
        "|---|---|---|---|",
    ]
    worst = sorted(
        ((n, s) for n, s in cal.by_function.items() if s.disagreements),
        key=lambda ns: (-ns[1].disagreements, ns[0]),
    )
    for name, s in worst:
        detail = ", ".join(f"{v} {c}" for v, c in sorted(s.verdicts.items()) if v != "pass")
        lines.append(f"| {name} | {s.cells} | {s.disagreements} | {detail} |")
    agreeing = sum(1 for s in cal.by_function.values() if not s.disagreements)
    lines += ["", f"Functions agreeing on every cell: {agreeing}.", ""]
    if cal.refused:
        lines += ["Refused workbooks:", ""]
        lines += [f"- `{w}`: {why}" for w, why in sorted(cal.refused.items())]
        lines.append("")
    lines += [
        f"Disagreeing cells (first {examples}):",
        "",
        "| workbook | cell | formula | Excel cached | oracle | verdict |",
        "|---|---|---|---|---|---|",
    ]
    for d in cal.disagreements[:examples]:
        formula = d["formula"].replace("|", "\\|")
        lines.append(
            f"| {d['workbook']} | {d['sheet']}!{d['cell']} | `={formula}` | {_value(d['cached'])} "
            f"| {_value(d['recalculated'])} | {d['verdict'] or 'missing'} |"
        )
    return "\n".join(lines) + "\n"


def _value(v: dict[str, Any] | None) -> str:
    if v is None:
        return "—"
    text = repr(v.get("v")) if v.get("t") != "empty" else "(empty)"
    return text.replace("|", "\\|")
