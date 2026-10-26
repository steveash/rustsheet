"""Sum m1_score outputs into subsets: round-1 data (beta_exact + w2 + probe1), this directory's probe (never used for
choosing the model), shapes with min >= 8, and all.  Usage: sumtab.py FILE.out"""
import re
import sys

pat = re.compile(r"^\s+(\S+)\s+\(([\d.]+),([\d.]+)\)\s+(\d+)/\s*(\d+)/\s*(\d+) of\s+(\d+)\s+CR\s+(\d+)/\s*(\d+)/\s*(\d+) of")
groups = {"round-1 data": [], "own probe p1 (held out)": [], "min(a,b) >= 8": [], "1 < min < 8": [], "all": []}
for line in open(sys.argv[1]):
    m = pat.match(line)
    if not m:
        continue
    blk, a, b = m.group(1), float(m.group(2)), float(m.group(3))
    v = [int(m.group(i)) for i in range(4, 11)]
    own = blk.startswith(("cut", "s8(", "shp(", "hold("))
    groups["own probe p1 (held out)" if own else "round-1 data"].append(v)
    groups["min(a,b) >= 8" if min(a, b) >= 8 else "1 < min < 8"].append(v)
    groups["all"].append(v)
print(f"{'subset':26s} {'n':>6s}  model ex/w4/w15        CR ex/w4/w15")
for g, vs in groups.items():
    s = [sum(v[i] for v in vs) for i in range(7)]
    print(f"{g:26s} {s[3]:6d}  {s[0]:6d}/{s[1]:6d}/{s[2]:6d}   {s[4]:6d}/{s[5]:6d}/{s[6]:6d}")
