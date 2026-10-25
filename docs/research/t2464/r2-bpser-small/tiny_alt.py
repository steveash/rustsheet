"""apser / fpser cells (probe tiny): is Excel's value the plain bpser series I = EXP(a LN x - lnB) * s on the
unswapped (a, b, x), with some lnB form, instead of TOMS fpser / apser? Variants of lnB (w3_lg / w3_order) and z."""
from collections import Counter
from dataclasses import replace

import lib

rows = lib.load_probe(("tiny",))
pts = [(r["a"], r["b"], r["x"]) for r in rows]
for gl in ("toms", "gl", "acc"):
    for order in ("arg", "min"):
        for z in ("z64", "z53"):
            m = lib.Model(replace(lib.bx.BEST, w3_lg=gl, w3_order=order, w3_z=z), lib.V(z=z))
            for _ in range(3):
                vals = [m.bpser_w3(a, b, x) for a, b, x in pts]
                if not lib.bx._GL_MISSING:
                    break
                lib.bx.fetch_missing()
            c = Counter()
            for r, v in zip(rows, vals):
                t = lib.tally([(v, r["excel"])])
                c[(r["blk"], 0)] += t[1]
                c[(r["blk"], 1)] += t[2]
                c[(r["blk"], 2)] += t[3]
                c[(r["blk"], 3)] += 1
            print(gl, order, z, {k: f"{c[(k, 0)]}/{c[(k, 1)]}/{c[(k, 2)]} of {c[(k, 3)]}" for k in ("fpser", "apser")})
