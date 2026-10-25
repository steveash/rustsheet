"""T-2464 r2-density-large: shared helpers for Excel's BETA.DIST density at a, b > 1.

Data: every BETA.DIST(x,a,b,FALSE) cell with a > 1 and b > 1 from w4-fractions' data.tsv (beta_exact pdf + w2 runs
and sweeps, exact at 50 digits) and w1-density's probe1 sheets (exact recomputed here at 160 bits), plus this
directory's own probes (probe_*.out.json). Reuses w1-density's lib (EXP V6 / LN mpmath models, x87 rounding, scorer)
and r2-log1p's Excel log1p model by import.
"""

from __future__ import annotations

import csv
import hashlib
import json
import math
import pickle
import sys
from pathlib import Path

HERE = Path(__file__).parent
T = HERE.parent
for p in (T, T / "w1-density", T / "r2-log1p"):
    if str(p) not in sys.path:
        sys.path.append(str(p))

import mpmath  # noqa: E402

import excelmath  # noqa: E402
import lib as W1  # noqa: E402  (w1-density/lib.py)

mpmath.mp.prec = 160
mpf = mpmath.mpf
R64 = W1.R64
R53 = W1.R53
D = W1.D
exp_v6 = W1.exp_v6
ln64 = W1.ln64
score = W1.score
within15 = W1.within15
ulpd = W1.ulpd


def LN(x) -> float:
    """Excel's LN (FLDLN2/FYL2X, stored)."""
    return W1.ln_x87(x)


def log1p_xl(t: float) -> float:
    """Excel's ln(1+t) for -0.375 <= t < 0 (r2-log1p's atanh-series model); LN(fl(1+t)) otherwise."""
    import excel_log1p

    if -0.375 <= t < 0:
        return excel_log1p.log1p_series(t)
    return LN(1.0 + t)


# ---------------------------------------------------------------- Excel GAMMALN, cached here
_GC = HERE / "gl_cache.pkl"
_gl: dict = pickle.loads(_GC.read_bytes()) if _GC.exists() else {}


def gl_prefetch(vs):
    need = sorted({float(v) for v in vs if float(v) not in _gl and v > 0})
    for i in range(0, len(need), 20000):
        ch = need[i : i + 20000]
        for v, r in zip(ch, excelmath.excel("GAMMALN", ch)):
            _gl[v] = r
    if need:
        _GC.write_bytes(pickle.dumps(_gl))


def GL(v: float) -> float:
    v = float(v)
    if v not in _gl:
        gl_prefetch([v])
    return _gl[v]


# ---------------------------------------------------------------- rounding modes
def rnd(v, mode: str):
    """'x' stored x87 op fl53(fl64), 'e' extended register fl64, 'd' SSE double fl53, 'm' exact."""
    if mode == "x":
        return mpf(D(v))
    if mode == "e":
        return R64(v)
    if mode == "d":
        return mpf(R53(v))
    return mpf(v)


# ---------------------------------------------------------------- data
def _exact_pdf(x, a, b):
    X, A, B = mpf(x), mpf(a), mpf(b)
    return X ** (A - 1) * (1 - X) ** (B - 1) / mpmath.beta(A, B)


def holdout(blk: str, x: float, a: float, b: float) -> bool:
    """True for the hold-out half (md5 of the point, odd)."""
    h = hashlib.md5(f"{blk}|{x!r}|{a!r}|{b!r}".encode()).digest()
    return bool(h[0] & 1)


_DC = HERE / "rows_cache.pkl"


def load(include_probes: bool = True):
    """Rows (dicts: src, blk, x, a, b, excel, exact (mpf), hold) for a > 1, b > 1."""
    if _DC.exists():
        rows = pickle.loads(_DC.read_bytes())
    else:
        rows = []
        for r in csv.DictReader(open(T / "w4-fractions" / "data.tsv"), delimiter="\t"):
            if r["pdf"] in ("", "nan", "None"):
                continue
            a, b = float(r["a"]), float(r["b"])
            if a > 1 and b > 1:
                blk = "old" if r["src"] == "old" else r["blk"]
                rows.append(
                    dict(
                        src=r["src"],
                        blk=blk,
                        x=float(r["x"]),
                        a=a,
                        b=b,
                        excel=float(r["pdf"]),
                        exact=mpf(r["pdfx"]),
                    )
                )
        p1 = json.load(open(T / "w1-density" / "probe" / "probe1.meta.json"))
        out = json.load(open(T / "w1-density" / "probe" / "probe1.out.json"))["sheets"]
        for j, (a, b) in enumerate(p1["conf"]):
            if not (a > 1 and b > 1):
                continue
            sh = out[f"C{j}"]
            for k, x in enumerate(p1["xs"]):
                c = sh.get(f"B{k + 1}")
                if c and c.get("t") == "n":
                    rows.append(
                        dict(
                            src="probe1",
                            blk=f"p1({a:g},{b:g})",
                            x=x,
                            a=a,
                            b=b,
                            excel=float(c["v"]),
                            exact=_exact_pdf(x, a, b),
                        )
                    )
        _DC.write_bytes(pickle.dumps(rows))
    if include_probes:
        rows = rows + load_own_probes()
    for r in rows:
        r["hold"] = holdout(r["blk"], r["x"], r["a"], r["b"])
    return rows


