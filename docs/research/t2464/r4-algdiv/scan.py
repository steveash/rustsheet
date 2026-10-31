"""T-3383 r4-algdiv: per-b lnB pin of the density under T-2513's model M2 (mode "m2"), on this worker's probes.

For each shape (a, b): scan L = lnb_base(a, b) (r2-bratio's TOMS algdiv in doubles) moved by j ulps, j in [-W, W];
report the best j, its hits, how many j tie, the accurate lnB's position (in ulps from lnb_base) and the implied
algdiv offset: fl(best L - GL(a)) against the accurate algdiv = lnG(b) - lnG(a+b), in ulps of algdiv.
Usage: scan.py a|hold     Prints one line per shape and a summary."""

import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r3-m1-residual"))
import m1lib as M  # noqa: E402
from lscan import scan  # noqa: E402

mp = M.mpmath


def rows_of(name):
    meta = json.loads((HERE / f"probe_{name}.meta.json").read_text())
    o = M.read_sidecar(meta["case"])
    rows, cut = [], 0
    for i, (x, a, b) in enumerate(meta["rows"]):
        r = i + 1
        if (o.get(f"A{r}"), o.get(f"B{r}"), o.get(f"C{r}")) != (x, a, b):
            cut += 1
            continue
        if o.get(f"D{r}") is None:
            continue
        rows.append(dict(x=x, a=a, b=b, excel=o[f"D{r}"]))
    return rows, cut


def accurate(a, b):
    """(lnB, algdiv) at 50 digits; workdps because the x87 emulation elsewhere lowers mpmath's global precision."""
    with mp.workdps(50):
        lb, lab = mp.loggamma(b), mp.loggamma(mp.mpf(a) + b)
        return +(mp.loggamma(a) + lb - lab), +(lb - lab)


def main(name, W=16):
    rows, cut = rows_of(name)
    M.ensure_gl(rows)
    res = scan(rows, "m2", W)
    tot0 = totb = n = full = 0
    out = []
    for (a, b), (cnt, hits, L0) in res.items():
        best = max(hits.values())
        ties = [j for j, h in hits.items() if h == best]
        jb = min(ties, key=abs)
        lnb, alg = accurate(a, b)
        jacc = M.ulp_steps(L0, float(lnb))
        gl = M.BE.GAMMALN(a)
        Lb = M.nb(L0, jb)
        imp = float(mp.mpf(Lb) - mp.mpf(gl))  # algdiv implied by the pinned lnB (exact difference, rounded)
        ulp_alg = mp.mpf(abs(M.nb(float(alg), 1) - float(alg)))
        d_imp = float((mp.mpf(imp) - alg) / ulp_alg)
        d_toms = float((mp.mpf(L0) - mp.mpf(gl) - alg) / ulp_alg)
        tot0 += hits[0]
        totb += best
        n += cnt
        full += best == cnt
        out.append((a, b, cnt, hits[0], jb, best, ties, jacc, d_toms, d_imp))
        print(
            f"a={a:<5} b={b:<9} n={cnt:3d} TOMS-d {hits[0]:3d}  best j={jb:+3d} {best:3d} ties={len(ties):2d} "
            f"[{min(ties):+d},{max(ties):+d}]  accurate j={jacc:+3d}  algdiv-acc in ulp(algdiv): TOMS-d {d_toms:+6.2f} "
            f"pinned {d_imp:+6.2f}"
        )
    print(f"{name}: rows {n} (literal-cut dropped {cut}); TOMS-d {tot0}; per-shape best {totb}; shapes at 100% "
          f"{full} of {len(res)}")
    (HERE / f"scan_{name}.json").write_text(json.dumps(out) + "\n")


if __name__ == "__main__":
    main(sys.argv[1])
