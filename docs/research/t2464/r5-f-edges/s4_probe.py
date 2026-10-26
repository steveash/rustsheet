"""Score models on this directory's probe rows by section (kind). Excel's value is C/2^600 where the scaled column
exists and is a number (a kept subnormal shows there), else B. Usage: s4_probe.py NAME [--miss KIND]
(NAME 'a' in sample; 'hold' only from final_score.py)."""
import sys

import elib as E
import edgeprobe as MK
import edgemodel as MD


def excel_of(o):
    c = o["C"] if o["scale"] and o["C"] is not None else o["B"]
    if c is None:
        return None
    if c.get("t") == "n":
        return c["v"] / 2**600 if c is o["C"] else c["v"]
    return c["v"]


def x_of(o):
    """The x Excel evaluated: the echo for formula x; a kept subnormal for the EXP(-k) rows."""
    if o["x"] is not None:
        return o["x"]
    f = o["xf"]
    if f.startswith("EXP("):
        return float(E.L.EXP(E.L.LDB(float(f[4:-1]))))
    if f.startswith("-"):
        return -1.0
    raise ValueError(f)


MODELS = {"M-edge": lambda o: MD.value(x_of(o), o["d1"], o["d2"]),
          "M-edge-mulr": lambda o: MD.value(x_of(o), o["d1"], o["d2"], "mulr"),
          "CR+rules": None}


def main():
    name = sys.argv[1]
    rows = [o for o in MK.rows(name) if excel_of(o) is not None]
    bad = [o for o in rows if o["x"] is not None and o["A"] and o["A"].get("v") != o["x"]]
    print(f"# {name}: {len(rows)} rows, echo mismatches {len(bad)}")
    kinds = sorted({o["kind"] for o in rows}, key=lambda k: [o["kind"] for o in rows].index(k))
    for mn in ("M-edge", "M-edge-mulr"):
        fn = MODELS[mn]
        res = [(fn(o), excel_of(o)) for o in rows]
        print(f"{mn:12s} all {E.tally(res)} | " + " | ".join(
            f"{k} {E.tally([p for p, o in zip(res, rows) if o['kind'] == k])}" for k in kinds))
    if "--miss" in sys.argv:
        k = sys.argv[sys.argv.index("--miss") + 1]
        for o in rows:
            if o["kind"] != k:
                continue
            m, e = MODELS["M-edge"](o), excel_of(o)
            if not E.grade(m, e)[2]:
                print(f"  row {o['row']} x={o['x']!r} {o['d1s']},{o['d2s']} model={m!r} excel={e!r}")


if __name__ == "__main__":
    main()
