"""T-3567 r5-fdf1 c2: the engine today vs FINAL's kernel at x = fl(x/fl(n+x)), y = fl(n/fl(n+x)), per branch."""
import io, math, sys, contextlib
from collections import Counter
from pathlib import Path
import rustsheet as ic
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r2-tdist"))
import xlk  # noqa
with contextlib.redirect_stdout(io.StringIO()):
    import c0  # noqa
rows = c0.rows
def engine(rows):
    m = ic.create("w", "en", "UTC")
    for i, (fn, x, n, up, e) in enumerate(rows):
        m.update_cell_with_number(0, i + 1, 1, x)
        f = f"=F.DIST(A{i+1},1,{n},TRUE)" if not up else f"=F.DIST.RT(A{i+1},1,{n})"
        m.update_cell_with_formula(0, i + 1, 2, f)
    m.evaluate()
    return [float(m.get_formatted_cell_value(0, i + 1, 2)) if False else xlk.englib._num(m.get_cell_value(0, i + 1, 2)) for i in range(len(rows))]
def model(rows, cfg):
    pts = [(0.5, n / 2, x / (n + x), n / (n + x)) for fn, x, n, up, e in rows]
    return [(w1 if r[3] else w, b) for r, (w, w1, b) in zip(rows, xlk.run(pts, cfg))]
CFG = xlk.XCfg(alnrel="xl", erfc="xl", bg_lnx="cut0.01", bg="ld")
mv = model(rows, CFG)
ev = engine(rows)
tab = {}
for r, (v, b), g in zip(rows, mv, ev):
    e = r[4]
    t = tab.setdefault(b, [0, 0, 0, 0, 0])
    t[0] += 1; t[1] += v == e; t[2] += abs(v - e) <= 4 * math.ulp(e); t[3] += g == e; t[4] += abs(g - e) <= 4 * math.ulp(e)
print(f"{'branch':12s} {'n':>4s} {'model ex/w4':>12s} {'engine ex/w4':>13s}")
T = [0] * 5
for b, t in sorted(tab.items()):
    print(f"{b:12s} {t[0]:4d} {t[1]:5d}/{t[2]:<5d} {t[3]:6d}/{t[4]:<5d}")
    T = [p + q for p, q in zip(T, t)]
print(f"{'all':12s} {T[0]:4d} {T[1]:5d}/{T[2]:<5d} {T[3]:6d}/{T[4]:<5d}")
if "-v" in sys.argv:
    for r, (v, b), g in zip(rows, mv, ev):
        if v != r[4]:
            print(b, r[:4], r[4], f"model {(v-r[4])/math.ulp(r[4]):+.0f} engine {(g-r[4])/math.ulp(r[4]):+.0f}")