def load_own_probes():
    """This directory's probes: probe_<name>.meta.json (list of [blk, x, a, b] per formula row) + .out.json."""
    rows = []
    for meta in sorted(HERE.glob("probe_*.meta.json")):
        outp = meta.with_name(meta.name.replace(".meta.json", ".out.json"))
        cache = meta.with_name(meta.name.replace(".meta.json", ".rows.pkl"))
        if not outp.exists():
            continue
        if cache.exists():
            rows += pickle.loads(cache.read_bytes())
            continue
        m = json.load(open(meta))
        sheets = json.load(open(outp))["sheets"]
        got = []
        for sheet, cell, blk, x, a, b in m["rows"]:
            c = sheets.get(sheet, {}).get(cell)
            if c and c.get("t") == "n":
                got.append(
                    dict(
                        src=meta.stem, blk=blk, x=x, a=a, b=b, excel=float(c["v"]), exact=_exact_pdf(x, a, b)
                    )
                )
        cache.write_bytes(pickle.dumps(got))
        rows += got
    return rows


def err_units(v: float, exact) -> float:
    """Relative error of v in units of 2^-53."""
    return float((mpf(v) - exact) / exact * mpf(2) ** 53)


def tally(pairs):
    n = ex = w4 = w15 = 0
    for m, e in pairs:
        n += 1
        ex += m == e
        w4 += ulpd(m, e) <= 4
        w15 += within15(m, e)
    return n, ex, w4, w15


def fmt(t):
    n, ex, w4, w15 = t
    return f"{ex:5d}/{w4:5d}/{w15:5d} of {n:5d}"


def blocks(rows):
    out: dict = {}
    for i, r in enumerate(rows):
        out.setdefault(r["blk"], []).append(i)
    return out


def report(rows, preds, label=""):
    """Print exact/w4/w15 per block and overall, plus the hold-out half."""
    print(f"== {label}")
    for blk, idx in sorted(blocks(rows).items()):
        r0 = rows[idx[0]]
        print(
            f"  {blk:14s} ({r0['a']:g},{r0['b']:g}) model {fmt(tally((preds[i], rows[i]['excel']) for i in idx))}"
            f"   CR {fmt(tally((float(rows[i]['exact']), rows[i]['excel']) for i in idx))}"
        )
    allp = [(preds[i], rows[i]["excel"]) for i in range(len(rows))]
    hold = [(preds[i], rows[i]["excel"]) for i in range(len(rows)) if rows[i]["hold"]]
    print(
        f"  ALL   model {fmt(tally(allp))}   CR {fmt(tally((float(r['exact']), r['excel']) for r in rows))}"
    )
    print(f"  HOLD  model {fmt(tally(hold))}")


def block_need(rows, idx, xpart):
    """Per fixed-shape block: the exact Loader shape constant Cex = -ln B + k ln(k/n) + (n-k) ln((n-k)/n), and the
    offset dC Excel needs on top of it (mean Excel error minus mean error of EXP(fl(Cex + X)), units of 2^-53).
    xpart(row) gives the model's X part. Shared by c1_need.py and c2_const.py."""
    import statistics as st

    r0 = rows[idx[0]]
    a, b = r0["a"], r0["b"]
    A, B = mpmath.mpf(a), mpmath.mpf(b)
    k, n = A - 1, A + B - 2
    nk = n - k
    cex = -mpmath.log(mpmath.beta(A, B)) + k * mpmath.log(k / n) + nk * mpmath.log(nk / n)
    me = [err_units(rows[i]["excel"], rows[i]["exact"]) for i in idx]
    mm = [err_units(W1.exp_v6(mpmath.mpf(W1.D(cex + xpart(rows[i])))), rows[i]["exact"]) for i in idx]
    return a, b, k, n, nk, cex, st.mean(me) - st.mean(mm), st.pstdev(me)


def toms_betaln_module():
    """w1-density's TOMS betaln port, wired to this directory's GAMMALN cache."""
    sys.path.append(str(T / "w1-density"))
    import toms_betaln

    toms_betaln.GL = GL
    return toms_betaln
