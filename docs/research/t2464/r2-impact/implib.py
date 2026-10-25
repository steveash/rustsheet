"""T-2464 r2-impact: shared code. The eval report's per-cell verdicts, the visibly-off buckets, the cell -> kernel
mapping (w5's argument forms) and the model runner (r2-bratio's `bratio_excel`, imported, never copied).

Run every script with /home/user/rustsheet/bindings/python/.venv/bin/python from this directory.
Inputs:
  REPORT   an eval report.json (default: the run this worker made, $R2I_REPORT overrides)
  cells_all.tsv  every single-call oracle cell of the incomplete-beta functions with resolved arguments
                 (../extract.py with ONLY="", git-ignored here; rebuild: cd tools && uv run python
                 ../docs/research/t2464/extract.py "" r2-impact/cells_all.tsv)
"""

from __future__ import annotations

import csv
import decimal
import math
import os
import pickle
import re
import sys
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
T2464 = HERE.parent
REPO = T2464.parents[2]
REPORT = Path(
    os.environ.get(
        "R2I_REPORT",
        "/tmp/claude-0/-home-user-rustsheet/77027746-e589-5f95-baa2-1e5785f680d3"
        "/scratchpad/evalrun/report.json",
    )
)
sys.path.insert(0, str(T2464 / "r2-bratio"))
sys.path.insert(0, str(T2464 / "r2-log1p"))

# Functions built on the incomplete beta (or on its inverse). T.DIST(.,FALSE), F.DIST(.,FALSE) and the BINOM /
# NEGBINOM pmfs are densities, not the kernel; they are listed under their own family below.
_FAMILIES = {
    "beta": "BETA.DIST BETADIST", "f": "F.DIST F.DIST.RT FDIST", "f-test": "F.TEST FTEST",
    "t": "T.DIST T.DIST.RT T.DIST.2T TDIST", "t-test": "T.TEST TTEST", "binom": "BINOM.DIST BINOMDIST",
    "binom-range": "BINOM.DIST.RANGE", "negbinom": "NEGBINOM.DIST NEGBINOMDIST",
    "inv": "BETA.INV BETAINV F.INV F.INV.RT FINV T.INV T.INV.2T TINV BINOM.INV CRITBINOM CONFIDENCE.T",
}
FAMILY = {fn: fam for fam, fns in _FAMILIES.items() for fn in fns.split()}
_CALL = re.compile(r"(?:_xlfn\.)?([A-Z][A-Z0-9.]*)\s*\(", re.I)
UNGATED = {"probes-numeric", "gen-m8a", "gen-m8b"}


def first_family_fn(formula: str):
    """The first incomplete-beta function called in a formula (outermost-first in reading order)."""
    for m in _CALL.finditer(formula or ""):
        name = m.group(1).upper()
        if name in FAMILY:
            return name
    return None


def within15(e: float, a: float) -> bool:
    """crates/rustsheet-eval/src/closeness.rs within_fifteen_digits, transcribed."""
    if not (math.isfinite(e) and math.isfinite(a)):
        return False
    gap = abs(a - e)
    if e == 0.0:
        return gap == 0.0
    exp10 = decimal.Decimal(repr(abs(e))).adjusted()
    return gap <= float(f"5e{exp10 - 15}")


def ulps(a: float, e: float) -> float:
    if a == e:
        return 0.0
    if not (math.isfinite(a) and math.isfinite(e)) or e == 0:
        return math.inf
    return abs(a - e) / math.ulp(e)


def bucket(verdict: str, e, a) -> str:
    """closeness.rs `bucket`: pass / ulp1 / ulp4 / digits15 / visible_numeric / visible_other."""
    if verdict in ("pass", "deviation"):
        return "pass"
    if verdict == "numeric_ulp":
        return "ulp1" if ulps(a, e) <= 1 else "ulp4"
    if verdict in ("numeric_close", "numeric_mismatch"):
        if isinstance(e, float) and isinstance(a, float) and within15(e, a):
            return "digits15"
        return "visible_numeric"
    return "visible_other" if verdict != "unsupported" else "unsupported"


def is_visible(b: str) -> bool:
    return b.startswith("visible")


def score3(pairs) -> tuple[int, int, int, int]:
    """[(model, excel)] -> (n, bit-exact, within 4 ulp, within 15 digits)."""
    n = ex = u4 = v = 0
    for m, e in pairs:
        n += 1
        ex += m == e
        u4 += ulps(m, e) <= 4
        v += within15(e, m)
    return n, ex, u4, v


def _num(v):
    return float(v["v"]) if isinstance(v, dict) and v.get("t") == "n" else v


