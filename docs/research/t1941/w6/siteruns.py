# T-1941 W6: shared data loader for the site tools (sitemap / sitescan / pkscan): split RFILE (rows "x B Rlo Rhi n case")
# into runs of consecutive doubles and sample whole runs at random across (1.5,2] (many distinct x0 break the period
# aliases of a single run); per-run mean + linear-trend removal (absorbs smooth coefficient errors).
import re, collections, numpy as np


def load_runs(rf, npts, minrun=1024, seed=3):
    by = collections.defaultdict(list)
    for l in open(rf):
        q = l.split()
        by[re.sub(r"[a-d]?(:B)?$", "", q[5])].append((float.fromhex(q[0]), float.fromhex(q[2]), float.fromhex(q[3])))
    runs = []
    for v in by.values():
        v = np.array(sorted(v))
        br = np.where(np.nextafter(v[:-1, 0], np.inf) != v[1:, 0])[0] + 1
        runs += [v[s] for s in np.split(np.arange(len(v)), br) if len(s) >= minrun]
    rng = np.random.default_rng(seed)
    order = rng.permutation(len(runs))
    pts, gid, tot = [], [], 0
    for g, i in enumerate(order):
        pts.append(runs[i])
        gid.append(np.full(len(runs[i]), g))
        tot += len(runs[i])
        if tot >= npts:
            break
    v = np.concatenate(pts)
    return v[:, 0], v[:, 1], v[:, 2], np.concatenate(gid)


class Detrend:
    """per-run removal of mean and linear trend in x"""

    def __init__(self, x, gid):
        self.G = [np.where(gid == g)[0] for g in np.unique(gid)]
        self.P = []
        for m in self.G:
            A = np.vstack([np.ones(len(m)), x[m] - x[m].mean()]).T
            self.P.append((A, np.linalg.pinv(A)))

    def __call__(self, a):
        a = np.array(a, dtype=float)
        for m, (A, Ai) in zip(self.G, self.P):
            a[m] -= A @ (Ai @ a[m])
        return a
