# T-1941 W6: apply a top-stage scan result (scan6.py) to a chain. dS_J(x) = (a + m + b dx + c dx^2 + d dx^3 ...) u,
# dx = x - X0, u = ulp(S_J) at X0, is a polynomial in x; its x^k coefficient is the change of c_{J+k}, so the scan's
# coefficients map to dc_J .. dc_top (raw-x Horner: S_J = c_J + c_{J+1} x + ... + c_top x^(top-J)). The integer m (a
# whole-u shift of S_J leaves its rounding pattern unchanged) is fitted to the data residual (env RFILE); without
# RFILE it is 0. Usage: apply6.py BASE OUT CH J a b [c [d]]   env X0 (1.75), U (override u)
import sys, os, numpy as np
from numpy.polynomial import polynomial as Pn

sys.path.insert(0, sys.path[0])
from synth import read_chain

base, out, CH, J = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4])
co = [float(v) for v in sys.argv[5:]]
X0 = float(os.environ.get("X0", 1.75))
N, D = read_chain(base)
N = list(np.array(N) / D[0])
D = list(np.array(D) / D[0])
C = N if CH == "N" else D
top = len(C) - 1
if len(co) - 1 > top - J:
    raise SystemExit("polynomial degree %d exceeds top - J = %d" % (len(co) - 1, top - J))
s = C[-1]
for j in range(top - 1, J - 1, -1):
    s = s * X0 + C[j]
u = float(os.environ.get("U", np.spacing(abs(s))))
# sum_i co_i (x - X0)^i  ->  coefficients in x
px = np.zeros(1)
for i, v in enumerate(co):
    px = Pn.polyadd(px, v * Pn.polypow([-X0, 1.0], i))
px = np.concatenate([px, np.zeros(top - J + 1 - len(px))])
dc = px * u
m = 0
if os.environ.get("RFILE"):
    # m from the data: a whole-u shift of S_J moves R by m u x^J / N (N) or -R m u x^J / D (D); regress the residual of the
    # chain with m = 0 on that pattern over a sample of RFILE (the function fixes the absolute S_J; the scan sees a mod 1)
    from lib6 import st, L, horner
    rows = [l.split() for l in open(os.environ["RFILE"])]
    idx = np.random.default_rng(0).choice(len(rows), min(40000, len(rows)), replace=False)
    x = np.array([float.fromhex(rows[i][0]) for i in idx])
    Rd = (np.array([float.fromhex(rows[i][2]) for i in idx]) + np.array([float.fromhex(rows[i][3]) for i in idx])) / 2
    C2 = list(C)
    for k in range(top - J + 1):
        uk = np.spacing(abs(C2[J + k]))
        C2[J + k] = C2[J + k] + round(dc[k] / uk) * uk
    N2, D2 = (C2, D) if CH == "N" else (N, C2)
    R = st(L(horner(N2[::-1], x)) / L(horner(D2[::-1], x)))
    r = (Rd - R) / R
    Nv, Dv = np.polyval(N2[::-1], x), np.polyval(D2[::-1], x)
    w = u * x**J / Nv if CH == "N" else -u * x**J / Dv
    m = round(float(np.sum(r * w) / np.sum(w * w)))
dc[0] += m * u
for k in range(top - J + 1):
    uk = np.spacing(abs(C[J + k]))
    C[J + k] = C[J + k] + round(dc[k] / uk) * uk
    print("  %s%d %+d ulps" % (CH, J + k, round(dc[k] / uk)))
print("whole-u shift m = %+d (a %.3f -> %.3f)" % (m, co[0], co[0] + m))
open(out, "w").write(
    "N " + " ".join(float(v).hex() for v in N) + "\nD " + " ".join(float(v).hex() for v in D) + "\n"
)
