"""s6: brute-force stored-sum structures on a few r > 0.375 runs, interval-scored with a free remainder K_lo.
Summands: Q (the constant inside the chain, any real: its phase is scanned continuously, 48 steps over the coarsest
stored grid), H = -h L (extended), S = -fl(h L), P = sg*fl(c LN x), X = -LN(x) (double), Y = -LN64(x) (extended).
A structure is an ordering of a term set with per-partial-sum flags (1 stored, 0 extended); E = sum + K_lo.  When X
or Y is a summand the final step is plain EXP (fin 'exp'), else fl(EXP(E)/x).
Printed per structure: count per run and total, best 25.  Usage: s6_brute.py SET RUNSPEC... (RUNSPEC m,n,side,x)"""

import itertools
import sys

import mpmath
import numpy as np

import clib as C

LDB, st = C.LDB, C.st
SETS = ("QHP", "QSP", "QHPX", "QSPX", "QHPY")


def term_vals(pc, x):
    h = LDB(pc["h"])
    HL = h * C.SL.L_xl(pc["r"])
    return {"H": -HL, "S": -LDB(st(HL)), "P": LDB(pc["P"]), "X": -LDB(C.SL.LN(x)), "Y": -LDB(C.R3.LN64(x))}


def structures():
    for ts in SETS:
        for order in itertools.permutations(ts):
            for flags in itertools.product((0, 1), repeat=len(order) - 1):
                if not any(flags):
                    if order != tuple(ts):
                        continue
                elif order.index("Q") > 0 and not any(flags[order.index("Q") - 1 :]):
                    continue  # Q added after the last store: same as Q outside
                yield "".join(order), flags


def chain(order, flags, t):
    acc = t[order[0]]
    for i, k in enumerate(order[1:]):
        acc = acc + t[k]
        if flags[i]:
            acc = LDB(st(acc))
    return acc


def score(run, order, flags):
    cells, Kref, tv = run
    q_matters = any(flags) and (order.index("Q") == 0 or any(flags[order.index("Q") - 1 :]))
    K0 = C.SL.ld(Kref)
    if q_matters:
        g = max(float(np.spacing(abs(float(chain(order, flags, dict(t, Q=K0)))))) for t in tv[:3])
        g = max(g, max(float(np.spacing(abs(float(t["P"])))) for t in tv[:3]))
        steps = np.linspace(-g, g, 48, endpoint=False)
    else:
        steps = [0.0]
    best = 0
    for d in steps:
        Q = K0 + LDB(d)
        ivs = []
        for (pc, E0, (lo, hi), _), t in zip(cells, tv):
            kk = float((E0 - C.ex(chain(order, flags, dict(t, Q=Q))) + C.ex(Q) - Kref) * C.U)
            ivs.append((kk + lo, kk + hi))
        best = max(best, C.best_overlap(ivs)[0])
    return best


def load_runs(name, specs):
    rows = C.load(name)
    out = []
    for spec in specs:
        m, n, side, x = (float(v) for v in spec.split(","))
        for key, rs in C.runs([r for r in rows if r["d1"] == m and r["d2"] == n]).items():
            if key[2] == side and abs(rs[0]["x"] / x - 1) < 1e-3:
                runs = {}
                for fin in ("div", "exp"):
                    cells = C.run_cells(rs, fin)
                    runs[fin] = (
                        cells,
                        C.K_exact(m, n, int(side)),
                        [term_vals(pc, r["x"]) for pc, _, _, r in cells],
                    )
                out.append((spec, runs))
    return out


def main():
    name, specs = sys.argv[1], sys.argv[2:]
    runs = load_runs(name, specs)
    res = []
    for order, flags in structures():
        fin = "exp" if ("X" in order or "Y" in order) else "div"
        cs = [score(r[fin], order, flags) for _, r in runs]
        res.append((sum(cs), order, flags, cs))
    res.sort(key=lambda t: -t[0])
    tot = sum(len(r["div"][0]) for _, r in runs)
    for s, o, f, cs in res[:25]:
        print(f"{o}{''.join(map(str, f))} {s}/{tot} {cs}")
    C.save()


if __name__ == "__main__":
    main()
