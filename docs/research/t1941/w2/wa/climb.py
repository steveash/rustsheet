# exact-score climb: wide moves along w1/w2, then all coefs by small integer steps (args as ridge.py)
exec(open("ridge.py").read().split("best = (0, None)")[0])
dc = start.copy()
cur = sc(*mk(dc)) / len(sel)
wide = [
    (a * s, b * s)
    for s in (400, 200, 100, 50, 25, 10, 5, 2, 1)
    for a, b in ((1, 0), (0, 1), (1, 2), (1, 3), (1, 4))
]
wide += [(-a, -b) for a, b in wide]
small = [
    (k, s)
    for k in range(len(WHICH))
    if k not in (ip1, iq0)
    for s in (1, 2, 3, 5, 10, 20, -1, -2, -3, -5, -10, -20)
]
for rnd in range(30):
    imp = False
    for a, b in wide:
        d2 = np.round(dc + a * w1 + b * w2)
        v = sc(*mk(d2)) / len(sel)
        if v > cur:
            cur, dc, imp = v, d2, True
    for k, s in small:
        d2 = dc.copy()
        d2[k] += s
        v = sc(*mk(d2)) / len(sel)
        if v > cur:
            cur, dc, imp = v, d2, True
    print(rnd, "%.4f" % cur, [int(v) for v in dc], flush=True)
    if not imp:
        break
pickle.dump(dc, open(sys.argv[1] + ".climb", "wb"))
