"""Hold-out on Student t: T.DIST.RT(t, n) for 0 < t <= 1 is 0.5 - BETA.DIST(z, 0.5, n/2, TRUE)/2 with
z = fl(fl(t*t)/fl(n + fl(t*t))) (statistics.md). Score the series candidate for the inner I_z(0.5, n/2):
EXP(fl(fl(0.5*LN z) - L))/0.5 * S (S = bpser ext converged, or exact), L = GL form for n/2 <= 8 else exact lnB;
against the correctly rounded inner value. df 2 (closed form) excluded."""
import collections
from lib import *
from series import bpser_sum
M = mpmath.mpf
rows = []
for line in open(T2464 / "cells.tsv").read().splitlines()[1:]:
    fn, case, cell, args, excel = line.split("\t")
    if fn != "T.DIST.RT":
        continue
    v = args.split("|")
    try:
        t, n = float(v[0]), float(v[1]); e = float(excel)
    except ValueError:
        continue
    n = math.trunc(n)
    if 0 < t <= 1 and n >= 1 and n != 2 and n <= 1e6:
        rows.append((case, cell, t, n, e))
t = np.array([r[2] for r in rows]); n = np.array([r[3] for r in rows], dtype=float); E = np.array([r[4] for r in rows])
tt = mul(t, t); z = div(tt, add(n, tt))
a = np.full_like(z, 0.5); b = n / 2
I = [exact_cdf(zi, 0.5, bi) for zi, bi in zip(z, b)]
lnBe = [mpmath.log(mpmath.beta(M(0.5), M(bi))) for bi in b]
Sx = [i / (M(zi) ** M(0.5) / M(0.5) / mpmath.exp(l)) for i, zi, l in zip(I, z, lnBe)]
ga, gb, gab = xl("GAMMALN", a), xl("GAMMALN", b), xl("GAMMALN", add(a, b))
Lgl = add(ga, sub(gb, gab)); Lex = np.array([float(l) for l in lnBe])
alx = mul(0.5, xl("LN", z))
Sb, _ = bpser_sum(z, a, b, mode="ext", tol_kind="unchanged", nmax=20000)
def tail(H):
    return sub(0.5, mul(H, 0.5))
cands = {"inner correctly rounded": np.array([float(v) for v in I])}
for Ln, L in (("GL<=8 else exact", np.where(b <= 8, Lgl, Lex)), ("exact always", Lex), ("GL always", Lgl)):
    P = div(xl("EXP", sub(alx, L)), 0.5)
    cands[f"series bpser-ext, L {Ln}"] = mul(P, Sb)
    cands[f"P*S_exact, L {Ln}"] = np.array([float(M(p) * s) for p, s in zip(P, Sx)])
cats = {"n=1": n == 1, "3<=n<=16": (n >= 3) & (n <= 16), "n>=17": n >= 17}
lines = [f"T.DIST.RT 0<t<=1 hold-out: {len(rows)} cells"]
for nm, H in cands.items():
    T = tail(H)
    lines.append(f"  {nm:38s} " + "  ".join(f"{c}: {fmt(score(T[m], E[m]))}" for c, m in cats.items()))
txt = "\n".join(lines); print(txt); open(HERE / "tdist_holdout.out", "w").write(txt + "\n")
save_exact()
