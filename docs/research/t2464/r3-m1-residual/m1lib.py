"""T-2513 r3-m1-residual: shared helpers for closing the misses of Excel's beta density model M1 (min(a,b) <= 1).

Base model: r2-bratio's `bratio_excel.Kernel(BEST).density` (w1's M1) with r2-log1p's Excel log1p patched in for
x <= 0.375 (`BASE`). Everything else imports the round-1/2 modules; nothing is copied.

  pieces(x, a, b)  -> (tx, ty) as stored doubles (M1's products, Excel LN / log1p)
  combine(tx, ty, L, a, b, mode) -> Excel's EXP of the exponent sum (mode "m1": E = tx + (ty - L) extended, b = 1
                      stored; other modes for the sum scans)
  loaders: old pdf cells (w1 beta_exact), w1 probe1, r2-log1p's BETA.DIST(p,1,2^k+1) rows with BINOM-pinned l,
           and this worker's probes (t2513-r3-m1-residual-*).
  score(models, excel) -> (n, exact, w4, w15)
Run with /home/user/rustsheet/bindings/python/.venv/bin/python.
"""

from __future__ import annotations

import importlib.util
import json
import math
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
T2464 = HERE.parent
REPO = T2464.parents[2]
SUITE = REPO / "eval" / "suites" / "probes-numeric" / "cases"
for d in (T2464, T2464 / "r2-bratio", T2464 / "r2-log1p", T2464 / "w1-density"):
    if str(d) not in sys.path:
        sys.path.append(str(d))

import mpmath  # noqa: E402

import bratio_excel as BE  # noqa: E402
import excel_log1p as XLP  # noqa: E402
import excelmath as em  # noqa: E402

W1 = BE.W1  # w1-density lib (exp_v6, ln_x87, load_pdf, load_probe, within15, ulpd)
sx = BE.sx
mpf = mpmath.mpf
fl64, fl53 = em.fl64, em.fl53

# r2-log1p's Excel log1p in place of bratio_excel's correctly rounded stand-in (x <= 0.375 only uses it).
BE.log1p_cr = XLP.log1p_series
KER = BE.Kernel(BE.BEST)
# Excel GAMMALN values fetched for this worker's shapes go to a local cache (bratio_excel's own stays untouched).
BE._GLF = HERE / "gammaln_cache.pkl"
if BE._GLF.exists():
    import pickle

    BE._GL.update({k: v for k, v in pickle.loads(BE._GLF.read_bytes()).items() if k not in BE._GL})


def _probed_gl():
    """Excel's own GAMMALN at this worker's shape arguments (probe t2513-r3-m1-residual-gl) override the model."""
    f = SUITE / "t2513-r3-m1-residual-gl.oracle.json"
    if not f.exists():
        return {}
    o = json.loads(f.read_text())["sheets"]["Calc"]
    args = json.loads((HERE / "probe_gl.args.json").read_text())
    got = {}
    for i, v in enumerate(args):
        e, g = o.get(f"A{i + 1}", {}), o.get(f"B{i + 1}", {})
        if e.get("v") == v and g.get("t") == "n":
            got[v] = g["v"]
    return got


BE._GL.update(_probed_gl())


def ensure_gl(rows) -> int:
    """Evaluate lnB for every shape of rows once so missing GAMMALN arguments are recorded, then fetch them."""
    for a, b in {(r["a"], r["b"]) for r in rows}:
        KER.lnb_m1(a, b)
    return BE.fetch_missing()


def base_density(x: float, a: float, b: float) -> float:
    """M1 as integrated in r2-bratio, with Excel's log1p: the base model of this worker."""
    return KER.density(x, a, b)[0]


def lnb_base(a: float, b: float) -> float:
    return float(KER.lnb_m1(a, b))


def ly_of(x: float) -> float:
    return XLP.log1p_series(-x) if x <= 0.375 else BE.LN(1.0 - x)


def pieces(x: float, a: float, b: float) -> tuple[float, float]:
    tx = sx("*", sx("-", a, 1.0), BE.LN(x))
    ty = sx("*", sx("-", b, 1.0), ly_of(x))
    return tx, ty


