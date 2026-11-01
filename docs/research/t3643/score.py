"""T-3643 R4.1/R4.2: score the logs' modulus models (doubly rounded root vs hypot) on the real parts.

Hold-out: sha1("<case id>!<cell>")[0] < 77 (about 30%), margin 5 points, fixed before scoring.
Run from tools/: uv run python ../docs/research/t3643/score.py ../eval/suites/probes/cases/t3643-imlog10-grid.case.json \
  ../eval/suites/probes/cases/t3643-imlog2-grid.case.json ../eval/suites/probes/cases/t3643-imln-points.case.json \
  ../eval/suites/probes-numeric/cases/t3643-r4-log-modulus.case.json
"""
import json, math, re, hashlib, sys
from mpmath import mp, mpf
K = math.nextafter(math.log2(math.e), math.inf)
def ext_sqrt(s):
    mp.prec = 64; r = mp.sqrt(mpf(s)); return float(r)
def fsq(p):
    s = p*p
    return 0.0 if (s != 0 and abs(s) < 2.2250738585072014e-308) else s
def chosen(x,y):
    if y == 0: return abs(x)
    if x == 0: return abs(y)
    return ext_sqrt(fsq(x)+fsq(y))
def hyp(x,y): return math.hypot(x,y)
FN = {"IMLN": math.log, "IMLOG10": math.log10, "IMLOG2": lambda m: math.log(m)*K}
def num(t): return float(t.replace("E","e"))
def real_of(t):
    if t[-1] not in "ij": return num(t)
    body = t[:-1]
    for i in range(len(body)-1, 0, -1):
        if body[i] in "+-" and body[i-1] not in "Ee":
            return num(body[:i])
    return 0.0
def lit(s):
    s = s.strip()
    return float(f"{float(s.replace('E','e')):.15g}")
def parse_text_arg(a):
    # "x+yi" literal from the grid generator
    if a[-1] != "i": return lit(a), 0.0
    body = a[:-1]
    for i in range(len(body)-1, 0, -1):
        if body[i] in "+-" and body[i-1] not in "Ee":
            return lit(body[:i]), lit(body[i:])
    return 0.0, lit(body) if body not in ("","+","-") else float(body+"1")
def same15(a, b):
    return float(f"{a:.15g}") == b or (a == 0 and b == 0)
rows = []
for case in sys.argv[1:]:
    cid = case.split("eval/suites/")[1]
    c = json.load(open(case))["sheets"][0]["cells"]
    o = json.load(open(case.replace(".case.json", ".oracle.json")))["sheets"]["S"]
    for cell, f in c.items():
        m = re.match(r'=(IMLN|IMLOG10|IMLOG2)\((.*)\)$', f)
        if not m: continue
        fn, arg = m.groups()
        if arg.startswith("COMPLEX("):
            x, y = (lit(v) for v in arg[8:-1].split(","))
        else:
            x, y = parse_text_arg(arg.strip('"'))
        ex = o[cell]
        if ex["t"] != "s" and ex["t"] != "n": continue  # errors: not the modulus model
        if x != 0 and y != 0 and not (fsq(x)+fsq(y)) >= 2.2250738585072014e-308: continue
        if x == 0 and y == 0: continue
        er = real_of(ex["v"]) if ex["t"] == "s" else float(ex["v"])
        hold = hashlib.sha1(f"{cid}!{cell}".encode()).digest()[0] < 77
        rows.append((cid, cell, fn, x, y, er, hold, same15(FN[fn](chosen(x,y)), er), same15(FN[fn](hyp(x,y)), er)))
def tally(sel):
    n=len(sel); return n, sum(r[7] for r in sel), sum(r[8] for r in sel)
grid=[r for r in rows if "r4-" not in r[0]]
sweep=[r for r in rows if "r4-" in r[0]]
for name, sel in (("train", [r for r in grid if not r[6]]), ("hold-out", [r for r in grid if r[6]]), ("fresh sweep", sweep)):
    n,a,b = tally(sel)
    print(f"{name}: n={n} chosen within-15-digits {a} ({100*a/n:.1f}%), hypot {b} ({100*b/n:.1f}%)")
for r in sweep:
    if not r[7]: print("chosen miss", r[:6])
for r in sweep:
    if r[7] != r[8]: print("differ", r[2], r[3], r[4], "chosen ok" if r[7] else "hypot ok")
