"""Verifier (T-2586, verify-r5-f-final): score the frozen M-final with own parser/grader.
  vscore.py hold     integrator hold-out t2586-r5-f-final-hold (value cells, literal edge cells)
  vscore.py sets     every earlier F.DIST(.,FALSE) case in probes-numeric, per set and in total
  vscore.py vhold    this verifier's hold-out t2586-verify-r5-f-final-hold (vs the predictions file)
  vscore.py off      visibly-off cells (not within 15 digits) of M-final across all sets incl both hold-outs"""

from __future__ import annotations

import json
import sys as _s
from pathlib import Path as _P
_s.path.insert(0, str(_P(__file__).resolve().parent))
import sys

import vflib as L
import vfparse as V

M = L.load_model()
MODELS = {
    "M-final": lambda r: M.value(r["x"], r["d1"], r["d2"]),
    "M-final-mulr": lambda r: M.value(r["x"], r["d1"], r["d2"], "mulr"),
    "M-sd+rules": lambda r: M.EM.value(r["x"], r["d1"], r["d2"]),
    "CR+rules": lambda r: L.cr_value(r["x"], r["d1"], r["d2"]),
}
FINAL = "t2586-r5-f-final-hold"
VHOLD = "t2586-verify-r5-f-final-hold"


def usable(rows):
    return [r for r in rows if r["excel"] is not None and not r["xapprox"] or isinstance(r["excel"], str)]


def report(name, rows, keys=L.KEYS, models=MODELS):
    print(f"== {name} ({len(rows)} cells)")
    res = {}
    for nm, fn in models.items():
        t = L.tally(rows, [fn(r) for r in rows])
        res[nm] = t
        print(f"  {nm:13s} {L.fmt(t, keys)}")
    return res


def earlier_sets():
    out = {}
    for c in V.all_cases():
        if c in (FINAL, VHOLD):
            continue
        rows = [r for r in V.rows(c) if r["excel"] is not None]
        if rows:
            out[c] = rows
    return out


def main(a):
    if a == "hold":
        rows = V.rows(FINAL)
        val = [r for r in rows if not r["literal"]]
        lit = [r for r in rows if r["literal"]]
        report("final-hold value cells", val)
        report("final-hold literal edge cells", lit)
        for r in lit:
            p = M.value(r["x"], r["d1"], r["d2"])
            g = L.grade(p, r["excel"])
            print(f"   {r['cell']:4s} x={r['x']!r} d=({r['d1']!r},{r['d2']!r}) excel={r['excel']!r} model={p!r} {g}")
    elif a == "sets":
        allr, tds = [], []
        for nm, rows in earlier_sets().items():
            report(nm, rows, ["all", "value", "rule"])
            allr += rows
        print()
        integ = [r for r in allr if not r["case"].startswith("t2513-r3-tdensity")]
        report("ALL earlier (excl tdensity products)", integ)
        report("ALL earlier incl tdensity", allr)
        hold = V.rows(FINAL)
        report("ALL earlier (excl tdensity) + final-hold", integ + hold)
        seen, ded = set(), []
        for r in allr + hold:
            k = (r["x"], r["d1"], r["d2"])
            if k not in seen:
                seen.add(k)
                ded.append(r)
        report("ALL distinct (x,d1,d2) incl tdensity + final-hold", ded)
        L.save_cache()
    elif a == "vhold":
        rows = V.rows(VHOLD)
        pred = json.loads((L.HERE / "vhold_pred.json").read_text())
        bad = 0
        for r in rows:
            p = pred[r["cell"]]
            now = M.value(r["x"], r["d1"], r["d2"])
            if not (p == now or (isinstance(p, float) and isinstance(now, float) and p == now)):
                bad += 1
        print(f"# prediction file vs live frozen model: {bad} differences")
        pm = {"M-final(pred file)": lambda r: pred[r["cell"]], **MODELS}
        val = [r for r in rows if not r["literal"]]
        lit = [r for r in rows if r["literal"]]
        report("verify hold value cells", val, models=pm)
        report("verify hold literal cells", lit, models=pm)
        for r in lit:
            p = pred[r["cell"]]
            g = L.grade(p, r["excel"])
            print(f"   {r['cell']:4s} x={r['x']!r} d=({r['d1']!r},{r['d2']!r}) excel={r['excel']!r} model={p!r} {g}")
        L.save_cache()
    elif a == "off":
        sets = earlier_sets()
        sets[FINAL] = V.rows(FINAL)
        if (V.CASES / f"{VHOLD}.oracle.json").exists():
            sets[VHOLD] = V.rows(VHOLD)
        cnt: dict = {}
        tot = 0
        for nm, rows in sets.items():
            for r in rows:
                if not L.grade(M.value(r["x"], r["d1"], r["d2"]), r["excel"])[2]:
                    tot += 1
                    for k in L.region(r)[1:]:
                        cnt[k] = cnt.get(k, 0) + 1
        print(f"visibly off (not within 15 digits): {tot}")
        for k in sorted(cnt):
            print(f"  {k:24s} {cnt[k]}")


if __name__ == "__main__":
    main(sys.argv[1])
