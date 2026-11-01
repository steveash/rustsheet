"""T-3569 r5-algdiv: exact, within-4-ulp and within-15-digit counts of the pinned lnB, per shape, for the best
storage variant of score_a.json's top tier (its first entry), TOMS algdiv in doubles and the correctly rounded lnB.
Usage: counts.py a|hold"""

import sys

import score as S

M, mp = S.M, S.M.mpmath


def cr_lnb(a, b):
    with mp.workdps(50):
        return float(mp.loggamma(a) + mp.loggamma(b) - mp.loggamma(mp.mpf(a) + b))


def parse(nm):
    return dict(kv.split("=", 1) for kv in nm.split())


def main(probe):
    shapes = S.pins(probe)
    best = parse(S.json.loads((S.HERE / "score_a.json").read_text())["ranking"][0][1])
    tomsd = dict(ar="d", aln="toms", lnb="xl", fin="toms", t2="sq", gsum="d")
    cands = {
        "best storage variant (" + S.name(best) + ")": lambda a, b: S.lnb_of(best, a, b),
        "TOMS algdiv in doubles": lambda a, b: S.lnb_of(tomsd, a, b),
        "correctly rounded lnB": cr_lnb,
    }
    print(f"{probe}: {len(shapes)} pinned shapes")
    for label, f in cands.items():
        ex = u4 = d15 = 0
        for a, b, L in shapes:
            y = f(a, b)
            ex += y == L
            u4 += abs(M.ulp_steps(L, y)) <= 4
            d15 += float(f"{y:.15g}") == float(f"{L:.15g}")
        print(f"  {label}: exact {ex}, within 4 ulp {u4}, same 15 digits {d15}")


if __name__ == "__main__":
    main(sys.argv[1])
