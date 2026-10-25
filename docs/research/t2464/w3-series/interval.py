"""Interval inversion: for a prefactor model P (double), Excel E = fl(P*S) admits S in [(E-h)/P, (E+h)/P]. Express
as delta = S/S_exact - 1 (units) and find the constant (or linear-in-index) delta covering the most cells.
If P is right and S is accurate, coverage -> ~100%."""
import sys
from lib import *
M = mpmath.mpf
blk = sys.argv[1]; Ld = int(sys.argv[2]); form = sys.argv[3] if len(sys.argv) > 3 else "exp"
rows = [r for r in load(cum=True, branch="direct") if r["block"] == blk]
x = np.array([r["x"] for r in rows]); A = rows[0]["a"]; B = rows[0]["b"]
E = np.array([r["excel"] for r in rows])
mA, mB = M(A), M(B)
I = [exact_cdf(xi, A, B) for xi in x]
Bt = mpmath.beta(mA, mB)
T = [M(xi) ** mA / (mA * Bt) for xi in x]
Sx = [i / t for i, t in zip(I, T)]
lnB0 = float(mpmath.log(Bt)); L = lnB0 + Ld * math.ulp(lnB0)
lnx = xl("LN", x); alx = mul(A, lnx); z = sub(alx, L)
P = div(xl("EXP", z), A)
lo, hi = [], []
for e, p, s in zip(E, P, Sx):
    h = M(math.ulp(e)) / 2
    lo.append(float(((M(e) - h) / M(p)) / s - 1) / U)
    hi.append(float(((M(e) + h) / M(p)) / s - 1) / U)
lo, hi = np.array(lo), np.array(hi)
print(blk, "interval width mean %.2f" % (hi - lo).mean(), "centre mean %.2f sd %.2f" % (((lo+hi)/2).mean(), ((lo+hi)/2).std()))
grid = np.linspace(-20, 20, 4001)
cov = [(np.sum((lo <= c) & (c <= hi)), c) for c in grid]
cov.sort(reverse=True)
print("best constant delta: %d/%d at %.3f" % (cov[0][0], len(E), cov[0][1]))
# linear in x-index
idx = np.arange(len(E)) / len(E)
best = (0,)
for c0 in np.linspace(-12, 12, 481):
    for c1 in np.linspace(-4, 4, 81):
        d = c0 + c1 * idx
        n = np.sum((lo <= d) & (d <= hi))
        if n > best[0]: best = (n, c0, c1)
print("best linear delta: %d at c0 %.2f c1 %.2f" % best)
c = (lo + hi) / 2
w = c - c.mean(); F = np.abs(np.fft.rfft(w)) ** 2; t = np.argsort(F)[::-1][:10]
print("implied-S centre lines", list(zip(t.tolist(), (F[t] / F.mean()).round(0).astype(int).tolist())))
