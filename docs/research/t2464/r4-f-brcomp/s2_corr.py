"""Per run: sd of (Excel - exact) and (model - exact) in units, and their correlation (does the model's noise track
Excel's?). Models: plain brcomp/x (TOMS gamln) and its swap variants."""
import statistics as S
import sys

import mpmath

import fblib as B

SET = sys.argv[1] if len(sys.argv) > 1 else "a"
VARIANTS = {"bratio": {"gamln": "toms"}, "noswap": {"gamln": "toms", "swap": "none"},
            "always": {"gamln": "toms", "swap": "always"}}
rows = B.load(SET)
runs: dict = {}
for r in rows:
    runs.setdefault(r["blk"], []).append(r)


def u(v, ex):
    return float((mpmath.mpf(v) - ex) / ex * mpmath.mpf(2) ** 53)


def corr(p, q):
    try:
        return S.correlation(p, q)
    except Exception:
        return float("nan")


print(f"# {SET}: per run sd(Excel-exact), then per model sd(model-exact) and corr with Excel")
for blk, rr in runs.items():
    e = [u(r["excel"], r["exact"]) for r in rr]
    out = [f"{blk:22s} xl sd {S.pstdev(e):6.1f}"]
    for k, o in VARIANTS.items():
        m = [u(B.model(r["x"], r["d1"], r["d2"], o), r["exact"]) for r in rr]
        out.append(f"{k} {S.pstdev(m):6.1f} r={corr(e, m):+.2f}")
    print(" | ".join(out))
