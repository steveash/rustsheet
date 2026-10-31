"""T-3509: the gamma-family held-out probe's per-region report, against Excel and the 50-digit math.

Reads, for each eval/suites/probes-numeric/cases/t3509-blind-*.case.json:
- the case (formulas, whose literal arguments place each cell in a region);
- its oracle sidecar (Excel's values, `rustsheet-oracle fill -b excel-graph`);
- its math sidecar (50-digit mpmath values, `rustsheet-accuracy math --suite probes-numeric
  --filter t3509-blind-`);
- a `rustsheet-eval run` report.json, which lists every cell that is not bit-exact with the engine's
  value (any other scored cell is Excel's value).

Usage, from tools/ (the uv project, for the formula reader):
    RUSTSHEET_EVAL_BIN=<binary> ... ; <binary> run --suite probes-numeric --filter t3509-blind- --out DIR
    uv run python ../docs/research/t3509/score.py DIR/report.json [--cells] > ../docs/research/t3509/score-<rev>.out

A region is function x shape bucket x (x/shape or p) bucket. For each region, three comparisons count
the cells in four exclusive classes, as the eval does (decision 0083):
- `=` bit-exact; `4u` within 4 ulp (not exact); `15d` within half a unit of the 15th significant
  digit (not within 4 ulp); `off` visibly off (more, or a different type or error);
- ours vs Excel (the target), ours vs math (the double nearest the 50-digit value) and Excel vs math
  (how accurate Excel itself is there: a region where Excel is `=`/`4u` against the math and we are
  not is an `accuracy` gap; where Excel is far from the math it is a `visible` one, decision 0083);
- `outA`: our cells outside Gate A's envelope (neither within 15 digits of Excel nor within 1e-9
  relative of the math, floor 1e-300; an error against a number either way; decision 0188 R2).
`--cells` appends the visibly-off cells (ours vs Excel) of each region, with the three values.
"""

from __future__ import annotations

import argparse
import json
import math
import struct
import sys
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from fractions import Fraction
from pathlib import Path
from typing import Any

from rustsheet_tools.accuracy.eval_math import math_path, single_call
from rustsheet_tools.fitlab.sweep import split_args

ROOT = Path(__file__).resolve().parents[3]
CASES = ROOT / "eval" / "suites" / "probes-numeric" / "cases"
PREFIX = "t3509-blind-"
EPSILON, FLOOR = 1e-9, 1e-300  # Gate A (decision 0188 R2)
CLASSES = ("=", "4u", "15d", "off")
SHAPES = [(0.5, "<0.5"), (2.5, "0.5-2.5"), (20, "2.5-20"), (1e3, "20-1E3"), (math.inf, ">=1E3")]
RATIOS = [
    (1e-6, "<1E-6"),
    (1e-3, "1E-6..1E-3"),
    (0.5, "1E-3..0.5"),
    (1.4, "0.5..1.4"),
    (10, "1.4..10"),
    (math.inf, ">=10"),
]
PROBS = [
    (1e-100, "p<1E-100"),
    (1e-10, "1E-100..1E-10"),
    (0.5, "1E-10..0.5"),
    (1 - 1e-6, "0.5..1-1E-6"),
    (2, ">=1-1E-6"),
]
Value = tuple[str, Any]  # ("n", float) | ("e", "#NUM!") | ...


def bucket(v: float, edges: list[tuple[float, str]]) -> str:
    """The label of the first edge above `v`."""
    return next(label for edge, label in edges if v < edge)


LABELS = ["<0"] + [label for _, label in SHAPES + RATIOS + PROBS] + ["-"]


def order(key: tuple[str, str, str]) -> tuple[str, int, int]:
    """Sorts regions by function, then by their buckets' numeric order."""
    return (key[0], LABELS.index(key[1]), LABELS.index(key[2]))


def shape_bucket(v: float) -> str:
    return "<0" if v < 0 else bucket(v, SHAPES)


