"""Frozen-model scores (M-edge = edgemodel.value, frozen 2026-10-24 before the hold-out probe was read).
  final_score.py probe a|hold      this directory's probes by section (Excel = C/2^600 where present, else B)
  final_score.py set NAME [...]    slib sets / case:<id> by edge region (CR, M-sd, M-edge, M-edge-mulr)"""
import sys

import edgemodel as MD
import elib as E
import s4_probe as S4


def probe(name):
    rows = [o for o in S4.MK.rows(name) if S4.excel_of(o) is not None]
    bad = [o for o in rows if o["x"] is not None and o["A"] and o["A"].get("v") != o["x"]]
    print(f"# probe {name}: {len(rows)} rows, echo mismatches {len(bad)}")
    kinds = list(dict.fromkeys(o["kind"] for o in rows))
    for mn, fin in (("M-edge", "div"), ("M-edge-mulr", "mulr")):
        res = [(MD.value(S4.x_of(o), o["d1"], o["d2"], fin), S4.excel_of(o)) for o in rows]
        print(f"{mn:12s} all {E.tally(res)}")
        for k in kinds:
            sub = [p for p, o in zip(res, rows) if o["kind"] == k]
            print(f"   {k:7s} {E.tally(sub)}")
    vr = [o for o in rows if o["kind"] == "vrun"]
    if vr:
        for reg in ("d1=1", "d1=2", "d2=1", "d2=2"):
            sub = [o for o in vr if E.region(o) == reg]
            res = [(MD.value(o["x"], o["d1"], o["d2"]), S4.excel_of(o)) for o in sub]
            import mpmath
            cr = [(float(mpmath.mpf(MD.L.F.f_exact(o["x"], o["d1"], o["d2"]))), S4.excel_of(o)) for o in sub]
            print(f"   vrun {reg}: M-edge {E.tally(res)}  CR {E.tally(cr)}")


def sets(names):
    M = {"CR": lambda r: r["cr"], "M-sd": lambda r: E.SD.value(r["x"], r["d1"], r["d2"]),
         "M-edge": lambda r: MD.value(r["x"], r["d1"], r["d2"]),
         "M-edge-mulr": lambda r: MD.value(r["x"], r["d1"], r["d2"], "mulr")}
    for s in names:
        E.report(s, E.L.load(s), M)


if __name__ == "__main__":
    if sys.argv[1] == "probe":
        for n in sys.argv[2:]:
            probe(n)
    else:
        sets(sys.argv[2:])
