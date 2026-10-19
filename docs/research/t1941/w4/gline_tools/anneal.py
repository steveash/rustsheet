# stochastic hill-climb / simulated annealing on exact matches. args: stride x.txt soft.pkl out.txt seconds [data] [seed] [T0]
import ctypes, pickle, numpy as np, sys, math, time

L = ctypes.CDLL("./libgscore.so")
L.gload.argtypes = [ctypes.c_char_p, ctypes.c_int]
L.gscore.restype = ctypes.c_long
L.gscore.argtypes = [
    ctypes.c_int,
    ctypes.c_int,
    ctypes.POINTER(ctypes.c_double),
    ctypes.POINTER(ctypes.c_double),
]
NP = L.gload((sys.argv[6] if len(sys.argv) > 6 else "../G.txt").encode(), int(sys.argv[1]))
x = np.array([float.fromhex(h) for h in open(sys.argv[2]).read().split()])
S = pickle.load(open(sys.argv[3], "rb"))
dirs = [np.array(d) for d, s in S["dirs"]][:16]
FZ = {8, 17, 9}
rng = np.random.default_rng(int(sys.argv[7]) if len(sys.argv) > 7 else 1)
T0 = float(sys.argv[8]) if len(sys.argv) > 8 else 0.0


def score(x):
    A = (ctypes.c_double * 9)(*x[:9])
    B = (ctypes.c_double * 9)(*x[9:])
    return L.gscore(8, 8, A, B)


best = score(x)
cur = best
bx = x.copy()
t0 = time.time()
it = 0
while time.time() - t0 < float(sys.argv[5]):
    it += 1
    xx = x.copy()
    if rng.random() < 0.6:
        k = rng.integers(0, len(dirs))
        xx = xx + dirs[k] * rng.normal() * 10 ** rng.uniform(-3, 0)
    else:
        i = rng.choice([c for c in range(18) if c not in FZ])
        xx[i] += math.ulp(xx[i]) * int(rng.choice([-1, 1])) * 2 ** int(rng.integers(0, 20))
    s = score(xx)
    T = T0 * max(0.0, 1 - (time.time() - t0) / float(sys.argv[5]))
    if s >= cur or (T > 0 and rng.random() < math.exp((s - cur) / T)):
        cur = s
        x = xx
        if s > best:
            best = s
            bx = xx.copy()
            print(
                "it %d t %.0f best %d (%.2f%%) T %.1f" % (it, time.time() - t0, s, 100 * s / NP, T),
                flush=True,
            )
            open(sys.argv[4], "w").write(" ".join(float(v).hex() for v in bx) + "\n")
open(sys.argv[4], "w").write(" ".join(float(v).hex() for v in bx) + "\n")
print("done", best, it)
