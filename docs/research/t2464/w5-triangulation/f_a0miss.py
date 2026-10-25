import math, sys
sys.path.insert(0, '.')
import w5lib as L
from f_argform_closed import forms
n_diff = 0; n_tot = 0
for c in L.cells({"F.DIST", "F.DIST.RT", "FDIST"}):
    a_ = c["args"]
    if c["fn"] == "F.DIST" and a_[3] is not True: continue
    X, d1, d2 = a_[0], float(math.trunc(a_[1])), float(math.trunc(a_[2]))
    if not X > 0 or (d1 != 2 and d2 != 2): continue
    a, b = d1 / 2, d2 / 2
    F = forms(X, d1, d2)
    w, v = F["1/(1+q) | 1/(1+u)"]
    w2, v2 = F["1/(1+r/x) | 1/(1+x/r)"]
    n_tot += 1; n_diff += (w, v) != (w2, v2)
    swap = w > 0.5
    a0, b0, x0 = (b, a, v) if swap else (a, b, w)
    if not (a0 == 1 and b0 != 1 and x0 >= 0.1): continue
    P = L.EXP(b0 * L.LN(1 - x0))
    tail = "lower" if c["fn"] == "F.DIST" else "upper"
    # oriented lower w0 = 1 - P; original lower = w0 (no swap) or 1 - w0
    w0a = (0.5 - P) + 0.5
    if swap:
        m = {"lower": (0.5 - w0a) + 0.5, "upper": w0a}[tail]
        alt = {"lower": P, "upper": 1 - P}[tail]
    else:
        m = {"lower": w0a, "upper": (0.5 - w0a) + 0.5}[tail]
        alt = {"lower": 1 - P, "upper": P}[tail]
    ex = c["excel"]
    print(f"{'MISS' if m != ex else 'ok  '} {tail} swap={swap} x0={x0:.5f} b0={b0} m={(m-ex)/math.ulp(ex):+.0f} P-direct={(alt-ex)/math.ulp(ex):+.0f} exact={(float(L.ibeta(w,a,b) if tail=='lower' else 1-L.ibeta(w,a,b))-ex)/math.ulp(ex):+.1f}")
print("forms differ on", n_diff, "of", n_tot)
