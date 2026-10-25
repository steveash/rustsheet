"""s1: per run (r > 0.375), scan K_hi (the double inside S1 = fl(K_hi + H)) and fit delta_tot = Ktot - K_exact
(units) by interval overlap: S2 = fl(fl(K_hi + H) + P), E = S2 + (Ktot - K_hi).
Printed per run: n, best count, the K_hi offsets t = (K_hi - K_exact) units that reach it (as ranges), the
delta_tot range there, M-sd's delta, t of fl(K_exact) and fl(K_sd), and whether those reach the best count.
--plant KHI_T DTOT: replace Excel's values by the model's with K_hi = fl(K_exact + KHI_T units), Ktot = K_exact +
DTOT units (positive control).  Usage: s1_scan.py SET [--plant t d] [--min-n N] [--branch log|ser|all]"""

import sys

import mpmath
import numpy as np

import clib as C

U, mpf = C.U, C.mpf


def ulp_units(v: float) -> float:
    return float(np.spacing(abs(np.float64(v)))) * 2.0**53 if v != 0 else 2.0**-1074


def scan_run(cells, m, n, side, quiet=False):
    Kref = C.K_exact(m, n, side)
    Kd = float(Kref)
    g1 = [ulp_units(C.st(C.LDB(Kd) + pc["H"])) for pc, *_ in cells]
    g2 = [ulp_units(C.S2_of(Kd, pc)) for pc, *_ in cells]
    G = max(max(g1), max(g2))
    step = max(ulp_units(Kd), min(g1) / 32)
    ts = np.arange(-G, G + step / 2, step)
    seen, res = set(), []
    for t in ts:
        Khi = float(Kref + mpf(t) / U)
        if Khi in seen:
            continue
        seen.add(Khi)
        cnt, lo, hi = C.best_overlap(C.ktot_intervals(cells, Khi, Kref))
        res.append((float((C.ex(Khi) - Kref) * U), cnt, lo, hi, Khi))
    best = max(c for _, c, *_ in res)
    return res, best, G


def ranges(res, best):
    out, cur = [], None
    for t, c, lo, hi, _ in sorted(res):
        if c == best:
            cur = [t, t, lo, hi] if cur is None else [cur[0], t, min(cur[2], lo), max(cur[3], hi)]
        elif cur is not None:
            out.append(cur)
            cur = None
    if cur is not None:
        out.append(cur)
    return out


def plant(cells, m, n, side, t, d):
    Kref = C.K_exact(m, n, side)
    Khi, Ktot = float(Kref + mpf(t) / U), Kref + mpf(d) / U
    out = []
    for pc, E0, iv, r in cells:
        v = C.value(r["x"], m, n, Khi, Ktot)
        r2 = dict(r, excel=v)
        iv2 = C.e_interval(r2)
        if iv2 is not None:
            out.append((pc, E0, iv2, r2))
    return out


def main():
    a = sys.argv[1:]
    name = a[0]
    pl = (float(a[a.index("--plant") + 1]), float(a[a.index("--plant") + 2])) if "--plant" in a else None
    minn = int(a[a.index("--min-n") + 1]) if "--min-n" in a else 8
    branch = a[a.index("--branch") + 1] if "--branch" in a else "log"
    for key, rs in sorted(C.runs(C.load(name), branch).items()):
        m, n, side, _ = key
        cells = C.run_cells(rs)
        if len(cells) < minn:
            continue
        if pl:
            cells = plant(cells, m, n, side, *pl)
        res, best, G = scan_run(cells, m, n, side)
        Kref = C.K_exact(m, n, side)
        dsd = C.SD.delta(m, n, side)
        tk = float((C.ex(float(Kref)) - Kref) * U)
        tsd = float((C.ex(float(C.K_sd(m, n, side))) - Kref) * U)
        byK = {r[4]: r[1] for r in res}
        okK = byK.get(float(Kref), -1)
        okS = byK.get(float(C.K_sd(m, n, side)), -1)
        rg = " ".join(
            f"[{t0:+.1f},{t1:+.1f}]d[{lo:+.1f},{hi:+.1f}]" for t0, t1, lo, hi in ranges(res, best)[:4]
        )
        print(
            f"({m:g},{n:g}) s{side} x{rs[0]['x']:.4g} n{len(cells)} best {best} G{G:.0f} | dsd {dsd:+.1f} "
            f"tK {tk:+.1f}:{okK} tSD {tsd:+.1f}:{okS} | {rg}"
        )
        sys.stdout.flush()
    C.save()


if __name__ == "__main__":
    main()
