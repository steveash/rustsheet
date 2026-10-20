# T-1941 W6: W5's exact chain-top objective (w5/p5/hypo.c, built locally as libhypo.so) on W6 data. For a hypothesis
# of one chain's top coefficients c_J0..c_top, it simulates the stored stages top..J0 and correlates their rounding
# errors (weighted by each stage's sensitivity of R) with the data residual, run by run; independent of the lower
# coefficients. Module: Top6(rfile, chain).obj(ch, J0, H, withlast); CLI (control capture-range check):
#   top6.py RFILE BASE CHAIN_CH J0 TRUTH   (needs env HYPOLIB = path of libhypo.so)
import sys, os, re, ctypes, collections, numpy as np

sys.path.insert(0, sys.path[0])
from lib6 import L
from synth import read_chain, nrm

Pd = ctypes.POINTER(ctypes.c_double)
Pl = ctypes.POINTER(ctypes.c_long)


class Top6:
    def __init__(self, rfile, chain, minrun=256, npts=None, maxruns=None):
        lib = ctypes.CDLL(os.environ["HYPOLIB"])
        lib.hypo_obj.argtypes = [
            Pd,
            Pd,
            Pl,
            ctypes.c_int,
            ctypes.c_long,
            Pd,
            ctypes.c_int,
            Pd,
            ctypes.c_int,
            ctypes.c_int,
            ctypes.c_int,
            Pd,
        ]
        self.lib = lib
        N, D = read_chain(chain)
        self.N = np.array(N) /nrm(D)
        self.D = np.array(D) /nrm(D)
        pts = collections.defaultdict(list)
        for l in open(rfile):
            q = l.split()
            pts[re.sub(r"[a-d]?(:B)?$", "", q[5])].append(
                (float.fromhex(q[0]), float.fromhex(q[2]), float.fromhex(q[3]))
            )
        xs, rs, starts = [], [], [0]
        for v in pts.values():
            v = np.array(sorted(v))
            br = np.where(np.nextafter(v[:-1, 0], np.inf) != v[1:, 0])[0] + 1
            for seg in np.split(np.arange(len(v)), br):
                if len(seg) < minrun:
                    continue
                x = v[seg, 0]
                XL = x.astype(L)
                R = np.polyval(self.N.astype(L)[::-1], XL) / np.polyval(self.D.astype(L)[::-1], XL)
                Rm = (v[seg, 1].astype(L) + v[seg, 2].astype(L)) / 2
                r = ((Rm - R) / (np.abs(R) * L(2.0**-52))).astype(float)
                xs.append(x)
                rs.append(r - r.mean())
                starts.append(starts[-1] + len(seg))
        if maxruns and len(xs) > maxruns:  # an evenly spread subset of runs (the objective sums per-run correlations)
            pick = np.linspace(0, len(xs) - 1, maxruns).astype(int)
            xs = [xs[i] for i in pick]
            rs = [rs[i] for i in pick]
            starts = [0] + list(np.cumsum([len(v) for v in xs]))
        self.x = np.ascontiguousarray(np.concatenate(xs))
        self.r = np.ascontiguousarray(np.concatenate(rs))
        self.rs = np.array(starts, dtype=np.int64)
        self.nr = len(starts) - 1
        XL = self.x.astype(L)
        self.Nv = np.polyval(self.N.astype(L)[::-1], XL)
        self.Dv = np.polyval(self.D.astype(L)[::-1], XL)

    def weights(self, ch, J0, top):
        """W[k] = relative-R weight (units 2^-52) per absolute error at stage J0-1+k: x^j/N (N chain), -x^j/D (D)."""
        F = (1 / self.Nv) if ch == "N" else (-1 / self.Dv)
        XL = self.x.astype(L)
        return np.ascontiguousarray(
            np.array([(F * XL**j).astype(float) * 2.0**52 for j in range(J0 - 1, top)])
        )

    def obj(self, ch, J0, H, withlast=1):
        H = np.ascontiguousarray(H, dtype=np.float64)
        top = J0 + H.shape[1] - 1
        W = self.weights(ch, J0, top)
        out = np.empty(H.shape[0])
        self.lib.hypo_obj(
            self.x.ctypes.data_as(Pd),
            self.r.ctypes.data_as(Pd),
            self.rs.ctypes.data_as(Pl),
            self.nr,
            len(self.x),
            W.ctypes.data_as(Pd),
            J0,
            H.ctypes.data_as(Pd),
            H.shape[1],
            H.shape[0],
            withlast,
            out.ctypes.data_as(Pd),
        )
        return out


if __name__ == "__main__":
    rf, base, ch, J0, truth = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), sys.argv[5]
    t = Top6(rf, base)
    print("runs %d, points %d" % (t.nr, len(t.x)))
    Nt, Dt = read_chain(truth)
    Ct = (np.array(Nt) if ch == "N" else np.array(Dt)) /nrm(Dt)
    Cb = t.N if ch == "N" else t.D
    top = len(Cb) - 1
    u = np.spacing(np.abs(Ct))
    H = [Cb[J0:], Ct[J0:]]
    lab = ["base", "truth"]
    for d in (1, 10, 100, 1000, 10000, 100000):
        h = Ct[J0:].copy()
        h[-1] += d * u[top]
        H.append(h)
        lab.append("truth, c%d %+d" % (top, d))
    for d in (1, 10, 100, 1000):
        h = Ct[J0:].copy()
        h[-2] += d * u[top - 1]
        H.append(h)
        lab.append("truth, c%d %+d" % (top - 1, d))
    v = t.obj(ch, J0, np.array(H), 0)
    for a, b in zip(lab, v):
        print("  %-22s %8.1f" % (a, b))
