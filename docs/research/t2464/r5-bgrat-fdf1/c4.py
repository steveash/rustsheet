"""T-3567 r5-fdf1 c4: F's (x, y) through r, the density's side ratio, on every F cdf/tail cell at d1 = 1, full kernel."""
import io, math, sys, contextlib
from collections import Counter
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r2-tdist"))
import xlk  # noqa
with contextlib.redirect_stdout(io.StringIO()):
    import c2  # noqa
rows, ev = c2.rows, c2.ev
def xy_s(x, n, m=1):
    s = n + m * x
    return m * x / s, n / s
def xy_r2(x, n, m=1):          # always side 2: r = fl(n/fl(m x))
    r = n / (m * x)
    return 1 / (1 + r), r / (1 + r)
def xy_r12(x, n, m=1):         # density's sides: side 1 (mx <= n) r = fl(mx/n), side 2 r = fl(n/mx)
    mx = m * x
    if mx <= n:
        r = mx / n
        return r / (1 + r), 1 / (1 + r)
    r = n / mx
    return 1 / (1 + r), r / (1 + r)
def xy_r1(x, n, m=1):          # always side 1
    r = m * x / n
    return r / (1 + r), 1 / (1 + r)
def score(f, label):
    pts = [(0.5, n / 2, *f(x, n)) for fn, x, n, up, e in rows]
    out = xlk.run(pts, c2.CFG)
    tab = {}
    for r, (w, w1, b), g in zip(rows, out, ev):
        v = w1 if r[3] else w; e = r[4]
        t = tab.setdefault(b, [0, 0, 0, 0, 0])
        t[0] += 1; t[1] += v == e; t[2] += abs(v - e) <= 4 * math.ulp(e); t[3] += g == e; t[4] += abs(g - e) <= 4 * math.ulp(e)
    T = [sum(t[i] for t in tab.values()) for i in range(5)]
    print(f"{label:10s} all {T[1]}/{T[2]}/{T[0]} (engine {T[3]}/{T[4]})  " + ", ".join(f"{b} {t[1]}/{t[2]}/{t[0]} (eng {t[3]}/{t[4]})" for b, t in sorted(tab.items())))
    return out
for f, l in ((xy_s, "s"), (xy_r2, "r side2"), (xy_r1, "r side1"), (xy_r12, "r sides")):
    score(f, l)
def xy_rbig(x, n, m=1):        # r >= 1 always: the opposite of the density's sides
    mx = m * x
    if mx <= n:
        return xy_r2(x, n, m)
    return xy_r1(x, n, m)
def xy_rbig_lt(x, n, m=1):     # the same with the tie mx = n on side 1
    return xy_r2(x, n, m) if m * x < n else xy_r1(x, n, m)
print("---")
OUT = score(xy_rbig, "r>=1")
score(xy_rbig_lt, "r>=1 (<)")
if "-v" in sys.argv:
    for r, (w, w1, b), g in zip(rows, OUT, ev):
        v = w1 if r[3] else w
        if v != r[4]:
            print(b, r[:4], r[4], f"model {(v - r[4]) / math.ulp(r[4]):+.0f} engine {(g - r[4]) / math.ulp(r[4]):+.0f}")
