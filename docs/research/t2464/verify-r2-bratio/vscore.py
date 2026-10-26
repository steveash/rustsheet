"""Independent scorer for the r2-bratio claim. Imports only the candidate kernel (bratio_excel.kernel/run, BEST)
from r2-bratio; data loading, argument parsing, CR reference and scoring are our own (vload.py, here)."""
import hashlib, math, pickle, sys
from collections import Counter, defaultdict
from multiprocessing import Pool
from pathlib import Path

import mpmath

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE.parent / "r2-bratio"))
import vload  # noqa

CRF = HERE / "cr_cache.pkl"


def _cr1(k):
    a, b, x, y, side = k
    with mpmath.workdps(60):
        if side == "lower":
            v = mpmath.betainc(mpmath.mpf(a), mpmath.mpf(b), 0, mpmath.mpf(x), regularized=True)
        else:
            v = mpmath.betainc(mpmath.mpf(b), mpmath.mpf(a), 0, mpmath.mpf(y), regularized=True)
    return k, float(v)


def cr_all(keys):
    cache = pickle.loads(CRF.read_bytes()) if CRF.exists() else {}
    todo = [k for k in keys if k not in cache]
    if todo:
        with Pool(4) as p:
            for k, v in p.imap_unordered(_cr1, todo, chunksize=64):
                cache[k] = v
        CRF.write_bytes(pickle.dumps(cache))
    return cache


def ulpd(m, e):
    if m == e:
        return 0.0
    if not (math.isfinite(m) and math.isfinite(e)) or e == 0:
        return math.inf
    return abs(m - e) / math.ulp(e)


def w15(m, e):
    if m == e:
        return True
    if e == 0 or not math.isfinite(m):
        return False
    # half a unit in the 15th significant digit of e (decimal exponent via repr, not log10 rounding)
    ex = int(f"{abs(e):.16e}".split("e")[1])
    return abs(m - e) <= 0.5 * 10.0 ** (ex - 14)


def tally(pairs):
    n = ex = u4 = v = 0
    for m, e in pairs:
        n += 1; ex += m == e; u4 += ulpd(m, e) <= 4; v += w15(m, e)
    return n, ex, u4, v


def key(r):
    return (r["a"], r["b"], r["x"], r["y"], r["side"])


def model_values(rows, cfg):
    import bratio_excel as bx
    pts = sorted({(r["a"], r["b"], r["x"], r["y"]) for r in rows})
    out = dict(zip(pts, bx.run(pts, cfg)))
    vals, br = [], []
    for r in rows:
        w, w1, b = out[(r["a"], r["b"], r["x"], r["y"])]
        vals.append(w if r["side"] == "lower" else w1); br.append(b.split("/")[0])
    return vals, br


def sha_half(s):
    return int(hashlib.sha1(s.encode()).hexdigest(), 16) % 2 == 0
