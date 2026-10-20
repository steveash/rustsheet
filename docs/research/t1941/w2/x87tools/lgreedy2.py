"""late-coefficient needle reader: greedy over full residues with the dgrid inner scan.
usage: lgreedy2.py start.pkl data.npz out.pkl kN kD G R rounds; env CLMAX"""
import ctypes, pickle, sys
import numpy as np
from search import nextd
from multiprocessing import Pool
L = ctypes.CDLL('./libdgrid.so'); DP = ctypes.POINTER(ctypes.c_double)
p, q = [list(v) for v in pickle.load(open(sys.argv[1], 'rb'))]
dat = np.load(sys.argv[2]); X = np.ascontiguousarray(dat['X'], float); Y = np.ascontiguousarray(dat['Y'], float)
kN, kD, G, R, ROUNDS = map(int, sys.argv[4:9])
CLMAX = int(__import__('os').environ.get('CLMAX', 99))
br = [0] + [i + 1 for i in range(len(X) - 1) if np.nextafter(X[i], 9) != X[i + 1]] + [len(X)]
CL = [(np.ascontiguousarray(X[a:b]), np.ascontiguousarray(Y[a:b])) for a, b in zip(br, br[1:])][:CLMAX]
n = sum(len(c[0]) for c in CL)
def score(pq):
    p, q = pq; pa = np.ascontiguousarray(p, float); qa = np.ascontiguousarray(q, float); tot = 0
    for xs, ys in CL:
        c = np.zeros((2 * G + 1) ** 2, np.int32)
        L.dgrid(pa.ctypes.data_as(DP), len(p), qa.ctypes.data_as(DP), len(q), kN, kD, xs.ctypes.data_as(DP), ys.ctypes.data_as(DP), len(xs), G, G, R, c.ctypes.data_as(ctypes.POINTER(ctypes.c_int)))
        tot += int(c.max())
    return tot
# offsets cover each coefficient's residue range mod ulp(s_k)/ulp(c_k) at x~2.4
OFFS = {('p', 5): range(-2, 3), ('p', 6): range(-64, 64), ('p', 7): range(-4, 5), ('p', 8): range(-64, 64), ('p', 9): range(-1024, 1024, 16),
        ('q', 4): range(-1, 2), ('q', 5): range(-2, 3), ('q', 6): range(-8, 8), ('q', 7): range(-128, 128, 2), ('q', 8): range(-4096, 4096, 64)}
OFFS = {k: v for k, v in OFFS.items() if (k[0] == 'p' and k[1] > kN) or (k[0] == 'q' and k[1] > kD)}
with Pool(4) as pool:
    cur = score((p, q)); print('start %.4f' % (cur / n), flush=True)
    for rnd in range(ROUNDS):
        moved = False
        for (w, i), offs in OFFS.items():
            cands = []
            for t in offs:
                if t == 0: continue
                pp, qq = list(p), list(q); a = pp if w == 'p' else qq; a[i] = nextd(a[i], t); cands.append((t, (pp, qq)))
            res = pool.map(score, [c[1] for c in cands])
            j = int(np.argmax(res))
            if res[j] > cur: cur = res[j]; p, q = cands[j][1]; moved = True; print(' %s%d %+d -> %.4f' % (w, i, cands[j][0], cur / n), flush=True)
        print('round', rnd, '%.4f' % (cur / n), flush=True); pickle.dump((p, q), open(sys.argv[3], 'wb'))
        if not moved: break
