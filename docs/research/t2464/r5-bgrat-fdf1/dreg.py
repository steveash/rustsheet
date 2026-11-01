"""T-3567 r5-bgrat-fdf1 dreg: which stored intermediate makes D's sawtooth? On the split probe's two runs of 1,024
consecutive doubles ((30.7, 0.7) at x = 0.8117, (40, 0.03) at x = 0.95), regress Excel's relative residual
eps = (D - F)/F in units of 2^-53 (F the exact I at Excel's stored lnx) on the relative rounding error of candidate
stored doubles (each in units of 2^-53), with a constant and a slope in k. A real stored intermediate gets a
significant coefficient near its analytic sensitivity, and the residual sd falls toward the final rounding's floor."""
import math, pickle, sys
from collections import defaultdict
from pathlib import Path
import mpmath
import numpy as np
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r3-bgrat"))
import bscore  # noqa: E402
from common import split_cells, xlk  # noqa: E402
bx = xlk.bx
mpmath.mp.dps = 40
U = 2.0 ** -53
C = {(c["a"], c["b"], c["x"]): c for c in bscore.cells()}
runs = defaultdict(list)
for a, b0, x, _case, d in split_cells():
    if (a, b0) in ((30.7, 0.7), (40.0, 0.030000000000000027)):
        runs[(a, b0)].append((x, d, C[(a, b0, x)]))
def rel(stored, exact):
    return float((mpmath.mpf(stored) - exact) / exact) / U
def ab(stored, exact):
    return float(mpmath.mpf(stored) - exact) / U
for (a, b0), pts in runs.items():
    pts.sort()
    nu = a + 0.5 * (b0 - 1)
    rows, eps = [], []
    for k, (x, d, c) in enumerate(pts):
        lnx = c["lnx"]; L = mpmath.mpf(lnx)
        F = c["F"]
        eps.append(float((mpmath.mpf(d) - F) / F) / U)
        z_e = -mpmath.mpf(nu) * L
        z = -nu * lnx
        lz = bx.LN(z)
        feats = {
            "z=fl(-nu lnx)": rel(z, z_e),
            "LN(z) abs": ab(lz, mpmath.log(z)),
            "b*LN(z) abs": ab(b0 * lz, b0 * mpmath.mpf(lz)),
            "a*lnx abs": ab(a * lnx, a * L),
            "0.5bm1*lnx abs": ab(0.5 * (b0 - 1) * lnx, mpmath.mpf(0.5 * (b0 - 1)) * L),
            "lnx vs ln(x) abs": ab(lnx, mpmath.log(x)),
            "y=fl(1-x)": rel(1 - x, 1 - mpmath.mpf(x)),
        }
        rows.append(feats)
    names = list(rows[0])
    e = np.array(eps)
    k = np.arange(len(e)) / len(e)
    print(f"\n(a, b0) = ({a}, {b0}), n = {len(e)}: eps mean {e.mean():+.2f} sd {e.std():.2f}")
    base = np.column_stack([np.ones_like(k), k])
    r0 = e - base @ np.linalg.lstsq(base, e, rcond=None)[0]
    print(f"  const + slope only: residual sd {r0.std():.3f}")
    for nm in names:
        col = np.array([r[nm] for r in rows])
        X = np.column_stack([base, col])
        coef, *_ = np.linalg.lstsq(X, e, rcond=None)
        res = e - X @ coef
        print(f"  + {nm:18s} coef {coef[2]:+8.3f}  feat sd {col.std():6.3f}  residual sd {res.std():.3f}")
    X = np.column_stack([base] + [np.array([r[nm] for r in rows]) for nm in names])
    coef, *_ = np.linalg.lstsq(X, e, rcond=None)
    res = e - X @ coef
    print("  all:", " ".join(f"{nm.split('=')[0].split()[0]}={c:+.2f}" for nm, c in zip(names, coef[2:])), f"residual sd {res.std():.3f}")
    # structure of what is left: lag autocorrelations and a smoothed trend
    for lag in (1, 2, 4, 8, 16, 64):
        print(f"  lag {lag:3d} autocorr {np.corrcoef(res[:-lag], res[lag:])[0, 1]:+.3f}")
    sm = np.convolve(res, np.ones(64) / 64, mode="valid")
    print(f"  64-point moving mean: sd {sm.std():.3f} (white noise would give {res.std() / 8:.3f})")
    F = np.abs(np.fft.rfft(res - res.mean()))
    top = np.argsort(F)[-5:][::-1]
    print("  top FFT bins (cycles per run):", ", ".join(f"{t}:{F[t] / F.mean():.1f}x" for t in top))
