"""verify-r5-f-edges: scores on runs of consecutive doubles (identified from x alone: >= 8 rows whose x are
successive doubles at one shape), by the run's trunc(d1)/trunc(d2) class."""
import math
import sys

import vparse
import vscore as V


def runs(rs):
    out, cur = [], [rs[0]]
    for p, r in zip(rs, rs[1:]):
        if (r["d1"], r["d2"]) == (p["d1"], p["d2"]) and r["x"] > 0 and r["x"] == math.nextafter(p["x"], math.inf):
            cur.append(r)
        else:
            out.append(cur)
            cur = [r]
    out.append(cur)
    return [c for c in out if len(c) >= 8]


def main(case):
    rs = vparse.rows(case)
    ME = V.model_edge()
    groups = {}
    for run in runs(rs):
        m, n = math.trunc(run[0]["d1"]), math.trunc(run[0]["d2"])
        key = "d1=1" if m == 1 else "d1=2" if m == 2 else "d2=1" if n == 1 else "d2=2" if n == 2 else "other"
        groups.setdefault(key, []).extend(run)
    for k, v in sorted(groups.items()):
        me = V.tally([(ME(r["x"], r["d1"], r["d2"]), r["excel"]) for r in v])
        cr = V.tally([(V.cr(r["x"], r["d1"], r["d2"]), r["excel"]) for r in v])
        shapes = sorted({(r["d1"], r["d2"]) for r in v})
        print(f"{case} vrun {k}: M-edge {V.fmt(me)}  CR {V.fmt(cr)}  shapes {shapes}")


if __name__ == "__main__":
    for c in sys.argv[1:]:
        main(c)
