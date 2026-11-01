"""T-3567 r5-fdf1: the frozen model of Excel's F cdf and upper tail (frozen before the hold-out was filled).

F's incomplete-beta arguments: with mx = fl(m x), r = fl(n/mx) if mx <= n else fl(mx/n), so r >= 1, and
    (x, y) = (fl(1/fl(1+r)), fl(r/fl(1+r)))   if mx <= n
             (fl(r/fl(1+r)), fl(1/fl(1+r)))   otherwise
The kernel is r2-tdist FINAL with bgrat at b = 1/2 in its implementable x87 form (bg = "ld"), a = m/2, b = n/2;
F.DIST(., TRUE) is w, F.DIST.RT / FDIST is w1. m and n are the truncated dfs.
"""
import sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r2-tdist"))
import xlk  # noqa: E402

CFG = xlk.XCfg(alnrel="xl", erfc="xl", bg_lnx="cut0.01", bg="ld")


def xy(x: float, m: float, n: float) -> tuple[float, float]:
    mx = m * x
    if mx <= n:
        r = n / mx
        return 1.0 / (1.0 + r), r / (1.0 + r)
    r = mx / n
    return r / (1.0 + r), 1.0 / (1.0 + r)


def xy_s(x: float, m: float, n: float) -> tuple[float, float]:
    """The control: x = fl(mx/fl(n+mx)), y = fl(n/fl(n+mx))."""
    mx = m * x
    s = n + mx
    return mx / s, n / s


def run(points, form=xy):
    """[(x, m, n)] -> [(lower, upper, branch)]."""
    pts = [(m / 2, n / 2, *form(x, m, n)) for x, m, n in points]
    return xlk.run(pts, CFG)
