"""Two-constant chains (r5lib.scan2): the Loader split K' = A + B, with D the piece that meets H first.
Usage: s12_two.py SET tag  (prints per run: base C+H!+P!, then D=B and D=A variants)"""
import sys

import r5lib as CL
import r5lib as R

src, tag = sys.argv[1], sys.argv[2]
CH = ["D+H!+C!+P!", "D+H!+C+P!", "C+D!+H!+P!"]
pred = R.small_r if tag == "all" else (lambda m, n, rs: tag in rs[0]["blk"])
tot = {}
for blk, rs, m, n in R.runs(src, pred):
    side1 = R.base(rs[0]["x"], m, n)[0]
    A, B = CL.loader_split(m, n, side1)
    out = {"base": CL.scan(rs, m, n, "C+H!+P!")[0]}
    for ch in CH:
        for nm, D0 in (("B", B), ("A", A)):
            out[f"{ch}|D={nm}"] = CL.scan2(rs, m, n, ch, D0, spanD=40.0, stepD=0.5)[0]
    for k, v in out.items():
        tot[k] = tot.get(k, 0) + v
    print(f"{blk:28s} A={float(A):8.3f} B={float(B):7.3f}", " ".join(f"{k}={v}" for k, v in out.items()))
    sys.stdout.flush()
print(tot)