def region(fn: str, args: list[str]) -> tuple[str, str, str]:
    """(function label, shape bucket, x/shape or p bucket) of one cell."""
    a = [x.strip().upper() for x in args]
    if fn == "GAMMA.DIST":
        x, shape = float(a[0]), float(a[1])
        return (
            "GAMMA.DIST " + ("cdf" if a[3] == "TRUE" else "pdf"),
            shape_bucket(shape),
            bucket(x / shape, RATIOS),
        )
    if fn == "CHISQ.DIST.RT":
        x, df = float(a[0]), float(a[1])
        return (fn, shape_bucket(df / 2), bucket(x / df, RATIOS))
    if fn == "GAMMA.INV":
        return (fn, shape_bucket(float(a[1])), bucket(float(a[0]), PROBS))
    if fn == "CHISQ.INV.RT":
        return (fn, shape_bucket(float(a[1]) / 2), bucket(float(a[0]), PROBS))
    if fn == "POISSON.DIST":
        k, mean = float(a[0]), float(a[1])
        return (
            "POISSON " + ("cdf" if a[2] == "TRUE" else "pmf"),
            shape_bucket(k + 1),
            bucket(mean / (k + 1), RATIOS),
        )
    return (fn, shape_bucket(float(a[0])), "-")  # GAMMALN, GAMMA: the shape column is x


def ordered(x: float) -> int:
    """The double's position on the number line, so ulp distance is a difference."""
    bits: int = struct.unpack("<q", struct.pack("<d", x))[0]
    return bits if bits >= 0 else -(bits & 0x7FFFFFFFFFFFFFFF)


def within_15(actual: float, expected: float) -> bool:
    """Half a unit of the 15th significant digit of `expected` (the eval's visibly-off test)."""
    if expected == 0 or not math.isfinite(expected):
        return actual == expected
    half = float(f"5e{math.floor(math.log10(abs(expected))) - 15}")
    return abs(Fraction(actual) - Fraction(expected)) <= Fraction(half)


def classify(actual: Value, expected: Value) -> str:
    """One of CLASSES."""
    if actual[0] != "n" or expected[0] != "n":
        return "=" if actual == expected else "off"
    a, e = float(actual[1]), float(expected[1])
    if a == e:
        return "="
    if math.isfinite(a) and math.isfinite(e) and abs(ordered(a) - ordered(e)) <= 4:
        return "4u"
    return "15d" if within_15(a, e) else "off"


def gate_a_inside(ours: Value, excel: Value, truth: Value | None) -> bool:
    """Decision 0188 R2's envelope (an error against a number either way is outside)."""
    if ours[0] != "n" or excel[0] != "n":
        return ours == excel
    if within_15(float(ours[1]), float(excel[1])):
        return True
    if truth is None or truth[0] != "n":
        return False
    t = float(truth[1])
    return abs(float(ours[1]) - t) <= max(EPSILON * abs(t), FLOOR)


def shown(v: Value | None) -> str:
    """A value for the cell list: a number's repr, an error's code, or `none`."""
    if v is None:
        return "none"
    return repr(v[1]) if v[0] == "n" else str(v[1])


def as_value(cell: dict[str, Any]) -> Value:
    """A report or sidecar cell as (type, value)."""
    t = cell.get("t")
    if t == "n":
        return ("n", float(cell["v"]))
    if t == "e":
        return ("e", cell.get("v", cell.get("why")))
    return (str(t), cell.get("v"))


@dataclass
class Row:
    """One region's counts."""

    n: int = 0
    vs_excel: Counter[str] = field(default_factory=Counter)
    vs_math: Counter[str] = field(default_factory=Counter)
    excel_math: Counter[str] = field(default_factory=Counter)
    out_a: int = 0
    no_math: int = 0

    def add(self, other: Row) -> None:
        """Adds another region's counts (for the per-function totals)."""
        self.n += other.n
        self.out_a += other.out_a
        self.no_math += other.no_math
        self.vs_excel.update(other.vs_excel)
        self.vs_math.update(other.vs_math)
        self.excel_math.update(other.excel_math)

    def line(self, key: tuple[str, str, str]) -> str:
        """The region's table line."""

        def counts(c: Counter[str]) -> str:
            return " ".join(f"{c[k]:>4}" for k in CLASSES)

        return (
            f"{key[0]:<17}{key[1]:<9}{key[2]:<15}{self.n:>4} | {'':13}{counts(self.vs_excel)} "
            f"| {'':12}{counts(self.vs_math)} | {'':13}{counts(self.excel_math)} | {self.out_a:>4} {self.no_math:>6}"
        )


