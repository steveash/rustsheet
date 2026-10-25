"""Out-of-sample test of the claim that the per-df constant offset is e((n+1)/2) - e(n/2) + c0 (r3-loader-const's
stirlerr table): for every df of the clean hold-outs (hold2, verifier vhold) scan the LT1 offset (step 0.05 units),
report the plateau of maximal bit-exact count, and the distance of the e-table prediction and of 0 (exact C) to it.
usage: vconst.py"""

from __future__ import annotations

import collections

import numpy as np

import vlib as V
import vrun as R

TD = R.TD


def main():
    rows, _ = V.load_all()
    S = R.sets(rows)
    _, _, etab = R.offsets()
    pred = etab()
    grid = np.round(np.arange(-4.0, 4.0001, 0.05), 2)
    dp, d0 = [], []
    for nm in ("hold2", "vhold"):
        per = collections.defaultdict(list)
        for r in S[nm]:
            per[r["n"]].append(r)
        for n in sorted(per):
            q = per[n]
            sc = np.array([V.score((TD.lt1(r["t"], n, float(g)), r["excel"]) for r in q)[1] for g in grid])
            mx = sc.max()
            top = grid[sc == mx]
            lo, hi = float(top.min()), float(top.max())

            def dist(x):
                return 0.0 if lo <= x <= hi else min(abs(x - lo), abs(x - hi))

            p = pred.get(n, 0.0)
            dp.append(dist(p))
            d0.append(dist(0.0))
            at = lambda x: int(sc[np.argmin(abs(grid - x))])  # noqa: E731
            print(
                f"{nm} df {n:g}: max {mx}/{len(q)} on [{lo:+.2f}, {hi:+.2f}] (|top| {len(top)}); "
                f"e-table {p:+.2f} -> {at(p)} (dist {dist(p):.2f}); exact C -> {at(0)} (dist {dist(0):.2f})"
            )
    print(
        f"distance to plateau, rms over {len(dp)} dfs: e-table {np.sqrt(np.mean(np.square(dp))):.2f}, "
        f"exact C {np.sqrt(np.mean(np.square(d0))):.2f}; e-table closer on {sum(a < b for a, b in zip(dp, d0))}, "
        f"exact closer on {sum(a > b for a, b in zip(dp, d0))}"
    )


if __name__ == "__main__":
    main()
