# directions invisible to phases (0.01 cycle) and Gamma fit (1e-16); env E7=runs pickle, FREEZE, A2
import sys, pickle, os

os.environ["MORE"] = "1"
exec(open("jointA.py").read())
x = np.array([float.fromhex(h) for h in open(sys.argv[1]).read().split()])
e7 = pickle.load(open(os.environ.get("E7", "e7runs.pkl"), "rb"))
runs = load_e3() + e7
rows = []
for a, yv, g, rd in runs:
    for ch, j, typ in LINES:
        dphi, row, Ad, As = measure_run(x, ch, j, typ, a, yv, g, rd)
        if min(Ad, As) < 50:
            continue
        rows.append(row / 0.01)
mp.mp.dps = 40
for t in [mp.mpf(1) + mp.mpf(k) / 60 for k in range(61)]:
    gg = mp.gamma(t)
    Dv = sum(mp.mpf(x[9 + i]) * t**i for i in range(9))
    rows.append(
        np.array([float(t**i / (gg * Dv)) for i in range(9)] + [float(-(t**i) / Dv) for i in range(9)])
        / 1e-16
    )
A = np.array(rows)
sc = np.abs(x)
FZ = [int(v) for v in os.environ.get("FREEZE", "").split(",") if v]
keepc = [c for c in range(18) if c not in FZ]
U, S, Vt = np.linalg.svd((A * sc)[:, keepc], full_matrices=False)


def emb(v):
    o = np.zeros(18)
    o[keepc] = v
    return o


dirs = [emb(Vt[k]) * sc / S[k] for k in range(len(S) - 1, -1, -1)]
pickle.dump(
    {"xs": list(x), "dirs": [(list(d), float(S[len(S) - 1 - k])) for k, d in enumerate(dirs)]},
    open("soft2.pkl", "wb"),
)
