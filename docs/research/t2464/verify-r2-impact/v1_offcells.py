"""Verifier step 1: recount visibly-off incomplete-beta-family cells straight from report.json, with my own bucket
rule (closeness.rs bucket(), re-read from source) and two attribution rules (first family call / any family call)."""
import json, math, re, sys
from collections import Counter

REPORT = "/tmp/claude-0/-home-user-rustsheet/77027746-e589-5f95-baa2-1e5785f680d3/scratchpad/evalrun/report.json"
FAM = set("""BETA.DIST BETADIST F.DIST F.DIST.RT FDIST F.TEST FTEST T.DIST T.DIST.RT T.DIST.2T TDIST T.TEST TTEST
BINOM.DIST BINOMDIST BINOM.DIST.RANGE NEGBINOM.DIST NEGBINOMDIST BETA.INV BETAINV F.INV F.INV.RT FINV T.INV T.INV.2T
TINV BINOM.INV CRITBINOM CONFIDENCE.T""".split())
CALL = re.compile(r"([A-Za-z_][A-Za-z0-9_.]*)\s*\(")


def w15(e, a):
    if not (math.isfinite(e) and math.isfinite(a)):
        return False
    if e == 0.0:
        return a == e
    # Rust {:e} prints the shortest round-trip digits; Python's f"{:e}" rounds to 6 digits (wrong at 9.99..e-98),
    # so take the exponent from the shortest repr instead: d.ddd -> normalise mantissa digits.
    m = repr(abs(e)).lower()
    mant, _, ex = m.partition("e")
    ex = int(ex or 0)
    ip, _, fp = mant.partition(".")
    digits = (ip + fp).lstrip("0")
    lead = len(ip.lstrip("0")) if ip.strip("0") else -(len(fp) - len(fp.lstrip("0")))
    ex = ex + (lead - 1 if ip.strip("0") else lead - 1)
    return abs(a - e) <= float(f"5e{ex - 15}")


def visible(cell):
    v = cell["verdict"]
    if v in ("pass", "deviation", "unsupported", "numeric_ulp"):
        return False
    if v in ("numeric_close", "numeric_mismatch"):
        e, a = cell["expected"], cell["actual"]
        if isinstance(e, dict) and isinstance(a, dict) and e.get("t") == "n" and a.get("t") == "n":
            return not w15(float(e["v"]), float(a["v"]))
        return True
    return True


def main():
    rep = json.load(open(REPORT))
    tot = Counter(); verd = Counter()
    for c in rep["cases"]:
        pop = "w2f" if "/t2464-w2f-" in c["id"] else "r2" if "/t2464-r2-" in c["id"] else "pre" if "t2464" not in c["id"] else "other-t2464"
        for cell in c["cells"]:
            names = [m.group(1).upper().removeprefix("_XLFN.") for m in CALL.finditer(cell.get("formula") or "")]
            fam = [n for n in names if n in FAM]
            if not fam or not visible(cell):
                continue
            tot[pop] += 1
            if pop == "pre":
                verd[cell["verdict"]] += 1
                tot["pre-gated"] += c["suite"] not in ("probes-numeric", "gen-m8a", "gen-m8b")
                tot["pre-suite:" + c["suite"]] += 1
    for k, v in sorted(tot.items()):
        print(k, v)
    print(verd)


if __name__ == "__main__":
    main()
