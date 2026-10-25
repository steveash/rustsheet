"""T-2464 r2-bratio: the scoring data and the scorer for bratio_excel.py.

Data sets (every row: name, a, b, x, y, side, excel, cr, src, hold):
  be    ../beta_exact.tsv cdf cells (via w4-fractions/data.tsv, which also carries the 50-digit exact value)
  w2f   w2-forensics probes xruns-1, xruns-2, absweep (data.tsv, src w2) and unit-shape (cells_w2f.tsv, exact here)
  w4p1  w4-fractions probe w4p1, column C: BETA.DIST(x, a, b0', TRUE) at the L140 runs (bpser at a > 1, b < 1)
  x29   probe t2464-r2-bratio-x29 (this directory): BETA.DIST cdf at x in [0.29, 0.301), a < 1 < b
  ix    w5-triangulation ix_evidence.tsv rows with confidence direct / derived / closed (F, T, BINOM, NEGBINOM ...)
BETA.DIST rows use the kernel with y = fl(1 - x). ix rows use the x and y w5 mapped. hold = md5 odd half.
Scores: bit-exact / within 4 ulp / within half a unit of the 15th significant digit (w15).
"""

from __future__ import annotations

import csv
import hashlib
import json
import math
import pickle
from collections import defaultdict
from pathlib import Path

import mpmath

HERE = Path(__file__).resolve().parent
T2464 = HERE.parent
_EXF = HERE / "exact_cache.pkl"


def _hold(*key) -> bool:
    return int(hashlib.md5(repr(key).encode()).hexdigest(), 16) % 2 == 1


def _cr(s: str) -> float:
    return float(mpmath.mpf(s))


def _exact_cdf(x: float, a: float, b: float, cache: dict) -> float:
    k = (x, a, b)
    if k not in cache:
        with mpmath.workdps(50):
            cache[k] = float(mpmath.betainc(mpmath.mpf(a), mpmath.mpf(b), 0, mpmath.mpf(x), regularized=True))
    return cache[k]


def load(sets=("be", "w2f", "w4p1", "ix", "x29")):
    rows = []
    cache = pickle.loads(_EXF.read_bytes()) if _EXF.exists() else {}
    n0 = len(cache)
    if "be" in sets or "w2f" in sets:
        for r in csv.DictReader(open(T2464 / "w4-fractions" / "data.tsv"), delimiter="\t"):
            if r["cdf"] in ("nan", ""):
                continue
            name = "be" if r["src"] == "old" else "w2f"
            if name not in sets:
                continue
            x, a, b = float(r["x"]), float(r["a"]), float(r["b"])
            rows.append(
                dict(
                    set=name,
                    blk=r["blk"],
                    a=a,
                    b=b,
                    x=x,
                    y=1.0 - x,
                    side="lower",
                    excel=float(r["cdf"]),
                    cr=_cr(r["cdfx"]),
                    hold=_hold(x, a, b),
                )
            )
    if "w2f" in sets:
        for r in csv.DictReader(open(T2464 / "w2-forensics" / "cells_w2f.tsv"), delimiter="\t"):
            if "unit-shape" not in r["case"]:
                continue
            args = r["args"].split("|")
            x, a, b = float(args[0]), float(args[1]), float(args[2])
            if args[3] != "True" or not 0 < x < 1:
                continue
            rows.append(
                dict(
                    set="w2f",
                    blk="unit-shape",
                    a=a,
                    b=b,
                    x=x,
                    y=1.0 - x,
                    side="lower",
                    excel=float(r["excel"]),
                    cr=_exact_cdf(x, a, b, cache),
                    hold=_hold(x, a, b),
                )
            )
    if "w4p1" in sets:
        meta = json.load(open(T2464 / "w4-fractions" / "probe" / "w4p1.meta.json"))
        out = json.load(open(T2464 / "w4-fractions" / "probe" / "w4p1.out.json"))["sheets"]
        for sh, i, x, a, _b, b0 in meta:
            # The formula parser cuts number literals to 15 significant digits: w4 wrote b0' = 0.5999999999999996
            # (11.6 - 11) as a literal, so Excel evaluated BETA.DIST(x, 7.3, 0.6, TRUE).
            b0 = float(f"{b0:.15g}")
            c = out[sh].get(f"C{i}")
            if not c or c.get("t") != "n":
                continue
            rows.append(
                dict(
                    set="w4p1",
                    blk=f"w4p1:{sh}",
                    a=a,
                    b=b0,
                    x=x,
                    y=1.0 - x,
                    side="lower",
                    excel=float(c["v"]),
                    cr=_exact_cdf(x, a, b0, cache),
                    hold=_hold(x, a, b0),
                )
            )
    if "x29" in sets and (HERE / "t2464-r2-bratio-x29.out.json").exists():
        meta = json.load(open(HERE / "t2464-r2-bratio-x29.meta.json"))
        out = json.load(open(HERE / "t2464-r2-bratio-x29.out.json"))["sheets"]["Calc"]
        for i, x, a, b in meta:
            c = out.get(f"A{i}")
            if c and c.get("t") == "n":
                rows.append(dict(set="x29", blk="x<0.3" if x < 0.3 else "x>=0.3", a=a, b=b, x=x, y=1.0 - x, side="lower",
                                 excel=float(c["v"]), cr=_exact_cdf(x, a, b, cache), hold=_hold(x, a, b)))
    if "ix" in sets:
        for r in csv.DictReader(open(T2464 / "w5-triangulation" / "ix_evidence.tsv"), delimiter="\t"):
            if r["confidence"] not in ("direct", "derived", "closed") or not r["exact"]:
                continue
            x, a, b, y = float(r["x"]), float(r["a"]), float(r["b"]), float(r["y"])
            rows.append(
                dict(
                    set="ix",
                    blk=r["source_fn"],
                    a=a,
                    b=b,
                    x=x,
                    y=y,
                    side=r["side"],
                    excel=float(r["excel_ix"]),
                    cr=_cr(r["exact"]),
                    hold=_hold(r["case"], r["cell"]),
                )
            )
    if len(cache) != n0:
        _EXF.write_bytes(pickle.dumps(cache))
    return rows