def failing_cells(path: Path = REPORT):
    """Every non-passing cell of the report: dict(case, suite, key 'Sheet!A1', formula, excel, engine, verdict,
    bucket, fn). Only failing cells are listed in report.json; passing cells are bit-exact by definition."""
    import json

    rep = json.loads(path.read_text())
    out = []
    for c in rep["cases"]:
        for cell in c.get("cells", []):
            e, a = _num(cell["expected"]), _num(cell["actual"])
            out.append(
                dict(
                    case=c["id"],
                    suite=c["suite"],
                    key=f"{cell['sheet']}!{cell['cell']}",
                    formula=cell.get("formula", ""),
                    excel=e,
                    engine=a,
                    verdict=cell["verdict"],
                    bucket=bucket(cell["verdict"], e, a),
                    fn=first_family_fn(cell.get("formula", "")),
                )
            )
    return out, rep


def _arg(s: str):
    return True if s == "True" else False if s == "False" else float(s)


def all_cells():
    """cells_all.tsv rows: dict(fn, case, key, args (tuple), excel)."""
    rows = []
    with open(HERE / "cells_all.tsv") as f:
        for r in csv.DictReader(f, delimiter="\t"):
            try:
                ex = float(r["excel"])
            except ValueError:
                continue
            rows.append(
                dict(
                    fn=r["fn"],
                    case=r["case"],
                    key=r["cell"],
                    args=tuple(_arg(s) for s in r["args"].split("|")),
                    excel=ex,
                )
            )
    return rows


# ---------------------------------------------------------------------------------------------- cell -> kernel
# A problem is ("cdf", a, b, x, y, side, post) or ("pdf", x, a, b, scale), or ("none", reason).
# post maps the kernel tail to the cell: "id", "half" (T: L = I/2), "one_minus_half" (T.DIST(t > 0) = 1 - L),
# "twice_half" (T.DIST.2T = 2L), "centre*" (L = 0.5 - H/2, w5 2.3), "times" (TDIST tails).
def _tr(v):
    return float(math.trunc(v))


def problem(fn: str, args: tuple):
    n_args = len(args)
    if fn in ("BETA.DIST", "BETADIST"):
        x, a, b = args[0], args[1], args[2]
        cum = True if fn == "BETADIST" else bool(args[3]) if n_args > 3 else None
        rest = args[3:] if fn == "BETADIST" else args[4:]
        lo = rest[0] if len(rest) > 0 else 0.0
        hi = rest[1] if len(rest) > 1 else 1.0
        if cum is None or not (a > 0 and b > 0 and lo < hi and lo <= x <= hi):
            return ("none", "beta-args")
        t = x if (lo, hi) == (0.0, 1.0) else (x - lo) / (hi - lo)
        if not 0.0 < t < 1.0:
            return ("none", "beta-edge")
        if cum:
            return ("cdf", a, b, t, 1.0 - t, "lower", "id" if (lo, hi) == (0.0, 1.0) else "id-ab")
        return ("pdf", t, a, b, hi - lo)
    if fn in ("F.DIST", "F.DIST.RT", "FDIST"):
        X, d1, d2 = args[0], _tr(args[1]), _tr(args[2])
        if fn == "F.DIST" and not (n_args > 3 and args[3] is True):
            return ("none", "f-pdf")
        if not (X > 0 and 1 <= d1 < 1e10 and 1 <= d2 < 1e10):
            return ("none", "f-edge")
        p = d1 * X
        x, y = 1 / (1 + d2 / p), 1 / (1 + p / d2)  # w5 2.3: both formed directly
        return ("cdf", d1 / 2, d2 / 2, x, y, "lower" if fn == "F.DIST" else "upper", "id")
    if fn in ("T.DIST", "T.DIST.RT", "T.DIST.2T", "TDIST"):
        t, n = args[0], _tr(args[1])
        if fn == "T.DIST" and not (n_args > 2 and args[2] is True):
            return ("none", "t-pdf")
        if not (1 <= n < 1e10) or t == 0:
            return ("none", "t-edge")
        if fn == "T.DIST":
            post = "L" if t < 0 else "1-L"
        elif fn == "T.DIST.RT":
            post = "L" if t > 0 else "1-L"
        elif fn == "T.DIST.2T":
            if t < 0:
                return ("none", "t-edge")
            post = "2L"
        else:
            tails = _tr(args[2]) if n_args > 2 else 0
            if tails not in (1, 2) or t < 0:
                return ("none", "t-edge")
            post = "L" if tails == 1 else "2L"
        u = abs(t)
        tt = u * u
        s = n + tt
        y, z = n / s, tt / s
        if n == 2 or (u > 1 and y <= 0.99):
            # I_y(n/2, 1/2) = 2L with y = fl(n/fl(n + t t)), complement fl(1 - y) (w5 2.3)
            return ("cdf", n / 2, 0.5, y, 1.0 - y, "lower", "T:" + post)
        # centre: L = 0.5 - I_z(1/2, n/2)/2, complement y (w5 2.3; statistics.md 19 of 23)
        return ("cdf", 0.5, n / 2, z, y, "lower", "Tc:" + post)
    if fn in ("BINOM.DIST", "BINOMDIST"):
        if not (n_args > 3 and args[3] is True):
            return ("none", "binom-pmf")
        k, n, p = math.trunc(args[0]), math.trunc(args[1]), args[2]
        if not (0 < p < 1 and 0 <= k < n):
            return ("none", "binom-edge")
        if k == 0:
            return ("none", "binom-k0")  # BINOM's own shortcut EXP(fl(n l)), w5/r2-log1p
        return ("cdf", float(k + 1), float(n - k), p, 1.0 - p, "upper", "id")
    if fn in ("NEGBINOM.DIST", "NEGBINOMDIST"):
        if fn == "NEGBINOM.DIST" and not (n_args > 3 and args[3] is True):
            return ("none", "negbinom-pmf")
        if fn == "NEGBINOMDIST":
            return ("none", "negbinom-pmf")
        f, r, p = math.trunc(args[0]), math.trunc(args[1]), args[2]
        if not (0 < p < 1 and f >= 0 and r >= 1):
            return ("none", "negbinom-edge")
        return ("cdf", float(r), float(f + 1), p, 1.0 - p, "lower", "id")
    return ("none", "inverse" if FAMILY.get(fn) == "inv" else "other")