def combine(tx: float, ty: float, L, a: float, b: float, mode: str = "m1") -> float:
    """Excel's EXP of the exponent. L may be a double or an extended (mpf) value."""
    T, Y, Lm = mpf(tx), mpf(ty), mpf(L)
    if mode == "m1":
        e = mpf(fl53(T - Lm)) if b == 1.0 else fl64(T + fl64(Y - Lm))
    elif mode == "ext":  # always extended, b = 1 too
        e = fl64(T + fl64(Y - Lm))
    elif mode == "xy_first":  # (tx + ty) - L
        e = fl64(fl64(T + Y) - Lm)
    elif mode == "x_first":  # (tx - L) + ty
        e = fl64(fl64(T - Lm) + Y)
    elif mode == "m2":  # T-2513 model M2: fl(EXP(hi) * fl(1 + lo)), hi = fl(fl(tx + ty) - L), lo = E - hi exactly
        hi = mpf(fl53(mpf(fl53(T + Y)) - Lm))
        return fl53(mpf(W1.exp_v6(hi)) * mpf(fl53(1 + ((T + Y - Lm) - hi))))
    elif mode in ("ddf", "dde"):  # T-2513: EXP(hi) times the double fl(1 + lo) ("ddf") or EXP(fl(lo)) ("dde")
        if b == 1.0:
            return W1.exp_v6(mpf(fl53(T - Lm)))
        hi = mpf(fl53(mpf(fl53(T + Y)) - Lm))
        lo = (T + Y - Lm) - hi
        f = mpf(fl53(1 + lo)) if mode == "ddf" else mpf(W1.exp_v6(mpf(fl53(lo))))
        return fl53(mpf(W1.exp_v6(hi)) * f)
    elif mode in ("dd_e2", "dd_e1"):  # lo from one rounding only: e2 = error of fl(s - L), e1 = error of s = fl(tx+ty)
        if b == 1.0:
            return W1.exp_v6(mpf(fl53(T - Lm)))
        s_ = mpf(fl53(T + Y))
        hi = mpf(fl53(s_ - Lm))
        lo = ((s_ - Lm) - hi) if mode == "dd_e2" else ((T + Y) - s_)
        return fl53(mpf(W1.exp_v6(hi)) * (1 + lo))
    elif mode in ("dd", "dd_x", "dd_l"):  # double exponent hi plus its exact remainder lo (T-2513 r3-m1-residual)
        if b == 1.0:
            return W1.exp_v6(mpf(fl53(T - Lm)))
        if mode == "dd":
            hi = mpf(fl53(mpf(fl53(T + Y)) - Lm))
        elif mode == "dd_x":
            hi = mpf(fl53(T + mpf(fl53(Y - Lm))))
        else:
            hi = mpf(fl53(mpf(fl53(T - Lm)) + Y))
        return fl53(mpf(W1.exp_v6(hi)) * (1 + ((T + Y - Lm) - hi)))
    elif mode == "exact":
        e = T + Y - Lm
    else:
        raise ValueError(mode)
    return W1.exp_v6(e)


def nb(x: float, k: int) -> float:
    d = math.inf if k > 0 else -math.inf
    for _ in range(abs(k)):
        x = math.nextafter(x, d)
    return x


def ulp_steps(x: float, y: float) -> int:
    """Signed distance y - x in ulps of doubles (same sign region)."""
    import struct

    def key(v):
        i = struct.unpack("<q", struct.pack("<d", v))[0]
        return i if i >= 0 else -(i & 0x7FFFFFFFFFFFFFFF)

    return key(y) - key(x)


# ---------------------------------------------------------------------------------------------- data
def load_old():
    return [r for r in W1.load_pdf() if min(r["a"], r["b"]) <= 1.0]


def load_probe1():
    return [r for r in W1.load_probe("probe1") if min(r["a"], r["b"]) <= 1.0]


def load_log1p_rows():
    """r2-log1p: C = BETA.DIST(p,1,n+1,FALSE), n = 2^k, with l pinned from BINOM.DIST(0,n,p) (ty = n*l exact)."""
    import pin  # r2-log1p

    import r2lib

    rows = []
    for nm in ("runs", "grid", "hold"):
        o = r2lib.read_oracle(nm)
        for i, (p, n, k, l) in enumerate(pin.load(nm)):
            c = o.get(f"C{i + 1}")
            if k in ("none", "multi") or not isinstance(c, float):
                continue
            rows.append(dict(case=nm, cell=f"C{i + 1}", x=p, a=1.0, b=float(n + 1), l=l, excel=c))
    return rows


def read_sidecar(case_id: str) -> dict:
    d = json.loads((SUITE / f"{case_id}.oracle.json").read_text())
    return {k: v.get("v") for k, v in d["sheets"]["Calc"].items() if v.get("t") == "n"}


def load_own(name: str):
    """Rows of this worker's probe t2513-r3-m1-residual-<name> (meta written by mkprobe.py)."""
    meta = json.loads((HERE / f"probe_{name}.meta.json").read_text())
    o = read_sidecar(f"t2513-r3-m1-residual-{name}")
    rows = []
    for i, (x, a, b) in enumerate(meta["rows"]):
        r = i + 1
        ex = o.get(f"D{r}")
        if ex is None:
            continue
        # the evaluated arguments, read back (literal-cut check)
        if (o.get(f"A{r}"), o.get(f"B{r}"), o.get(f"C{r}")) != (x, a, b):
            continue
        rows.append(dict(case=name, cell=f"D{r}", x=x, a=a, b=b, excel=ex, blk=meta["blk"][i]))
    return rows


# ---------------------------------------------------------------------------------------------- scoring
def score(models, excel):
    n = ex = w4 = w15 = 0
    for m, e in zip(models, excel):
        n += 1
        ex += m == e
        w4 += W1.ulpd(m, e) <= 4
        w15 += W1.within15(m, e)
    return n, ex, w4, w15


def fmt(s) -> str:
    n, ex, w4, w15 = s
    return f"{ex:5d}/{w4:5d}/{w15:5d} of {n:5d}"


def lit(x: float) -> str:
    """Exact formula fragment for x (r2-log1p's two-piece literal; integers written plainly)."""
    import r2lib

    return str(int(x)) if x == int(x) and abs(x) < 2**31 else r2lib.lit(x)