def ulpd(m: float, e: float) -> float:
    if m == e:
        return 0.0
    if not (math.isfinite(m) and math.isfinite(e)) or e == 0:
        return math.inf
    return abs(m - e) / math.ulp(e)


def w15(m: float, e: float) -> bool:
    if m == e:
        return True
    if e == 0 or not math.isfinite(m):
        return False
    return abs(m - e) <= 0.5 * 10.0 ** (math.floor(math.log10(abs(e))) - 14)


def tally(pairs):
    """pairs: [(model, excel)] -> (n, exact, w4, w15)."""
    n = ex = f = v = 0
    for m, e in pairs:
        n += 1
        ex += m == e
        f += ulpd(m, e) <= 4
        v += w15(m, e)
    return n, ex, f, v


def side_value(row, res):
    w, w1, _ = res
    return w if row["side"] == "lower" else w1


def table(rows, models: dict, key, title: str, holdout: bool = True):
    """Print per-group exact/w4/w15 for each model (models: name -> list of values aligned with rows) and CR."""
    groups = defaultdict(list)
    for i, r in enumerate(rows):
        groups[key(r)].append(i)
    names = list(models)
    print(f"\n## {title}")
    hdr = "| group | n | " + " | ".join(f"{m} ex/w4/w15" for m in names) + " | CR ex/w4/w15 |"
    if holdout:
        hdr += " " + " | ".join(f"{m} hold-out ex" for m in names) + " | CR hold-out ex |"
    print(hdr)
    print("|" + "---|" * (hdr.count("|") - 1))
    allidx = list(range(len(rows)))
    for g in sorted(groups, key=str) + ["ALL"]:
        idx = allidx if g == "ALL" else groups[g]
        cells = []
        for m in names:
            n, ex, f, v = tally((models[m][i], rows[i]["excel"]) for i in idx)
            cells.append(f"{ex}/{f}/{v}")
        n, ex, f, v = tally((rows[i]["cr"], rows[i]["excel"]) for i in idx)
        line = f"| {g} | {n} | " + " | ".join(cells) + f" | {ex}/{f}/{v} |"
        if holdout:
            hidx = [i for i in idx if rows[i]["hold"]]
            hs = [str(tally((models[m][i], rows[i]["excel"]) for i in hidx)[1]) for m in names]
            hc = tally((rows[i]["cr"], rows[i]["excel"]) for i in hidx)
            line += " " + " | ".join(f"{h}/{len(hidx)}" for h in hs) + f" | {hc[1]}/{len(hidx)} |"
        print(line)
