"""The power term as two stored LN(x) products whose coefficients add up to the side's c (a | -b), in several chain
positions.  Usage: s19_pairs.py SET tag"""
import sys

import r5lib as CL
import r5lib as R

# (Q1, Q2) token pairs per side, in r5lib's chain grammar ('-qX_Lx' = +fl(X LN x), 'qX_Lx' = -fl(X LN x))
PAIRS = {
    1: [("-qh_Lx", "qb_Lx"), ("qb_Lx", "-qh_Lx"), ("-qam1_Lx", "-q1_Lx"), ("-q1_Lx", "-qam1_Lx"), ("-qap1_Lx", "q1_Lx"),
        ("-qah_Lx", "-qah_Lx"), ("-qhm1_Lx", "qbm1_Lx"), ("qbm1_Lx", "-qhm1_Lx")],
    2: [("-qa_Lx", "qh_Lx"), ("qh_Lx", "-qa_Lx"), ("qbp1_Lx", "-q1_Lx"), ("-q1_Lx", "qbp1_Lx"), ("qbm1_Lx", "q1_Lx"),
        ("qbh_Lx", "qbh_Lx"), ("-qam1_Lx", "qhm1_Lx"), ("qhm1_Lx", "-qam1_Lx")],
}
SHAPES = ["C+H+{q1}!{q2s}!", "C+H!{q1s}!{q2s}!", "C{q1s}+H!{q2s}!", "C{q1s}!+H!{q2s}!", "C+H{q1s}{q2s}!",
          "C+H{q1s}!{q2s}!"]


def sgn(t):
    return t if t.startswith("-") else "+" + t


src, tag = sys.argv[1], sys.argv[2]
for blk, rs, m, n in R.runs(src, lambda m, n, rs: tag in rs[0]["blk"]):
    side = 1 if R.base(rs[0]["x"], m, n)[0] else 2
    res = [(CL.scan(rs, m, n, "C+H!+P!")[0], "C+H!+P!")]
    for q1, q2 in PAIRS[side]:
        for sh in SHAPES:
            ch = sh.format(q1=q1.lstrip("+"), q1s=sgn(q1), q2s=sgn(q2))
            if ch.startswith("C+H+-"):
                ch = ch.replace("C+H+-", "C+H-")
            try:
                res.append((CL.scan(rs, m, n, ch)[0], ch))
            except ValueError as e:
                res.append((-1, ch + " ERR " + str(e)))
    res.sort(reverse=True)
    print(blk, len(rs), res[:8])
    sys.stdout.flush()
