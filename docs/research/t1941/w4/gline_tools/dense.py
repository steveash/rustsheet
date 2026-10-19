# build yruns-format json (y0, ycentre, residual list in 2^-53 units) from extract_runs output (x B C D case)
import sys, math, json, mpmath as mp

mp.mp.dps = 30
rows = []
for l in open(sys.argv[1]):
    p = l.split()
    if not p[4].startswith("kmyst-gam-y1"):
        continue
    x = float.fromhex(p[0])
    d = p[3]
    if d == "nan":
        continue
    rows.append((x, float.fromhex(d)))
rows.sort()
out = []
s = 0
for i in range(1, len(rows) + 1):
    if i == len(rows) or rows[i][0] - rows[i - 1][0] != math.ulp(rows[i - 1][0]):
        if i - s >= 1024:
            ys = [a for a, b in rows[s:i]]
            r = [float((mp.mpf(b) - mp.gamma(mp.mpf(a))) / mp.mpf(2) ** -53) for a, b in rows[s:i]]
            out.append((ys[0], ys[len(ys) // 2], r))
        s = i
json.dump(out, open(sys.argv[2], "w"))
print(len(out), "runs", sum(len(o[2]) for o in out), "points")
