"""Data preparation and the exact chain-top hypothesis objective (libhypo.so), T-1941 W5 part 5."""
import numpy as np, sys, os, ctypes
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..'))
from sc import res
from meter import resid, L
_lib = ctypes.CDLL(os.path.join(HERE, 'libhypo.so')); Pd = ctypes.POINTER(ctypes.c_double); Pl = ctypes.POINTER(ctypes.c_long)
_lib.hypo_obj.argtypes = [Pd, Pd, Pl, ctypes.c_int, ctypes.c_long, Pd, ctypes.c_int, Pd, ctypes.c_int, ctypes.c_int, ctypes.c_int, Pd]
ROWS = os.environ.get('ROWS', '/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows2.txt')
def ld(f):
    l = open(f).read().split('\n'); return np.array([float.fromhex(v) for v in l[0].split()]), np.array([float.fromhex(v) for v in l[1].split()])
class Data:
    def __init__(self, N0, D0, mode, truthfile=None, LO=0.7, HI=1.47, RL=int(os.environ.get('RL', 16384))):
        z = np.load(ROWS+'.near1.npz'); x = z['x']; b = z['b']
        br = np.where(np.nextafter(x[:-1], np.inf) != x[1:])[0]+1
        NEAR = float(os.environ.get('NEAR1', 0))   # >0: keep only runs with 2e-12 < |x-1| < NEAR (near-1 mode)
        keep = (lambda s: 2e-12 < abs(x[s[0]]-1) < NEAR) if NEAR > 0 else (lambda s: LO <= x[s[0]] < HI and abs(x[s[0]]-1) > 2e-3)
        segs = [s[:RL] for s in np.split(np.arange(len(x)), br) if len(s) >= 256 and keep(s)]
        idx = np.concatenate(segs); self.rs = np.cumsum([0]+[len(s) for s in segs]).astype(np.int64)
        self.x = np.ascontiguousarray(x[idx]); self.nr = len(segs)
        B = res(self.x, *ld(truthfile)) if mode == 'control' else b[idx]
        XL = self.x.astype(L)
        Nv = np.polyval(N0.astype(L)[::-1], XL); Dv = np.polyval(D0.astype(L)[::-1], XL)
        self.ref = (XL-1)*Nv/Dv; self.sg = np.sign(self.ref); self.Nv = Nv; self.Dv = Dv
        r = ((L(B)-self.ref)/(np.abs(self.ref)*L(2.0**-52))).astype(float)
        for i in range(self.nr): r[self.rs[i]:self.rs[i+1]] -= r[self.rs[i]:self.rs[i+1]].mean()
        self.r = np.ascontiguousarray(r)
    def weights(self, CH, J0, top):
        """W[k] = relative-output weight per unit absolute error at stage j = J0-1+k, in units of 2^-52"""
        F = (self.sg/self.Nv) if CH == 'N' else (-self.sg/self.Dv)
        XL = self.x.astype(L)
        return np.ascontiguousarray(np.array([(F*XL**j).astype(float)*2.0**52 for j in range(J0-1, top)]))
    def obj(self, CH, J0, Hyps, withlast=1):
        Hyps = np.ascontiguousarray(Hyps, dtype=np.float64); top = J0+Hyps.shape[1]-1
        W = self.weights(CH, J0, top); out = np.empty(Hyps.shape[0])
        _lib.hypo_obj(self.x.ctypes.data_as(Pd), self.r.ctypes.data_as(Pd), self.rs.ctypes.data_as(Pl), self.nr, len(self.x), W.ctypes.data_as(Pd), J0,
                      Hyps.ctypes.data_as(Pd), Hyps.shape[1], Hyps.shape[0], withlast, out.ctypes.data_as(Pd))
        return out
