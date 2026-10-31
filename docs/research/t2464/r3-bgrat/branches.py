"""T-2512 r3-bgrat: which BETA.DIST cdf cells reach bgrat, and with which (a, b, lnx, z, w) at the call.

Model: r2-tdist's FINAL configuration (r2-bratio BEST + Excel log1p, Excel ERFC at b = 1/2, bgrat's lnx cut at y = 0.01)
via xlk.XKernel; bgrat calls are recorded, not changed. Writes calls.pkl (git-ignored)."""
from __future__ import annotations

import pickle
import sys
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r2-tdist"))
import xlk  # noqa: E402

import data  # noqa: E402

FINAL = xlk.XCfg(alnrel="xl", erfc="xl", bg_lnx="cut0.01", bg="ideal")
CALLS: dict = {}
_cur = [None]


class RecKernel(xlk.XKernel):
    def bgrat(self, a, b, x, y, w, eps):
        m = self.c.bg_lnx
        yy = 0.5 if y > float(m[3:]) else min(y, 0.375)
        lnx = self.log(x) if yy > 0.375 else self.alnrel(-y)
        g = super().bgrat(a, b, x, y, w, eps)
        CALLS[_cur[0]] = (a, b, x, y, w, lnx, g)
        return g


def evaluate(rows, cfg=FINAL):
    xlk.bx._KERNELS[cfg] = RecKernel(cfg)
    res = []
    for _ in range(4):
        res = []
        for i, r in enumerate(rows):
            _cur[0] = i
            res.append(xlk.bx.kernel(r["a"], r["b"], r["x"], r["y"], cfg))
        if not xlk.englib.MISSING and not xlk.bx._GL_MISSING:
            break
        xlk.englib.fetch_missing()
        if xlk.bx._GL_MISSING:
            xlk.bx.fetch_missing()
    return res


if __name__ == "__main__":
    rows = data.load()
    res = evaluate(rows)
    cnt = Counter()
    for i, (r, (w, w1, br)) in enumerate(zip(rows, res)):
        if i in CALLS:
            cnt[(br, CALLS[i][1] == 0.5)] += 1
    for k, v in sorted(cnt.items()):
        print(k, v)
    pickle.dump(dict(rows=rows, res=res, calls=CALLS), open(HERE / "calls.pkl", "wb"))
