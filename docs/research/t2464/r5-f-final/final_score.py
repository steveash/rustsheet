"""T-2586 r5-f-final: one-shot scores of the frozen M-final (fdensity_excel.py) and baselines, per region.
  final_score.py hold        the integrator's hold-out t2586-r5-f-final-hold (value rows + literal edge cells)
  final_score.py sets        cells.tsv's F pdf rows and every earlier F.DIST(.,FALSE) probe case
  final_score.py off         M-final's visibly-off cells (not within 15 digits) on all sets, by region
Scores are bit-exact / within 4 ulp / within 15 digits of n (r5-f-edges `elib.grade`: errors and zeros must match
exactly). Case cells are parsed by verify-r3-fdensity `vlib` (15-digit literal truncation, echo checks)."""

from __future__ import annotations

import math
import pickle
import re
import sys
from pathlib import Path

import fdensity_excel as M
import mkhold as MH

HERE = Path(__file__).resolve().parent
EL = M.EM.E  # r5-f-edges elib: grade
SL, SD, BC, EM = M.SL, M.BC.SD, M.BC, M.EM
F = SL.F
V = SL.B._load("vlib_r5final", M.T / "verify-r3-fdensity" / "vlib.py")
_PC = HERE / "exact_cache.pkl"
_EX: dict = pickle.loads(_PC.read_bytes()) if _PC.exists() else {}


def exact(x: float, m: float, n: float):
    k = (x, m, n)
    if k not in _EX:
        _EX[k] = F.f_exact(x, m, n)
    return _EX[k]


def _with_rules(core):
    def fn(r):
        m, n = float(math.trunc(r["d1"])), float(math.trunc(r["d2"]))
        v = M.rule(r["x"], m, n)
        return v if v is not None else core(r["x"], m, n)

    return fn


def _cr(x, m, n):
    v = float(exact(x, m, n))
    return 0.0 if abs(v) < M.TINY else v


MODELS = {
    "M-final": lambda r: M.value(r["x"], r["d1"], r["d2"]),
    "M-final-mulr": lambda r: M.value(r["x"], r["d1"], r["d2"], "mulr"),
    "M-sd+rules": lambda r: EM.value(r["x"], r["d1"], r["d2"]),
    "M-sdm+rules": lambda r: EM.value(r["x"], r["d1"], r["d2"], "mulr"),
    "CR+rules": _with_rules(_cr),
}


def region(r) -> list[str]:
    """Region keys of a value row: side, r class, df class, edge class."""
    m, n, x = math.trunc(r["d1"]), math.trunc(r["d2"]), r["x"]
    if not (x > 0 and m >= 1 and n >= 1) or isinstance(r["excel"], str) or r["excel"] == 0:
        return ["rule"]
    side, rr = SL.side_r(x, float(m), float(n))
    lo = "r<=.375" if rr <= 0.375 else "r>.375"
    dfc = "small" if min(m, n) < 5 else ("mid" if min(m, n) < 16 else "large")
    keys = [f"side{side}", lo, f"side{side}/{lo}", dfc, f"{dfc}/{lo}"]
    if x < 1e-6 or x > 1e6 or abs(r["excel"]) < 1e-280:
        keys.append("edge-x")
    return keys


def tally_by(rows, fn):
    out: dict = {}
    for r in rows:
        p = (fn(r), r["excel"])
        for k in ["all"] + region(r):
            out.setdefault(k, []).append(p)
    return {k: EL.tally(v) for k, v in out.items()}


KEYS = [
    "all",
    "side1",
    "side2",
    "r<=.375",
    "r>.375",
    "side1/r<=.375",
    "side1/r>.375",
    "side2/r<=.375",
    "side2/r>.375",
    "small",
    "mid",
    "large",
    "small/r<=.375",
    "mid/r<=.375",
    "large/r<=.375",
    "small/r>.375",
    "mid/r>.375",
    "large/r>.375",
    "edge-x",
    "rule",
]


def report(name, rows):
    print(f"== {name} ({len(rows)} cells)")
    for nm, fn in MODELS.items():
        t = tally_by(rows, fn)
        print(f"  {nm:13s} " + " | ".join(f"{k} {t[k].replace(' of ', '/')}" for k in KEYS if k in t))


# ------------------------------------------------------------------ data
def _xl(v: dict):
    if v is None:
        return None
    return float(v["v"]) if v.get("t") == "n" else (v["v"] if v.get("t") == "e" else None)


