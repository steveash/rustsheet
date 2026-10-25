"""Free-constant scan over blocks and bd0 knob sets.
Usage: scan.py [--src old|<probe>] [--chain R|C] BLOCKREGEX 'k=v,k=v' ['k=v' ...]
Knobs (r3lib.series/bd0/xpart): v s ej t v2 (x|e), coef (div|mul), nt (int), cmp (own|x), d sm (x|e), cut,
q (fl|ex), nq (mul|sub), np (x|e).  Prints even (fit) and hold-out (odd half, md5) exact counts per block."""

from __future__ import annotations

import re
import sys

import r3lib as L


def parse(spec: str) -> dict:
    o: dict = {}
    for part in filter(None, spec.split(",")):
        k, v = part.split("=")
        o[k] = float(v) if k == "cut" else (int(v) if k == "nt" else v)
    return o


def load(src: str):
    return L.load_old() if src == "old" else L.load_probe(src)


def main(argv):
    src, chain = "old", "C"
    while argv and argv[0].startswith("--"):
        if argv[0] == "--src":
            src = argv[1]
        elif argv[0] == "--chain":
            chain = argv[1]
        argv = argv[2:]
    rows = load(src)
    pat = re.compile(argv[0])
    specs = argv[1:] or [""]
    tot: dict = {}
    for blk, idx in sorted(L.blocks(rows).items()):
        if not pat.fullmatch(blk) or len(idx) < 32:
            continue
        r0 = rows[idx[0]]
        for spec in specs:
            res = L.fit_block(rows, idx, parse(spec), chain)
            t = tot.setdefault(spec, [0, 0, 0, 0])
            t[0] += res["even"]
            t[1] += res["nev"]
            t[2] += res["hold"]
            t[3] += res["nod"]
            print(
                f"{blk:16s} ({r0['a']:g},{r0['b']:g}) [{spec}] {chain} off {res['off']:+7.3f}u  even "
                f"{res['even']}/{res['nev']}  hold {res['hold']}/{res['nod']}  br {res['br']}",
                flush=True,
            )
        L.save_cache()
    for spec, t in tot.items():
        print(f"TOTAL [{spec}] even {t[0]}/{t[1]} hold {t[2]}/{t[3]}")


if __name__ == "__main__":
    main(sys.argv[1:])
