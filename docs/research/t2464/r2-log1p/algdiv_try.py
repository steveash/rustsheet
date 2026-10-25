"""Does Excel's alnrel (this worker's series, applied to a positive argument a/b) inside TOMS algdiv close
w1-density's a<1, b>8 density cells? Patches ../w1-density/toms_betaln.alnrel_double and rescores M1 on that
region (old cells and probe1 sheets (0.5,20), (1,20)).  Usage: algdiv_try.py"""
import sys

import r2lib as R
import excel_log1p as XL

sys.path.insert(0, str(R.HERE.parent / "w1-density"))
import model_final as MF  # noqa: E402
import toms_betaln as TB  # noqa: E402
import dens_m1  # noqa: E402

orig = TB.alnrel_double


def run(label):
    for lab, loader in (("old", MF.load_pdf), ("probe1", lambda: MF.load_probe("probe1"))):
        rows = loader()
        MF.ingredients = dens_m1.patched
        m = MF.m1(rows)
        MF.ingredients = dens_m1.orig
        sel = [i for i, r in enumerate(rows) if max(r["a"], r["b"]) > 8 and min(r["a"], r["b"]) <= 1]
        print(f"{label:28s} {lab:7s} max>8, min<=1: n={len(sel)} {MF.fmt(MF.score(m, rows, sel))}")


run("alnrel = TOMS double")
TB.alnrel_double = XL.log1p_series
run("alnrel = Excel series (x87)")
TB.alnrel_double = orig
