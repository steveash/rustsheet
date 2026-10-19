# frequency tracks across [1,2) from yruns json (env YRUNS); args f:v seeds; writes tracks.json
import json, numpy as np, sys, os

R = json.load(open(os.environ.get("YRUNS", "yruns.json")))
NF = 1 << 18
runs = []
for y0, yc, r in R:
    r = np.array(r)
    r -= r.mean()
    n = len(r)
    P = np.abs(np.fft.rfft(r * np.hanning(n), NF)) ** 2
    P /= np.median(P)
    pk = []
    for i in range(1, len(P) - 1):
        if P[i] > P[i - 1] and P[i] >= P[i + 1] and P[i] > 30:
            a, b, c = np.log(P[i - 1]), np.log(P[i]), np.log(P[i + 1])
            d = 0.5 * (a - c) / (a - 2 * b + c)
            pk.append(((i + d) / NF, P[i]))
    runs.append((y0 + (n / 2) * 2**-52, pk))
runs.sort()
seeds = [tuple(map(float, a.split(":"))) for a in sys.argv[1:]]
out = {}
for s, v0 in seeds:
    tr = [(1.0, s)]
    miss = 0
    for yc, pk in runs[1:]:
        T = tr[-8:]
        yy = np.array([t[0] for t in T])
        ff = np.array([t[1] for t in T])
        if yy[-1] - yy[0] < 0.015:
            pred = tr[-1][1] + 2 * v0 * (yc - tr[-1][0])
        else:
            dg = 1 if yy[-1] - yy[0] < 0.05 else 2
            pred = np.polyval(np.polyfit(yy - yy[-1], ff, dg), yc - yy[-1])
        best = None
        for p, pw in pk:
            for c in (p, -p, 1 - p, 1 + p, p - 1, 2 - p, p + 1):
                d = abs(c - pred)
                if best is None or d < best[0]:
                    best = (d, c, pw)
        tol = 6e-4 + 0.02 * (yc - tr[-1][0])
        if best and best[0] < tol:
            tr.append((yc, best[1]))
            miss = 0
        else:
            miss += 1
            if miss >= 3:
                break
    out["%s:%s" % (s, v0)] = tr
    print(
        "seed",
        s,
        "points",
        len(tr),
        "last y %.4f" % tr[-1][0],
        " ".join("%.3f:%.5f" % t for t in tr[:: max(1, len(tr) // 12)]),
    )
json.dump(out, open("tracks.json", "w"))
# Taylor coefficients: fit F(t)=sum phi_k t^k (t=y-1), then m_k = ±phi_k/(k+1).