def _edge_args(args: str):
    """(x, d1, d2) of a literal edge cell as Excel reads them (TRUE -> 1, text "2" -> 2, EXP(k) -> Excel's EXP)."""
    out = []
    for tok in args.split(","):
        tok = tok.strip()
        if tok == "TRUE":
            out.append(1.0)
        elif tok.startswith('"'):
            out.append(V.lit15(tok.strip('"')))
        elif mm := re.fullmatch(r"EXP\((-?\d+)\)", tok):
            out.append(SL.EXP(SL.LDB(int(mm.group(1)))))
        else:
            out.append(V.lit15(tok))
    return out


def hold_rows():
    sh = F.sidecar(MH.CASE)
    rows, bad = [], 0
    for i, (blk, d1, d2, x) in enumerate(MH.plan(), 1):
        echo, ex = F.val(sh, f"A{i}"), _xl(sh.get(f"B{i}"))
        if echo != x:
            bad += 1
            continue
        rows.append(dict(src="hold", blk=blk, x=x, d1=d1, d2=d2, excel=ex))
    edges = []
    for i, args in enumerate(MH.EDGES, 1):
        x, d1, d2 = _edge_args(args)
        edges.append(dict(src="edge", blk=args, x=x, d1=d1, d2=d2, excel=_xl(sh.get(f"D{i}"))))
    print(f"# hold-out: {len(rows)} value rows ({bad} echo mismatches), {len(edges)} edge cells")
    return rows, edges


def case_ids():
    pat = re.compile(r"^(t2220-f-dist-sweep-|t2513-r3-fdensity-|t2513-verify-r3-fdensity|t2586-)")
    return sorted(
        p.name[: -len(".case.json")]
        for p in V.CASES.glob("*.case.json")
        if pat.match(p.name)
        and p.name[: -len(".case.json")] != MH.CASE
        and (V.CASES / p.name.replace(".case.json", ".oracle.json")).exists()
    )


def set_rows():
    """{set name: rows}: 'cells' (cells.tsv) and each case."""
    out = {"cells": [r for r in F.cells_rows() if r["excel"] > 0]}
    for cid in case_ids():
        if "r5-f-edges" in cid:  # their own layout: read through r5-f-edges' probe loader (exact x, scaled C column)
            continue
        out[cid] = V.case_rows(cid)
    S4 = SL.B._load("s4_probe", M.T / "r5-f-edges" / "s4_probe.py")
    for nm in ("a", "hold"):
        rows = [o for o in S4.MK.rows(nm) if S4.excel_of(o) is not None and S4.x_of(o) is not None]
        out[f"t2586-r5-f-edges-{nm}"] = [dict(src=nm, blk=o["kind"], x=S4.x_of(o), d1=o["d1"], d2=o["d2"],
                                              excel=S4.excel_of(o)) for o in rows]
    return out


def main(argv):
    if argv[0] == "hold":
        rows, edges = hold_rows()
        report("final-hold values", rows)
        report("final-hold edges", edges)
        for e in edges:
            got = M.value(e["x"], e["d1"], e["d2"])
            g = EL.grade(got, e["excel"])
            print(f"   edge F.DIST({e['blk']}): excel {e['excel']!r} model {got!r} {'OK' if g[2] else 'OFF'}")
    elif argv[0] == "sets":
        allr = []
        for nm, rows in set_rows().items():
            report(nm, rows)
            allr += rows
        report("ALL earlier sets", allr)
        report("ALL earlier sets + final-hold", allr + hold_rows()[0])
    elif argv[0] == "off":
        sets = set_rows()
        sets["final-hold"] = hold_rows()[0]
        cnt: dict = {}
        for nm, rows in sets.items():
            for r in rows:
                if not EL.grade(M.value(r["x"], r["d1"], r["d2"]), r["excel"])[2]:
                    k = "/".join(region(r)[2:4]) if len(region(r)) > 1 else "rule"
                    cnt.setdefault(k, [0, set()])
                    cnt[k][0] += 1
                    cnt[k][1].add((math.trunc(r["d1"]), math.trunc(r["d2"])))
        tot = sum(v[0] for v in cnt.values())
        print(f"# visibly-off M-final cells: {tot}")
        for k, (c, shp) in sorted(cnt.items(), key=lambda kv: -kv[1][0]):
            print(f"  {k:24s} {c:6d} cells, {len(shp)} shapes")
    _PC.write_bytes(pickle.dumps(_EX))


if __name__ == "__main__":
    main(sys.argv[1:])