Regions = dict[tuple[str, str, str], Row]
OffCells = dict[tuple[str, str, str], list[str]]


def engine_values(report: Path) -> dict[tuple[str, str, str], Value]:
    """(case file name, sheet, A1) -> the engine's value, for every cell the report lists."""
    data = json.loads(report.read_text(encoding="utf-8"))
    out: dict[tuple[str, str, str], Value] = {}
    for case in data["cases"]:
        name = case["id"].rsplit("/", 1)[-1]
        if name.startswith(PREFIX):
            for c in case.get("cells", []):
                out[(name, c["sheet"], c["cell"])] = as_value(c["actual"])
    return out


def sidecar(path: Path, sheet: str) -> dict[str, Any]:
    """One sheet of an oracle or math sidecar ({} when there is none)."""
    if not path.exists():
        return {}
    cells: dict[str, Any] = json.loads(path.read_text(encoding="utf-8"))["sheets"].get(sheet, {})
    return cells


def collect(engine: dict[tuple[str, str, str], Value]) -> tuple[Regions, OffCells, int]:
    """Every probe cell's classes, by region; the visibly-off cells; the cells left out."""
    rows: Regions = defaultdict(Row)
    off_cells: OffCells = defaultdict(list)
    missing = 0
    for case_path in sorted(CASES.glob(f"{PREFIX}*.case.json")):
        sheet = json.loads(case_path.read_text(encoding="utf-8"))["sheets"][0]
        oracle_path = case_path.with_name(case_path.name.replace(".case.json", ".oracle.json"))
        oracle = sidecar(oracle_path, sheet["name"])
        maths = sidecar(math_path(oracle_path), sheet["name"])
        for a1, text in sheet["cells"].items():
            call = single_call(text)
            if call is None or a1 not in oracle:
                missing += 1
                continue
            excel = as_value(oracle[a1])
            ours = engine.get((case_path.name, sheet["name"], a1), excel)
            truth = as_value(maths[a1]) if a1 in maths and maths[a1].get("f") == text else None
            key = region(call[0], split_args(call[1]))
            row = rows[key]
            row.n += 1
            versus = classify(ours, excel)
            row.vs_excel[versus] += 1
            if truth is None:
                row.no_math += 1
            else:
                row.vs_math[classify(ours, truth)] += 1
                row.excel_math[classify(excel, truth)] += 1
            row.out_a += not gate_a_inside(ours, excel, truth)
            if versus == "off":
                off_cells[key].append(
                    f"{case_path.name} S!{a1} {text}  Excel {shown(excel)}  ours {shown(ours)}  math {shown(truth)}"
                )
    return rows, off_cells, missing


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("report", type=Path, help="report.json of a run over the t3509-blind- cases")
    parser.add_argument("--cells", action="store_true", help="list the visibly-off cells of each region")
    args = parser.parse_args()
    rows, off_cells, missing = collect(engine_values(args.report))
    head = " ".join(f"{c:>4}" for c in CLASSES)
    print(
        f"{'function':<17}{'shape':<9}{'x/shape or p':<15}{'n':>4} | ours vs Excel {head} | ours vs math {head} "
        f"| Excel vs math {head} | outA nomath"
    )
    totals: dict[str, Row] = defaultdict(Row)
    last = None
    for key in sorted(rows, key=order):
        if last is not None and key[0] != last:
            print()
        last = key[0]
        totals[key[0]].add(rows[key])
        print(rows[key].line(key))
    print("\nper function")
    for fn, total in sorted(totals.items()):
        print(total.line((fn, "all", "all")))
    if missing:
        print(f"\n{missing} cells without an oracle value or not one call", file=sys.stderr)
    if args.cells:
        print("\nvisibly-off cells (ours vs Excel), by region")
        for key in sorted(off_cells, key=order):
            print(f"\n{key[0]} | shape {key[1]} | {key[2]} ({len(off_cells[key])})")
            for entry in off_cells[key]:
                print(f"  {entry}")


if __name__ == "__main__":
    main()
