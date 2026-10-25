"""Hold-out on T.DIST.RT (0 < t <= 1, df != 2): 0.5 - I_z(0.5, n/2)/2 with the w3 model for the inner I
(fwd-from-1/a x87 series, lnB with accurate lnGamma below 1 and GAMMALN above, accurate for max > 8),
z53 or z64 EXP argument. Never used for tuning."""
from lib import *
from model import model
from series import bpser_sum
src = open(HERE / "tdist_holdout.py").read().split("cands = {")[0]
exec(src)
cands = {"inner correctly rounded": np.array([float(v) for v in I]),
         "previous best (bpser, GL<=8 else exact)": None}
P = div(xl("EXP", sub(alx, np.where(b <= 8, Lgl, Lex))), 0.5)
cands["previous best (bpser, GL<=8 else exact)"] = mul(P, Sb)
cands["w3 model z53 big8"] = model(z, a, b, big=8.0)
cands["w3 model z64 big8"] = model(z, a, b, big=8.0, zmode="z64")
cats = {"n=1": n == 1, "3<=n<=16": (n >= 3) & (n <= 16), "n>=17": n >= 17, "all": n > 0}
lines = [f"T.DIST.RT 0<t<=1 hold-out (never tuned on): {len(rows)} cells"]
for nm, H in cands.items():
    T = tail(H)
    lines.append(f"  {nm:40s} " + "  ".join(f"{c}: {fmt(score(T[m], E[m]))}" for c, m in cats.items()))
txt = "\n".join(lines); print(txt); open(HERE / "tdist_v2.out", "w").write(txt + "\n")
H = cands["w3 model z53 big8"]; T = tail(H)
for i in range(len(rows)):
    if not digits15_ok(T[i], E[i]):
        print("  off:", rows[i][2], rows[i][3], "z", z[i], "inner model vs CR ulps", (H[i] - float(I[i])) / math.ulp(float(I[i])), "T ulps", (T[i] - E[i]) / math.ulp(E[i]))
