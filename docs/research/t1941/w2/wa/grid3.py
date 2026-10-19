# 2-D coherence grid over (dp1, dq0), followers by LS, then integer refinement; env R, ST, K
exec(open("phasecore.py").read())
R = int(os.environ.get("R", "60000"))
st = int(os.environ.get("ST", "20"))
K = int(os.environ.get("K", "400"))
a1 = np.arange(-R, R + 1, st)
a2 = np.arange(-R, R + 1, st)
cand = []
for i in range(0, len(a1), 50):
    blk = a1[i : i + 50]
    ph1 = np.outer(blk, gg1)
    for j in range(0, len(a2), 2000):
        b2 = a2[j : j + 2000]
        c = (np.cos(2 * np.pi * (ph1[:, None, :] + np.outer(b2, gg2)[None, :, :] - gph)) * gw).sum(2)
        for k in np.argsort(c, axis=None)[-20:]:
            u, v = np.unravel_index(k, c.shape)
            cand.append((c[u, v], blk[u], b2[v]))
cand.sort(reverse=True)
res = []
for c0, dp1, dq0 in cand[:K]:
    dc, cur = refine(np.round(dp1 * w1 + dq0 * w2))
    res.append((cur, dc))
res.sort(key=lambda r: -r[0])
for c, dc in res[:6]:
    print("coh %.4f" % c, [int(v) for v in dc])
pickle.dump(res[0][1], open(sys.argv[1] + ".dc", "wb"))
