"""Side checks: (1) the max = 8 boundary (GL form vs TOMS algdiv) on the shapes with max == 8; (2) engine GAMMALN
against Excel's measured GAMMALN; (3) the min = 1 misses of M2 in every dataset (row, ulps, x, echo)."""

import vlib as V
import vscore as S


def main():
    for c in S.GL_CASES:
        V.load_measured_gl(c)
    data = {n: S.load(n) for n in ("old", "probe1", "a", "hold", "vhold")}
    print("(1) max == 8 shapes")
    for n, rows in data.items():
        sh = {}
        for r in rows:
            if max(r["a"], r["b"]) == 8.0:
                sh.setdefault((r["a"], r["b"]), []).append(r)
        for (a, b), rs in sorted(sh.items()):
            V.prefetch(rs)
            lo = min(a, b)
            if lo == 1.0:
                continue
            g = V.lnb_m2(a, b, True, "toms", "x")
            al = V.gl(lo) + V.algdiv(lo, 8.0, V._C[("LN", 8.0)])
            ex = [r["excel"] for r in rs]
            sc = [V.score(V.m2(rs, lambda *_, L=L: L, "x"), ex)[1] for L in (g, al)]
            print(f"   {n} ({a}, {b}) n {len(rs)}: GL form {sc[0]}, TOMS algdiv {sc[1]} (lnB equal: {g == al})")
    print("(2) engine GAMMALN vs measured")
    args = sorted(V._MEASURED_GL)
    V.xl("GAMMALN", args)
    for lab, sel in (("< 1", lambda v: v < 1), (">= 1", lambda v: v >= 1)):
        s = [v for v in args if sel(v)]
        d = [v for v in s if V._C[("GAMMALN", v)] != V._MEASURED_GL[v]]
        print(f"   args {lab}: {len(s)}, engine differs on {len(d)}")
    print("(3) min = 1 misses")
    for n, rows in data.items():
        rs = [r for r in rows if V.region(r["a"], r["b"]) == "min1"]
        mod = V.m2(rs, lambda a, b: V.lnb_m2(a, b, True, "toms", "x"), "x")
        for r, m in zip(rs, mod):
            if m != r["excel"]:
                print(f"   {r['src']} x={r['x']!r} a={r['a']} b={r['b']} model {m!r} excel {r['excel']!r} "
                      f"ulps {V.ulpdist(m, r['excel'])}")


if __name__ == "__main__":
    main()
