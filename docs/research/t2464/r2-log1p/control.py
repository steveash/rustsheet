"""Positive control: synthesise BINOM.DIST(0,2^k,p) = Excel EXP(2^k*l) from a planted log1p on the grid+hold p
sets, run the same inversion (r2lib.invert_binom) and score both the planted and the rival model.
Planted kernels: the final forward power sum (excel_log1p) and the Horner variant (gen_scan 'S N=12').
Usage: control.py"""
import excel_log1p as XL
import gen_scan as G
import make_probes as MP
import r2lib as R

horner = G.CANDS["S N=12"]


def k_horner(a):
    return horner(R.r_of(-a))


KERNELS = {"fwd power sum": XL.log1p_series, "Horner S N=12": k_horner}

if __name__ == "__main__":
    ps = [p for nm in ("grid", "hold") for p in MP.sets(nm) if 1e-4 <= p <= 0.375]
    ns = [R.binom_n(p) for p in ps]
    for planted, f in KERNELS.items():
        vals = R.xl("EXP", [n * f(-p) for p, n in zip(ps, ns)])
        hits = R.invert_binom(ps, ns, vals)
        pinned = [R.nb(R.cr_log1p(-p), h[0]) if len(h) == 1 else None for p, h in zip(ps, hits)]
        ok = sum(x is not None for x in pinned)
        print(f"planted {planted}: pinned uniquely {ok}/{len(ps)}; recovered == planted "
              f"{sum(x == f(-p) for x, p in zip(pinned, ps))}")
        for nm, g in KERNELS.items():
            print(f"    scored model {nm:15s}: {sum(x == g(-p) for x, p in zip(pinned, ps))}/{len(ps)}")
