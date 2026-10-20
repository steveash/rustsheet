"""usage: dgrid.py fit.pkl data.npz kN kD G R; 'overall best' = hand-off fingerprint value (W2 uses G=96 R=16)."""
import ctypes, pickle, sys
import numpy as np
L = ctypes.CDLL('./libdgrid.so'); DP = ctypes.POINTER(ctypes.c_double)
p, q = [list(v) for v in pickle.load(open(sys.argv[1], 'rb'))]
dat = np.load(sys.argv[2]); X = np.ascontiguousarray(dat['X'], float); Y = np.ascontiguousarray(dat['Y'], float)
kN, kD, G, R = map(int, sys.argv[3:7])
pa = np.ascontiguousarray(p, float); qa = np.ascontiguousarray(q, float)
br = [0] + [i + 1 for i in range(len(X) - 1) if np.nextafter(X[i], 9) != X[i + 1]] + [len(X)]
tot = 0; best_tot = 0
for a, b in zip(br, br[1:]):
    xs = np.ascontiguousarray(X[a:b]); ys = np.ascontiguousarray(Y[a:b]); c = np.zeros((2 * G + 1) ** 2, np.int32)
    L.dgrid(pa.ctypes.data_as(DP), len(p), qa.ctypes.data_as(DP), len(q), kN, kD, xs.ctypes.data_as(DP), ys.ctypes.data_as(DP), len(xs), G, G, R, c.ctypes.data_as(ctypes.POINTER(ctypes.c_int)))
    c = c.reshape(2 * G + 1, 2 * G + 1); i, j = np.unravel_index(np.argmax(c), c.shape)
    print('cluster x0 %.6f n %d at(0,0) %.4f  best %.4f at dN %+.3f dD %+.3f' % (xs[0], len(xs), c[G, G] / len(xs), c[i, j] / len(xs), (i - G) / R, (j - G) / R), flush=True)
    tot += len(xs); best_tot += c[i, j]
print('overall best %.4f' % (best_tot / tot))
