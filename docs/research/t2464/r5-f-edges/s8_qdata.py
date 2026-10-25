"""Print the clean-region D(q) data per (shape, side), with the spread under Q = 0 (M-sd's structure) and the range
of q. Usage: s8_qdata.py"""
import qbits as QB

G = QB.groups()
for k, cs in sorted(G.items()):
    qs = sorted({float(c["q"]) for c in cs})
    s0 = QB.spread(cs, QB.mpf(0))
    pts = " ".join(f"q{float(c['q']):g}:{float(c['D']):.1f}" for c in sorted(cs, key=lambda c: (c["q"], c["x"])))
    print(f"({k[0]:g},{k[1]:g}) s{k[2]} n={len(cs)} spread(Q=0)={s0:7.1f} | {pts[:400]}")
