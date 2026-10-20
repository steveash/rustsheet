"""x87 kernel emulator: y = fl(t*fl(N/D)), t = x-2, raw-x Horner for N (len(p)) and D (len(q)), coefficients highest
degree first. Every op is computed in 80-bit; an op is stored to double (fl53) unless its name is in the keep list.
Op names: N/D + mul/add + stage i (stage i multiplies by x then adds c[i], i >= 1), plus 'div'."""
import ctypes, os
import numpy as np
_L = ctypes.CDLL(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'libx87.so'))
DP = ctypes.POINTER(ctypes.c_double); UP = ctypes.POINTER(ctypes.c_ubyte)
_L.xcount.restype = ctypes.c_int; _L.xl1.restype = ctypes.c_int
NAMES = {'div': 200}
for i in range(1, 16):
    for b, nm in ((0, 'mul'), (1, 'add')):
        NAMES['N%s%d' % (nm, i)] = 2 * (i - 1) + b; NAMES['D%s%d' % (nm, i)] = 100 + 2 * (i - 1) + b
def keepmask(names):
    k = np.zeros(256, np.uint8)
    for n in names:
        if n: k[NAMES[n]] = 1
    return k
def _P(a): return np.ascontiguousarray(a, float).ctypes.data_as(DP)
class XScorer:
    """exact-hit count of the emulated kernel against Y"""
    def __init__(s, X, Y, names): s.X = np.ascontiguousarray(X, float); s.Y = np.ascontiguousarray(Y, float); s.k = keepmask(names)
    def __call__(s, p, q): return _L.xcount(_P(p), len(p), _P(q), len(q), s.X.ctypes.data_as(DP), s.Y.ctypes.data_as(DP), len(s.X), s.k.ctypes.data_as(UP))
def xsim(p, q, X, names):
    X = np.ascontiguousarray(X, float); y = np.zeros_like(X); k = keepmask(names)
    _L.xsim(_P(p), len(p), _P(q), len(q), X.ctypes.data_as(DP), len(X), y.ctypes.data_as(DP), k.ctypes.data_as(UP)); return y
