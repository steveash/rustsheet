"""Stream kmyst-gam-* and t1042-gamma-* cases from the research branch (git cat-file) and write
per-function rows: x_hex y_hex case. Columns chosen by formula (as extract_runs.py)."""
import json, re, subprocess, sys, os
REF = "origin/agent/research/T-1941-crack-gammaln-gamma-ln"
OUT = sys.argv[1]
FUN = {'GAMMALN.PRECISE': 'gl', 'GAMMALN': 'gl', 'LN': 'ln', 'GAMMA': 'ga'}
names = subprocess.run(["git", "ls-tree", "--name-only", REF, "eval/suites/probes-numeric/cases/", "eval/suites/probes/cases/"],
                       capture_output=True, text=True, check=True).stdout.split()
cases = sorted(n for n in names if re.search(r"/(kmyst-gam|t1042-gamma)[^/]*\.case\.json$", n))
p = subprocess.Popen(["git", "cat-file", "--batch"], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
def blob(path):
    p.stdin.write(f"{REF}:{path}\n".encode()); p.stdin.flush()
    hdr = p.stdout.readline().split()
    if hdr[-1] == b"missing": return None
    n = int(hdr[2]); data = p.stdout.read(n); p.stdout.read(1); return data
outs = {k: open(os.path.join(OUT, k + ".rows"), "w") for k in ("gl", "ln", "ga")}
pat = re.compile(r"=([A-Z.]+)\(A(\d+)\)")
for f in cases:
    o = blob(f.replace(".case.json", ".oracle.json"))
    if o is None: continue
    c = json.loads(blob(f))["sheets"][0]["cells"]; v = json.loads(o)["sheets"].get("Calc", {})
    nm = os.path.basename(f)[:-10]
    for cell, formula in c.items():
        if not isinstance(formula, str): continue
        m = pat.fullmatch(formula)
        if not m or m.group(1) not in FUN: continue
        x = c.get("A" + m.group(2))
        y = v.get(cell, {}).get("v")
        if isinstance(x, (int, float)) and isinstance(y, (int, float)) and not isinstance(y, bool):
            outs[FUN[m.group(1)]].write(f"{float(x).hex()} {float(y).hex()} {nm}\n")
print(len(cases), "cases")
