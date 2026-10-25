"""T-2586 r5-f-smallr: frozen F density variants (frozen before the hold-out t2586-r5-f-smallr-hold was read).
All use M-sd's constant K = sdmodel.K_of(m, n, side) (r4-f-small-df: exact K plus the stirlerr/side offset, a
longdouble), H = -h L(r) (Excel's log1p: series to 0.375, LN(fl(1 + r)) above; extended product), the stored power
P = +fl(a LN x) (side 1) | -fl(b LN x) (side 2), and F = fl(EXP_V6(E) / x).
  msd     r4-f-small-df's M-sd (E = K + P + H in extended)
  dd      E = fl(fl(fl(K) + H) + P) + (K - fl(K))     (r4-f-power's dd with M-sd's constant)
  xs      E = fl(fl(K + H) + P)                       (K extended inside the first stored sum; no outside remainder)
  xs1     E = fl(K + H + P)                           (one stored sum)
Usage: final_model.py SET [SET ...]   (sets as in r5lib.load, e.g. own:a, pw:a, br:big, own:hold)"""
from __future__ import annotations

import collections
import importlib.util
import sys

import r5lib as R

LD, st = R.LD, R.st
_spec = importlib.util.spec_from_file_location("r4sd", R.T / "r4-f-small-df" / "sdmodel.py")
sys.path.insert(0, str(R.T / "r4-f-small-df"))
SD = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(SD)
EXP = R.F.R3.exp_v6
VARIANTS = ("CR", "msd", "dd", "xs", "xs1")


def value(x: float, m: float, n: float, var: str) -> float:
    if var == "msd":
        return SD.value(x, m, n, "div")
    side1, a, b, h, r, P = R.base(x, m, n)
    K = LD(SD.K_of(m, n, 1 if side1 else 2))
    L = LD(R.F.TD.log1p_xl(r))
    H = -LD(h) * L
    Pd = LD(float(P))
    if var == "dd":
        Kh = LD(st(K))
        E = LD(st(LD(st(Kh + H)) + Pd)) + (K - Kh)
    elif var == "xs":
        E = LD(st(LD(st(K + H)) + Pd))
    elif var == "xs1":
        E = LD(st(K + H + Pd))
    else:
        raise ValueError(var)
    return float(LD(EXP(E, True)) / LD(x))


def main():
    for src in sys.argv[1:]:
        rows = R.load(src)
        for var in VARIANTS:
            g = collections.defaultdict(list)
            for q in rows:
                v = q["cr"] if var == "CR" else value(q["x"], q["d1"], q["d2"], var)
                side1, _, _, _, r, _ = R.base(q["x"], q["d1"], q["d2"])
                g["all"].append((v, q["excel"]))
                g["r<=0.375" if r <= 0.375 else "r>0.375"].append((v, q["excel"]))
            print(f"{src:10s} {var:4s} " + " | ".join(f"{k} {R.F.fmt(R.F.tally(v))}" for k, v in sorted(g.items())))
            sys.stdout.flush()


if __name__ == "__main__":
    main()
