"""Noise budget per direct x-run block: sd of Excel's rel err vs sd of candidate models' errs (detrended by mean):
CR (one rounding), all-ext with Excel LN (z, exp, product in extended), stored chain EXP(fl(fl(a*LN x)-L))/a*fl(S).
Models with fewer roundings have smaller sd; Excel's sd brackets how many stored roundings it has."""
from lib import *
M = mpmath.mpf
blocks = ["xruns-1:0", "xruns-1:1", "xruns-1:2", "xruns-1:3", "xruns-1:4", "xruns-1:5", "xruns-1:6", "xruns-1:7", "xruns-2:3", "xruns-2:4"]
R = load(cum=True, branch="direct")
print(f"{'block':10s} {'a':>6s} {'b':>6s}  sd: excel   CR  all-ext(LN dbl)  stored-chain   corr(excel, all-ext) corr(excel, chain)")
for blk in blocks:
    rows = sorted([r for r in R if r["block"] == blk], key=lambda r: r["x"])
    x = np.array([r["x"] for r in rows]); A = rows[0]["a"]; B = rows[0]["b"]; mA = M(A)
    E = np.array([r["excel"] for r in rows])
    I = [exact_cdf(xi, A, B) for xi in x]; lnB = mpmath.log(mpmath.beta(mA, M(B)))
    Sx = [i / (M(xi) ** mA / mA / mpmath.exp(lnB)) for i, xi in zip(I, x)]
    err = rel_units(E, I)
    cr = rel_units([float(v) for v in I], I)
    lnx = xl("LN", x)
    ext = [fl53(mpmath.exp(fl64(fl64(mA * M(l)) - lnB)) / mA * s) for l, s in zip(lnx, Sx)]
    e_ext = rel_units(ext, I)
    L = float(lnB)
    ch = mul(div(xl("EXP", sub(mul(A, lnx), L)), A), np.array([float(s) for s in Sx]))
    e_ch = rel_units(ch, I)
    print(f"{blk:10s} {A:6g} {B:6g}  {err.std():9.2f} {cr.std():5.2f} {e_ext.std():10.2f} {e_ch.std():14.2f}   {np.corrcoef(err, e_ext)[0,1]:+.2f}  {np.corrcoef(err, e_ch)[0,1]:+.2f}")
save_exact()
