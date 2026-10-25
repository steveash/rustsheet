"""Score full models (model.py) per block on the a, b > 1 density data.  Usage: m1_score.py 'opt=v,...' [limit]
opts: stir=D|E|S|exact lf=LF0|LF1|exact lc=x|e g=x|e chain=R|K|S."""
import sys
import statistics as st
import dlib as L
import model as M

rows = L.load()
spec = sys.argv[1] if len(sys.argv) > 1 else ""
limit = int(sys.argv[2]) if len(sys.argv) > 2 else 200
opts = dict(p.split("=") for p in filter(None, spec.split(",")))
tot = []
print(f"== {spec}")
for blk, idx in sorted(L.blocks(rows).items()):
    sel = idx[:limit]
    pr = M.predict(rows, opts, sel)
    errs = [L.err_units(pr[i], rows[i]["exact"]) - L.err_units(rows[i]["excel"], rows[i]["exact"]) for i in sel]
    t = L.tally((pr[i], rows[i]["excel"]) for i in sel)
    tot += [(pr[i], rows[i]["excel"]) for i in sel]
    r0 = rows[sel[0]]
    print(f"  {blk:12s} ({r0['a']:g},{r0['b']:g}) {L.fmt(t)}  CR {L.fmt(L.tally((float(rows[i]['exact']), rows[i]['excel']) for i in sel))}"
          f"  model-excel mean {st.mean(errs):+6.2f} sd {st.pstdev(errs):5.2f}", flush=True)
print("  ALL", L.fmt(L.tally(tot)))
