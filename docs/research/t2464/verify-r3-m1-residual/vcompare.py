"""Compare this verifier's lnB (vlib.lnb_m2) and densities with the worker's frozen lnb_m2 / M2 combine, per shape,
and list the rows where the two implementations disagree. Usage: vcompare.py [dataset ...]"""

import sys

import vlib as V
import vscore as S

sys.path.insert(0, str(V.T2464 / "r3-m1-residual"))
import final_score as W  # noqa: E402  (the worker's frozen lnb_m2, imported after reading it against RESULTS.md)
import m1lib as WM  # noqa: E402


def main(names):
    V.load_measured_gl("t2513-r3-m1-residual-gl")
    for n in names:
        rows = S.load(n)
        WM.ensure_gl(rows)
        mine = V.m2(rows, lambda a, b: V.lnb_m2(a, b, True, "toms"))
        shapes = {(r["a"], r["b"]) for r in rows}
        dl = {k: (V.lnb_m2(*k), float(W.lnb_m2(*k, "toms"))) for k in shapes}
        diff = {k: v for k, v in dl.items() if v[0] != v[1]}
        print(f"[{n}] shapes {len(shapes)}, lnB differs on {len(diff)}: {sorted(diff.items())[:8]}")
        nd = 0
        for r, m in zip(rows, mine):
            tx, ty = WM.pieces(r["x"], r["a"], r["b"])
            w = WM.combine(tx, ty, W.lnb_m2(r["a"], r["b"], "toms"), r["a"], r["b"], "m2")
            if w != m:
                nd += 1
                if nd <= 6:
                    print("   row", r["src"], r["x"], r["a"], r["b"], "mine", m, "worker", w, "excel", r["excel"])
        print(f"   density rows differing: {nd} of {len(rows)}")


if __name__ == "__main__":
    main(sys.argv[1:] or ["old", "probe1", "a", "hold"])
