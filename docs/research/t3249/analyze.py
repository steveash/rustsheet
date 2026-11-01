"""T-3249: the engine's error against Excel on the sweep, without fitting. Reads `rustsheet-eval show --all` for both
cases and the meta files, and prints per case and column:
  - by the run's |ln f| level (as the generator placed it; 0 is the mode run): cells, sd and mean of
    (ours - Excel) / Excel in units of 2^-52, exact matches, cells beyond 15 significant digits (decision 0114);
  - by shape: n = a + b - 2, the same columns.
Usage (repo root): python3 docs/research/t3249/analyze.py"""

from __future__ import annotations

import math
import re
import statistics
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).parent
sys.path.insert(0, str(HERE))
import mkprobe as G  # noqa: E402

CELL = re.compile(r"^Calc!([AB])(\d+)\s")
VAL = re.compile(r"^\s+(expected|actual)\s*:\s*(\S+)")


def cells(case: str) -> dict[tuple[str, int], tuple[float, float]]:
    out = subprocess.run(
        ["./target/release/rustsheet-eval", "show", f"probes-numeric/{case}.case.json", "--all"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    res: dict[tuple[str, int], tuple[float, float]] = {}
    key, vals = None, {}
    for line in out.splitlines():
        m = CELL.match(line)
        if m:
            key, vals = (m[1], int(m[2])), {}
            continue
        m = VAL.match(line)
        if m and key:
            try:
                vals[m[1]] = float(m[2])
            except ValueError:
                vals[m[1]] = math.nan
            if len(vals) == 2:
                res[key] = (vals["expected"], vals["actual"])
    return res


def off15(excel: float, ours: float) -> bool:
    if excel == ours:
        return False
    if not (math.isfinite(excel) and math.isfinite(ours)) or excel == 0:
        return True
    return abs(ours - excel) > 0.5 * 10 ** (math.floor(math.log10(abs(excel))) - 14)


def table(title: str, groups: dict) -> None:
    print(f"  {title}: key cells sd mean exact off15")
    for k in sorted(groups):
        es = groups[k]
        u = [e for e, _ in es if math.isfinite(e)]
        sd = statistics.pstdev(u) if len(u) > 1 else 0.0
        mean = statistics.fmean(u) if u else 0.0
        print(
            f"    {k!s:>14} {len(es):5d} {sd:7.1f} {mean:7.1f} {sum(e == 0 for e, _ in es):5d} "
            f"{sum(o for _, o in es):5d}"
        )


def main() -> None:
    for name in G.PROBES:
        case = G.case_id(name)
        got = cells(case)
        rows = []
        for label, a, b, x0, k0, cnt in G.blocks(name):
            level = 0 if label.endswith("mode") else -int(label.rsplit("L", 1)[1])
            rows += [(x, a, b, level) for x in G.MP.run(x0, k0, cnt)]
        for col in "AB":
            by_level, by_shape = defaultdict(list), defaultdict(list)
            for i, (_, a, b, level) in enumerate(rows, 1):
                if (col, i) not in got:
                    continue
                excel, ours = got[(col, i)]
                err = (ours - excel) / excel * 2**52 if excel and math.isfinite(ours) else math.nan
                by_level[level].append((err, off15(excel, ours)))
                by_shape[(a + b - 2, a, b)].append((err, off15(excel, ours)))
            n = sum(len(v) for v in by_level.values())
            print(f"{case} column {col} ({'BETA.DIST' if col == 'A' else 'F.DIST'}): {n} cells")
            table("|ln f| level (0: the mode run)", by_level)
            table("(n, a, b)", by_shape)


if __name__ == "__main__":
    main()
