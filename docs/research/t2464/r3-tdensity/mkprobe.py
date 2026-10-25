"""T-2513 r3-tdensity probes: T.DIST(t, n, FALSE) on exact-input runs with Excel's own inner columns beside it.

Each row: A = t (w2-forensics' exact encoder, so no 15-digit literal truncation), and
  B  =T.DIST(A,n,FALSE)
  C  =A*A
  D  =POWER(1+A*A/n,-h)            h = (n+1)/2
  E  =POWER(n+A*A,-h)
  F  =BETA.DIST(n/(n+A*A),n/2,0.5,FALSE)
  G  =F.DIST(A*A,1,n,FALSE)
  H  =BETA.DIST(A*A/(n+A*A),0.5,n/2,FALSE)
  I  =POWER(n/(n+A*A),h)
Blocks: runs of consecutive doubles from an irregular start t0 (t*t rounds), runs t = c + k 2^-20 (t*t and n + t*t
exact), and df steps at fixed t.
  a1, a2  in-sample (df 3, 4, 5 | 7, 10, 30 and the df steps 3..100)
  hold    hold-out, built and filled with a1/a2 before any tuning: df 6, 8, 12, 20 and 3, 4 at other t, df 101..300
  hold2   second hold-out (A and B only): df 9, 11, 14, 16, 24, 28, 41, 53
Usage: mkprobe.py [--write] [name ...]   (writes the cases into eval/suites/probes-numeric/cases and <name>.meta.json here)"""

from __future__ import annotations

import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.append(str(HERE.parent / "w2-forensics"))
import make_probes as MP  # noqa: E402  (w2-forensics: exact encoder)

COLS = {
    "B": "excel",
    "A": "t_rb",
    "C": "tt",
    "D": "pow1q",
    "E": "pownu",
    "F": "betay",
    "G": "fdist",
    "H": "betaz",
    "I": "powy",
}


def runs(dfs, t0s, cnt, exact_c=(), exact_cnt=0):
    out = []
    i = 0
    for n in dfs:
        for t0 in t0s:
            x0 = t0 * (1 + 0.000137 * i + 0.0000731)
            i += 1
            out.append(
                (
                    f"run(n{n:g},t{t0:g})",
                    n,
                    MP.run(x0, 0, cnt),
                )
            )
        for c in exact_c:
            out.append((f"ex(n{n:g},t{c:g})", n, [c + k * 2.0**-20 for k in range(exact_cnt)]))
    return out


def dfsteps(ts, dfs):
    return [(f"df(t{t:g})", n, [t]) for t in ts for n in dfs]


PROBES = {
    "a1": lambda: runs((3, 4, 5), (0.3125, 1.7, 4.3), 96, (1.5,), 64),
    "a2": lambda: (
        runs((7, 10, 30), (0.3125, 1.7, 4.3), 96, (1.5,), 64) + dfsteps((0.5, 0.9, 2.0), range(3, 101))
    ),
    "hold": lambda: (
        runs((6, 8, 12, 20, 3, 4), (0.43, 1.23, 2.9, 7.7), 48) + dfsteps((1.37,), range(101, 301, 2))
    ),
    # second hold-out (columns A, B only), built after the first hold-out was read and before the stirlerr-table
    # constant (LT1e) was scored; its dfs are used nowhere else
    "hold2": lambda: runs((9, 11, 14, 16, 24, 28, 41, 53), (0.77, 2.2, 5.9), 40),
}
MINIMAL = {"hold2"}


def row_cells(r, t, n, minimal=False):
    h = (n + 1) / 2
    A = f"A{r}"
    hs = f"{h:g}"
    if minimal:
        return {A: "=" + MP.enc(t), f"B{r}": f"=T.DIST({A},{n},FALSE)"}
    return {
        A: "=" + MP.enc(t),
        f"B{r}": f"=T.DIST({A},{n},FALSE)",
        f"C{r}": f"={A}*{A}",
        f"D{r}": f"=POWER(1+{A}*{A}/{n},-{hs})",
        f"E{r}": f"=POWER({n}+{A}*{A},-{hs})",
        f"F{r}": f"=BETA.DIST({n}/({n}+{A}*{A}),{n / 2:g},0.5,FALSE)",
        f"G{r}": f"=F.DIST({A}*{A},1,{n},FALSE)",
        f"H{r}": f"=BETA.DIST({A}*{A}/({n}+{A}*{A}),0.5,{n / 2:g},FALSE)",
        f"I{r}": f"=POWER({n}/({n}+{A}*{A}),{hs})",
    }


def build(name):
    cells = {}
    meta = []
    r = 0
    for blk, n, ts in PROBES[name]():
        for t in ts:
            r += 1
            cells.update(row_cells(r, t, n, name in MINIMAL))
            meta.append(dict(row=r, blk=blk, t=t, n=float(n)))
    assert len(cells) <= 16384, len(cells)
    case = {
        "format": "rustsheet-case/v1",
        "description": f"T-2513 r3-tdensity probe {name}: T.DIST(t,n,FALSE) on exact-input t runs and df steps, with "
        "Excel's inner columns (t*t, POWER(1+t*t/n,-h), POWER(n+t*t,-h), BETA.DIST(y,n/2,0.5,FALSE), "
        "F.DIST(t*t,1,n,FALSE), BETA.DIST(z,0.5,n/2,FALSE), POWER(y,h)) beside it.",
        "tags": ["M9", "probe", "statistics", "student-t", "T-2513"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    cols = {"B": "excel", "A": "t_rb"} if name in MINIMAL else COLS
    return case, dict(case=f"t2513-r3-tdensity-{name}", cols=cols, rows=meta)


if __name__ == "__main__":
    tot = 0
    for name in PROBES:
        if len(sys.argv) > 2 and name not in sys.argv[2:]:
            continue
        c, m = build(name)
        k = len(c["sheets"][0]["cells"])
        tot += k
        print(name, len(m["rows"]), "rows", k, "cells")
        if "--write" in sys.argv:
            (MP.CASES / f"t2513-r3-tdensity-{name}.case.json").write_text(json.dumps(c, indent=1) + "\n")
            (HERE / f"{name}.meta.json").write_text(json.dumps(m) + "\n")
    print("total", tot)
