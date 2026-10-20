"""Per-point stage reader. usage: s4read.py fit.pkl data.npz kN kD A B out.npy  (kN = stages done; 4 -> s4 includes p[4])"""
import ctypes, os, pickle, sys
import numpy as np
L = ctypes.CDLL(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'libs4r.so')); DP = ctypes.POINTER(ctypes.c_double)
p, q = [list(v) for v in pickle.load(open(sys.argv[1], 'rb'))]
dat = np.load(sys.argv[2]); X = np.ascontiguousarray(dat['X'], float); Y = np.ascontiguousarray(dat['Y'], float)
kN, kD, A, B = map(int, sys.argv[3:7])
W = (2 * A + 1) * (2 * B + 1); out = np.zeros(len(X) * W, np.uint8)
pa = np.ascontiguousarray(p, float); qa = np.ascontiguousarray(q, float)
L.s4read(pa.ctypes.data_as(DP), len(p), qa.ctypes.data_as(DP), len(q), kN, kD, X.ctypes.data_as(DP), Y.ctypes.data_as(DP), len(X), A, B, out.ctypes.data_as(ctypes.POINTER(ctypes.c_ubyte)))
O = out.reshape(len(X), 2 * A + 1, 2 * B + 1); np.save(sys.argv[7], O)
print('n', len(X), 'base hit %.4f' % O[:, A, B].mean(), 'any (a,b) %.4f' % O.reshape(len(X), -1).any(1).mean(),
      'b=0 any a %.4f' % O[:, :, B].any(1).mean(), 'a=0 any b %.4f' % O[:, A, :].any(1).mean())
print('a-marginal (b=0):', O[:, :, B].mean(0).round(3).tolist())
print('b-marginal (a=0):', O[:, A, :].mean(0).round(3).tolist())
