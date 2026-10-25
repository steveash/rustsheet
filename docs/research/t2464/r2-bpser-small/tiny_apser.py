"""apser cells of probe tiny: Excel against CR, the plain bpser (GL lnB, z64) and TOMS apser (BEST), relative errors, by x."""
from dataclasses import replace

import lib

rows = [r for r in lib.load_probe(("tiny",)) if r["blk"] == "apser"]
m = lib.Model(replace(lib.bx.BEST, w3_lg="gl"), lib.V())
for r in sorted(rows, key=lambda r: (r["a"], r["x"])):
    a, b, x, e = r["a"], r["b"], r["x"], r["excel"]
    v = m.bpser_w3(a, b, x)
    t = lib.kernel(a, b, x, 1.0 - x)[0]
    print(f"a={a:<7g} b={b:<9.3g} x={x:<7g} y*a={(1 - x) * a:7.4f} rel(cr-e)={(r['cr'] - e) / e:+.2e} "
          f"rel(bpser-e)={(v - e) / e:+.2e} rel(apser-e)={(t - e) / e:+.2e}")
