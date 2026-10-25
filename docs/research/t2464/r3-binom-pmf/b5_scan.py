"""Knob x chain scan on selected blocks: total all-cell best-overlap counts (free C per block).
Usage: b5_scan.py <probe> <fn> <block substring> <chains comma> <knob dicts separated by ;>"""
import sys

import blib as B

name, fn, sub, chains, knobs = sys.argv[1:6]
rows = B.load_probe(name)
bl = {k: v for k, v in B.blocks(rows, fn).items() if sub in k[1]}
for ks in knobs.split(";"):
    o = eval(ks)
    for ch in chains.split(","):
        tot = n = 0
        per = []
        for key, idx in sorted(bl.items()):
            f = B.fit_block_g(rows, idx, o, ch) if ":" in ch else B.fit_block(rows, idx, o, ch)
            tot += f["all"]
            n += f["n"]
            per.append(f["all"])
        print(f"{ks:40s} {ch:7s} {tot}/{n}  {per}")
