"""Hybrid direct-cdf candidate, no per-group fitting: min(a,b) < 1 -> EXP(fl(fl(a*LN x) - L))/a * S,
L = GLa+(GLb-GLab) if max(a,b) <= 8 else exact lnB; S = bpser summed to convergence ('unchanged', ext) or exact;
a, b > 1 -> correctly rounded (series not competitive there). Scored per region on all direct cdf cells
(closed forms a=1/b=1 excluded) and on the beta_exact cells only, with tune/hold halves."""
from lib import *
from series import bpser_sum
M = mpmath.mpf
rows = [r for r in load(cum=True, branch="direct") if r["a"] != 1.0 and r["b"] != 1.0]
x = np.array([r["x"] for r in rows]); a = np.array([r["a"] for r in rows]); b = np.array([r["b"] for r in rows])
E = np.array([r["excel"] for r in rows])
I = [exact_cdf(xi, ai, bi) for xi, ai, bi in zip(x, a, b)]
cr = np.array([float(v) for v in I])
lnBe = [mpmath.log(mpmath.beta(M(ai), M(bi))) for ai, bi in zip(a, b)]
Sx = [i / (M(xi) ** M(ai) / M(ai) / mpmath.exp(l)) for i, xi, ai, l in zip(I, x, a, lnBe)]
ga, gb, gab = xl("GAMMALN", a), xl("GAMMALN", b), xl("GAMMALN", add(a, b))
Lgl = add(ga, sub(gb, gab)); Lex = np.array([float(l) for l in lnBe])
small = np.minimum(a, b) < 1
L = np.where(small & (np.maximum(a, b) <= 8), Lgl, Lex)
P = div(xl("EXP", sub(mul(a, xl("LN", x)), L)), a)
Sb, _ = bpser_sum(x, a, b, mode="ext", tol_kind="unchanged", nmax=20000)
mS = np.array([float(M(p) * s) for p, s in zip(P, Sx)])
mB = mul(P, Sb)
models = {"hybrid S exact": np.where(small, mS, cr), "hybrid bpser ext": np.where(small, mB, cr),
          "hybrid, L exact always": np.where(small, np.array([float(M(p) * s) for p, s in zip(div(xl("EXP", sub(mul(a, xl("LN", x)), Lex)), a), Sx)]), cr),
          "correctly rounded": cr}
tune, hold = split_half(rows); tid = {id(r) for r in tune}
def reg(r):
    if min(r["a"], r["b"]) < 1:
        return ("a<1" if r["a"] < 1 else "b<1") + (" max<=8" if max(r["a"], r["b"]) <= 8 else " max>8")
    return "a,b>1"
regs = sorted({reg(r) for r in rows})
lines = []
for src in ("all", "be"):
    lines.append(f"-- source {src}")
    for nm, m in models.items():
        cells = []
        for g in regs:
            idx = [i for i, r in enumerate(rows) if reg(r) == g and (src == "all" or r["src"] == "be")]
            sc = score(m[idx], E[idx]); cells.append(f"{g}: {sc[0]}/{sc[1]}/{sc[2]} of {sc[3]}")
        ih = [i for i, r in enumerate(rows) if id(r) not in tid and (src == "all" or r["src"] == "be") and min(r["a"], r["b"]) < 1]
        sh = score(m[ih], E[ih])
        lines.append(f"{nm:24s} | " + " | ".join(cells) + f" | hold-out min<1: {sh[0]}/{sh[1]}/{sh[2]} of {sh[3]}")
txt = "\n".join(lines); print(txt); open(HERE / "hybrid.out", "w").write(txt + "\n")
save_exact()
