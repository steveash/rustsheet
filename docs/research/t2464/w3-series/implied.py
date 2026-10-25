"""Implied series S_impl = Excel / P for the direct cdf, P = EXP(fl(fl(a*LN x) - lnB))/a (lnB = GL form
GLa+(GLb-GLab) for min(a,b)<1 and max(a,b)<=8, exact otherwise). delta = S_impl/S_exact - 1 (units 2^-53).
Per (a,b) group: n, mean, sd and range of delta, and the slope of delta against ln x (a prefactor-exponent error
shows as a slope: d delta/d ln x = error in a, a ln y term gives slope in ln y)."""
import collections
from lib import *
M = mpmath.mpf
rows = [r for r in load(cum=True, branch="direct") if r["a"] != 1.0 and r["b"] != 1.0 and r["src"] == "be"]
g = collections.defaultdict(list)
for r in rows:
    g[(r["a"], r["b"])].append(r)
out = []
for (A, B), rs in sorted(g.items()):
    if len(rs) < 3:
        continue
    x = np.array([r["x"] for r in rs]); E = np.array([r["excel"] for r in rs])
    I = [exact_cdf(xi, A, B) for xi in x]
    lnBe = mpmath.log(mpmath.beta(M(A), M(B)))
    Sx = [i / (M(xi) ** M(A) / M(A) / mpmath.exp(lnBe)) for i, xi in zip(I, x)]
    if min(A, B) < 1 and max(A, B) <= 8:
        ga, gb, gab = xl("GAMMALN", [A, B, add(A, B)])
        L = add(ga, sub(gb, gab)); Lname = "GL"
    else:
        L = float(lnBe); Lname = "ex"
    alx = mul(A, xl("LN", x)); P = div(xl("EXP", sub(alx, L)), A)
    d = np.array([float((M(e) / M(p)) / s - 1) / U for e, p, s in zip(E, P, Sx)])
    ln_x = np.log(x); ln_y = np.log1p(-x)
    sl = np.polyfit(ln_x, d, 1)[0] if np.ptp(ln_x) > 0 else 0
    err = rel_units(E, I)
    out.append((A, B, len(rs), Lname, d.mean(), d.std(), d.min(), d.max(), sl, err.std(), x.min(), x.max()))
print(f"{'a':>7s} {'b':>7s} {'n':>3s} lnB   mean_d   sd_d   min_d   max_d  slope/lnx  sd(excel err)  x range")
for o in out:
    print(f"{o[0]:7g} {o[1]:7g} {o[2]:3d} {o[3]:4s} {o[4]:+8.2f} {o[5]:6.2f} {o[6]:+7.1f} {o[7]:+7.1f} {o[8]:+9.2f} {o[9]:9.2f}   {o[10]:.3g}-{o[11]:.3g}")
save_exact()