def finish_cdf(post: str, tail: float) -> float:
    """Map a kernel tail onto the cell value (w5 2.3, 2.4)."""
    if post in ("id", "id-ab"):
        return tail
    kind, op = post.split(":")
    if kind == "T":
        L = tail / 2
    else:  # centre form, L = fl(0.5 - H/2)
        L = 0.5 - tail / 2
    return {"L": L, "1-L": 1.0 - L, "2L": 2 * L}[op]


# ---------------------------------------------------------------------------------------------- models
def configs():
    """(name, Cfg, use_excel_log1p). The r2-bratio attribution ladder, then r2-log1p's log1p in the density."""
    import bratio_excel as bx
    from dataclasses import replace

    R, T = bx.R_LIKE, bx.TOMS_XL
    s1 = replace(T, closed=True)
    s2 = replace(s1, bpser="w3", w3_z="z53", w3_lg="acc")
    s3 = replace(s2, w3_z="z64")
    s4 = replace(s3, w3_lg="toms")
    s5 = replace(s4, brcmp1="pdf")
    s6 = replace(s5, bup_arith="w4")
    s7 = replace(s6, brcomp="pdf")
    assert s7 == bx.BEST, (s7, bx.BEST)
    return [
        ("R_LIKE", R, False),
        ("TOMS_XL", T, False),
        ("+closed", s1, False),
        ("+w3bpser", s2, False),
        ("+z64", s3, False),
        ("+w3lg", s4, False),
        ("+brcmp1pdf", s5, False),
        ("+w4arith", s6, False),
        ("BEST", s7, False),
        ("BEST+log1p", s7, True),
        # one-piece ablations of BEST
        ("BEST-closed", replace(s7, closed=False), False),
        ("BEST-bpser", replace(s7, bpser="toms"), False),
        ("BEST-brcmp1", replace(s7, brcmp1="toms"), False),
        ("BEST-brcomp", replace(s7, brcomp="toms"), False),
    ]


def set_log1p(excel: bool) -> None:
    """Swap the density's log1p(-x) between the correctly rounded stand-in and r2-log1p's Excel model."""
    import bratio_excel as bx

    if not hasattr(bx, "_cr_log1p_orig"):
        bx._cr_log1p_orig = bx.log1p_cr
    if excel:
        import excel_log1p as el

        bx.log1p_cr = el.log1p_series
    else:
        bx.log1p_cr = bx._cr_log1p_orig


def evaluate(problems, cfg, excel_log1p: bool):
    """Model cell values for [(key, problem)] -> {key: (value, branch)}. GAMMALN values the model lacks are fetched
    from the harness into this directory's cache (bratio_excel's own cache file is redirected here)."""
    import bratio_excel as bx

    bx._GLF = HERE / "gammaln_cache.pkl"
    if bx._GLF.exists():
        for v, r in pickle.loads(bx._GLF.read_bytes()).items():
            bx._GL.setdefault(v, r)
    set_log1p(excel_log1p)
    kern = bx.Kernel(cfg)
    for _ in range(4):
        out = {}
        for k, p in problems:
            if p[0] == "cdf":
                w, w1, br = bx.kernel(p[1], p[2], p[3], p[4], cfg)
                out[k] = (finish_cdf(p[6], w if p[5] == "lower" else w1), br)
            elif p[0] == "pdf":
                v, modelled = kern.density(p[1], p[2], p[3])
                out[k] = (v / p[4] if p[4] != 1.0 else v, "pdf:M1" if modelled else "pdf:CR-standin")
        if not bx._GL_MISSING:
            return out
        bx.fetch_missing()
    return out


def cache(name: str, fn):
    path = HERE / f"{name}.pkl"
    if path.exists():
        return pickle.loads(path.read_bytes())
    val = fn()
    path.write_bytes(pickle.dumps(val))
    return val


def tally_by(rows, key):
    c = Counter()
    for r in rows:
        c[key(r)] += 1
    return c
