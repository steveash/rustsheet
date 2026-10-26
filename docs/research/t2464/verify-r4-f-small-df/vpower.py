"""verify-r4-f-small-df: does F.DIST at a df of 2 equal the in-sheet POWER closed forms of probe -a?
d1 = 2: F = (1 + 2x/d2)^-h exactly, so the test is F == C. d2 = 2: F = x^-2 (1 + 2/(d1 x))^-h, tested as
F == fl(D/fl(x*x)) and F == fl(fl(D/x)/x). Usage: vpower.py"""

from __future__ import annotations

import json

import vscore as V

CID = "t2586-r4-f-small-df-a"


def main() -> None:
    cells = json.loads((V.CASES / f"{CID}.case.json").read_text())["sheets"][0]["cells"]
    orc = json.loads((V.CASES / f"{CID}.oracle.json").read_text())["sheets"]["Calc"]
    byrow = {r["row"]: r for r in V.rows(CID)}
    t = {"d1=2 F==C": [0, 0], "d2=2 F==D/(x*x)": [0, 0], "d2=2 F==D/x/x": [0, 0], "df2 CR==F": [0, 0]}
    for i, r in byrow.items():
        c, d = orc.get(f"C{i}"), orc.get(f"D{i}")
        if f"C{i}" not in cells or c is None or d is None:
            continue
        x, F = r["x"], r["excel"]
        if r["d1"] == 2 and r["d2"] != 2:
            t["d1=2 F==C"][0] += F == c["v"]
            t["d1=2 F==C"][1] += 1
        if r["d2"] == 2 and r["d1"] != 2:
            t["d2=2 F==D/(x*x)"][0] += F == d["v"] / (x * x)
            t["d2=2 F==D/(x*x)"][1] += 1
            t["d2=2 F==D/x/x"][0] += F == d["v"] / x / x
            t["d2=2 F==D/x/x"][1] += 1
        t["df2 CR==F"][0] += F == float(V.exact(x, r["d1"], r["d2"]))
        t["df2 CR==F"][1] += 1
    for k, (a, n) in t.items():
        print(f"{k}: {a} of {n}")


if __name__ == "__main__":
    main()
