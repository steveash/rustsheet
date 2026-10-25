"""Pair form of the power term at small r: C + H + Q_h - Q_o in extended, one final rounding, where
Q_h = fl(h * V1) and Q_o = fl(o * V2) (o = b side 1, a side 2; side 2 enters with the opposite sign), V in
{LN x, LN r, LN mx, -LN R}.  Usage: s20_pairgrid.py SET tag"""
import sys

import r5lib as CL
import r5lib as R

VS = {"Lx": "", "Lr": "", "Lmx": "", "LR": "-"}  # LN R enters with a minus (ln R = -ln r)

src, tag = sys.argv[1], sys.argv[2]
pred = R.small_r if tag == "all" else (lambda m, n, rs: tag in rs[0]["blk"])
tot: dict = {}
for blk, rs, m, n in R.runs(src, pred):
    side1 = R.base(rs[0]["x"], m, n)[0]
    out = {"base": CL.scan(rs, m, n, "C+H!+P!")[0]}
    for v1 in VS:
        for v2 in VS:
            # side 1: + h V1 - b V2 ; side 2: - h V1 + a V2  (q = -fl(.)); LR flips the sign of its product
            s1 = 1 if VS[v1] == "" else -1
            s2 = 1 if VS[v2] == "" else -1
            if side1:
                t1 = ("-" if s1 > 0 else "+") + f"qh_{v1}"
                t2 = ("+" if s2 > 0 else "-") + f"qb_{v2}"
            else:
                t1 = ("+" if s1 > 0 else "-") + f"qh_{v1}"
                t2 = ("-" if s2 > 0 else "+") + f"qa_{v2}"
            ch = f"C+H{t1}{t2}!"
            out[f"{v1}/{v2}"] = CL.scan(rs, m, n, ch)[0]
    for k, v in out.items():
        tot[k] = tot.get(k, 0) + v
    print(f"{blk:28s}", " ".join(f"{k}={v}" for k, v in out.items()))
    sys.stdout.flush()
print("TOTAL", sorted(tot.items(), key=lambda kv: -kv[1]))
