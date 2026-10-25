"""Equality of Excel's F.DIST with the in-sheet POWER closed forms at df 2 (probe mya columns C, D):
C = POWER(1+d1*x/d2, -h), D = POWER(1+d2/(d1*x), -h). At d1 = 2, F = C exactly in math; at d2 = 2, F = D/x^2... .
Reports per (shape, side): F == C, F == D, F == fl(D/fl(x*x)), F == fl(fl(D/x)/x) counts."""
import slib as L

rows = [r for r in L.load("mya") if r["pc"] is not None]
g: dict = {}
for r in rows:
    s, _ = L.side_r(r["x"], r["d1"], r["d2"])
    g.setdefault((r["d1"], r["d2"], s), []).append(r)
for (m, n, s), rs in sorted(g.items()):
    eqC = sum(r["excel"] == r["pc"] for r in rs)
    eqD = sum(r["excel"] == r["pd"] for r in rs)
    eqDx2 = sum(r["excel"] == L.st(L.LDB(r["pd"]) / L.LDB(L.st(L.LDB(r["x"]) * L.LDB(r["x"])))) for r in rs)
    eqDxx = sum(r["excel"] == L.st(L.LDB(L.st(L.LDB(r["pd"]) / L.LDB(r["x"]))) / L.LDB(r["x"])) for r in rs)
    cC = sum(r["cr"] == r["pc"] for r in rs)
    print(f"({m:g},{n:g}) s{s} n {len(rs)}: F==C {eqC:2d}  F==D {eqD:2d}  F==D/x^2 {eqDx2:2d}  F==D/x/x {eqDxx:2d}"
          f" | CR==C {cC:2d}  CR==F {sum(r['cr'] == r['excel'] for r in rs):2d}")
