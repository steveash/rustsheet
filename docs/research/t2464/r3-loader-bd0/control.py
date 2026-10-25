"""Positive control for the free-constant interval fit: synthetic 'Excel' values from a known model (r3 knobs, chain
Ce or K, constant C0 + 3.3 units) at the probe's own x, then the same fit_block run.  Expect full hits at +3.3."""
import r3lib as L
import scan as S

rows = S.load("a")
B = L.blocks(rows)
o = {"v": "x", "ph": 1, "ld": "x"}
for chain in ("Ce", "K"):
    for blk in ("a(20.5,45.5)v+0.10", "a(40.25,89.75)v-0.30", "a(5.3,9.7)v+0.19"):
        idx = B[blk]
        f0 = L.fit_block(rows, idx, o, chain, W=1024.0)
        L5, h = L.shape_pieces(rows[idx[0]]["a"], rows[idx[0]]["b"])
        syn = []
        for i in idx:
            r = dict(rows[i])
            b1, b2, _ = f0["X"][i]
            r["excel"] = L.chain_out(chain, L.LDB(f0["C0"]) + L.LDB(3.3) * L.U53, b1, b2, L5, h, None)
            syn.append(r)
        f = L.fit_block(syn, list(range(len(syn))), o, chain, W=1024.0)
        # a wrong model on the same synthetic data: r2 knobs
        g = L.fit_block(syn, list(range(len(syn))), {}, chain, W=1024.0)
        print(f"{chain} {blk:22s} recovered off {f['off'] - 0:+.3f} (planted +3.3 rel. C0)  hold {f['hold']}/{f['nod']};"
              f"  r2 knobs on it: hold {g['hold']}/{g['nod']}")
L.save_cache()
