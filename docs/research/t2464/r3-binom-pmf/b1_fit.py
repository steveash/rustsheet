"""Structure test on in-sample probes: one free constant per block, x part from r3-loader-bd0's model (or knobs given
as argv[2]), chains C (E = fl(C + X)) and Ce (extended).  Prints even-fit / odd-hold counts per block.
Usage: b1_fit.py <probe> [knob-dict] [chains] [block substring] [fn]"""
import sys

import blib as B

name = sys.argv[1] if len(sys.argv) > 1 else "a"
o = eval(sys.argv[2]) if len(sys.argv) > 2 else dict(cut=0.2, v="x", ph=1, ld="x")
chains = sys.argv[3].split(",") if len(sys.argv) > 3 else ["C", "Ce"]
rows = B.load_probe(name)
print("echo misreads:", B.echo_check(name)[:5])
tot = {c: [0, 0] for c in chains}
SUB = sys.argv[4] if len(sys.argv) > 4 else ""
FN = sys.argv[5] if len(sys.argv) > 5 else None
for key, idx in sorted(B.blocks(rows, FN).items()):
    if SUB not in key[1]:
        continue
    out = []
    for ch in chains:
        f = (B.fit_block_fx(rows, idx, o, ch[3:]) if ch.startswith("FX") else B.fit_block_g(rows, idx, o, ch) if ":" in ch
             else B.fit_block(rows, idx, o, ch))
        out.append(f"{ch}: {f['hold']}/{f['nod']} (all {f['all']}/{f['n']}, off {f['off_all']:+.2f})")
        tot[ch][0] += f["hold"]
        tot[ch][1] += f["nod"]
    print(f"{key[0]:4s} {key[1]:28s} br={f['br']}  " + "  ".join(out))
print("totals (odd halves):", tot)
R = B.R3
R.save_cache = lambda: None
