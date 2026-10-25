"""Frozen-model scores (models frozen 2026-10-24 before the hold-out `chold` was read).
Models (all share P = sg*fl(c LN x), H = -h L(r) extended, L = Excel's log1p; K = M-sd's constant K_sd, mpf):
  CR      correctly rounded exact density
  M-sd    r4-f-small-df's sdmodel.value (E = K + P + H extended, F = fl(EXP(E)/x))
  M-sdm   the same with fl(EXP(E)*fl(1/x))
  hkp-sd  the requested combination: S2 = fl(fl(K_hi + H) + P), K_hi = fl(K_sd), E = S2 + (K_sd - K_hi)
  hk-sd   S1 = fl(K_hi + H) stored, P added in extended: E = S1 + P + (K_sd - K_hi)   (s5's best free-fit order)
  dd      r4-f-power's fmodel 'dd' (exact K', HKP11 with K_hi = fl(K'))
  hyb     hkp-sd when max(d1, d2) > 60, else M-sdm (chosen on the in-sample size classes, final_insample.out)
Usage: final_score.py SET... (sets: ca, chold, mya, myhold, big, hold4, a, hold, sw, holdsw, cells,
case:<id>); prints all / r > 0.375 / r <= 0.375 / max(d) class tallies per model."""

import importlib.util
import sys

import clib as C

LDB, st = C.LDB, C.st
MODELS = ("CR", "M-sd", "M-sdm", "hkp-sd", "hk-sd", "dd", "hyb")
_FM = None


def _fmodel():
    global _FM
    if _FM is None:
        p = C.T / "r4-f-power"
        sys.path.insert(0, str(p))
        spec = importlib.util.spec_from_file_location("r4p_fmodel", p / "fmodel.py")
        _FM = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(_FM)
    return _FM


def value(r, model: str) -> float:
    x, m, n = r["x"], r["d1"], r["d2"]
    if model == "CR":
        return r["cr"]
    if model == "M-sd":
        return C.SD.value(x, m, n)
    if model == "M-sdm":
        return C.SD.value(x, m, n, "mulr")
    if model == "hyb":
        return value(r, "hkp-sd" if max(m, n) > 60 else "M-sdm")
    if model == "dd":
        return _fmodel().value(x, m, n, "dd")
    pc = C.pieces(x, m, n)
    K = C.K_sd(m, n, pc["side"])
    Khi = float(K)
    S1 = LDB(st(LDB(Khi) + pc["H"]))
    if model == "hkp-sd":
        S = C.ex(st(S1 + LDB(pc["P"])))
    elif model == "hk-sd":
        S = C.ex(S1) + C.ex(pc["P"])
    else:
        raise ValueError(model)
    return C.final(C.SL.ld(S + (K - C.ex(Khi))), x)


def main():
    for name in sys.argv[1:]:
        rows = C.load(name)
        for mdl in MODELS:
            g: dict = {}
            for r in rows:
                v = float(value(r, mdl))
                rr = C.SL.side_r(r["x"], r["d1"], r["d2"])[1]
                md = max(r["d1"], r["d2"])
                sz = "d<=12" if md <= 12 else ("d<=60" if md <= 60 else "d>60")
                for k in ("all", "r>0.375" if rr > 0.375 else "r<=0.375", sz):
                    g.setdefault(k, []).append((v, r["excel"]))
            print(
                f"{name:10s} {mdl:7s} "
                + " | ".join(f"{k} {C.F.fmt(C.F.tally(p))}" for k, p in sorted(g.items()))
            )
            sys.stdout.flush()


if __name__ == "__main__":
    main()
