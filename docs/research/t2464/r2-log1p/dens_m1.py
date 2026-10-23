"""Re-score w1-density's M1 (../w1-density/model_final.py) with ly = this worker's Excel log1p model instead of
the correctly rounded log1p, on the old pdf cells and probe1.  Usage: dens_m1.py"""
import sys

import r2lib as R  # noqa: F401  (paths)
import excel_log1p as XL

sys.path.insert(0, str(R.HERE.parent / "w1-density"))
import model_final as MF  # noqa: E402

orig = MF.ingredients


def patched(rows):
    ing = orig(rows)
    for r, g in zip(rows, ing):
        if 0 < r["x"] <= 0.375:
            g["l1p"] = R.mpf(XL.log1p_series(-r["x"]))
    return ing


if __name__ == "__main__":
    for lab, loader in (("old", MF.load_pdf), ("probe1", lambda: MF.load_probe("probe1"))):
        rows = loader()
        base = MF.m1(rows)
        MF.ingredients = patched
        new = MF.m1(rows)
        MF.ingredients = orig
        idx = [i for i, r in enumerate(rows) if MF.modelled(r["a"], r["b"])]
        lo = [i for i in idx if rows[i]["x"] <= 0.375]
        for nm, sel in (("modelled", idx), ("modelled, x<=0.375", lo)):
            ev = [i for i in sel if i % 2 == 0]
            od = [i for i in sel if i % 2 == 1]
            print(f"{lab:7s} {nm:20s} n={len(sel):5d}  M1(CR log1p) {MF.fmt(MF.score(base, rows, sel))}  "
                  f"M1(Excel log1p) {MF.fmt(MF.score(new, rows, sel))}  even {MF.score(new, rows, ev)[1]}/{len(ev)} "
                  f"odd {MF.score(new, rows, od)[1]}/{len(od)}")
