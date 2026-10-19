# needle analysis on synthetic truth: exact rate along the line current->truth, one-ulp sensitivities, basin width per soft direction
# usage: needle.py current.txt truth.txt SYN.txt soft.pkl   (SYN.txt = "yhex x x qhex tag", q = model(truth, y); soft.pkl from nulldirs.py)
import ctypes, numpy as np, pickle, sys, math
from fractions import Fraction as Fr

L = ctypes.CDLL("./libgscore.so")
L.gload.argtypes = [ctypes.c_char_p, ctypes.c_int]
L.gscore.restype = ctypes.c_long
L.gabserr.restype = ctypes.c_double
for fn in (L.gscore, L.gabserr):
    fn.argtypes = [
        ctypes.c_int,
        ctypes.c_int,
        ctypes.POINTER(ctypes.c_double),
        ctypes.POINTER(ctypes.c_double),
    ]
NP = L.gload(sys.argv[3].encode(), 3)
a = [Fr(float.fromhex(h)) for h in open(sys.argv[1]).read().split()]
t = [Fr(float.fromhex(h)) for h in open(sys.argv[2]).read().split()]


def arr(x):
    return (ctypes.c_double * 9)(*x[:9]), (ctypes.c_double * 9)(*x[9:])


def sc(x):
    A, B = arr(x)
    return L.gscore(8, 8, A, B) / NP


def ae(x):
    A, B = arr(x)
    return L.gabserr(8, 8, A, B)


for s in (0, 0.2, 0.4, 0.6, 0.8, 0.9, 0.99, 0.999, 1.0, 1.001, 1.01):
    print("t=%.3f exact %.4f" % (s, sc([float(p + Fr(s) * (q - p)) for p, q in zip(a, t)])))
tt = np.array([float(q) for q in t])
for i in range(18):
    if tt[i] == 0:
        continue
    x = list(tt)
    x[i] += math.ulp(x[i])
    print("coef %d +1ulp exact %.4f" % (i, sc(x)))
dirs = [np.array(d) for d, s in pickle.load(open(sys.argv[4], "rb"))["dirs"]]
for k, d in enumerate(dirs):
    print(
        "dir %2d exact at 0.03/0.1/0.3/1/3 units: %s | mean|err| at 0.1/0.3/1/3: %s"
        % (
            k,
            " ".join("%.3f" % sc(tt + u * d) for u in (0.03, 0.1, 0.3, 1, 3)),
            " ".join("%.3f" % ae(tt + u * d) for u in (0.1, 0.3, 1, 3)),
        )
    )
