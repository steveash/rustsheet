"""Free-constant structure scan, general knobs.  density = EXP_V6(comb(C + X)), X = loader.xpart(**knobs).
Usage: s2_scan.py BLOCK 'k=v,k=v' ...   knobs: ar lnmode ymode cut addar sar xar (loader) and comb (x|e|m).
C is fitted on the even half (coarse 2^-55 then fine 2^-60 steps) and scored on the hold-out half."""
import sys
import mpmath
import dlib as L
import loader as LD

rows = L.load()


def parse(spec):
    kw = {}
    for part in filter(None, spec.split(",")):
        k, v = part.split("=")
        kw[k] = float(v) if k == "cut" else v
    return kw


def run(blk, spec, limit=1600, quiet=False):
    idx = L.blocks(rows)[blk][:limit]
    kw = parse(spec)
    comb = kw.pop("comb", "x")
    r0 = rows[idx[0]]
    X0 = LD.xpart(r0["x"], r0["a"], r0["b"], ar="m", lnmode="m", ymode="ex")[0]
    C0 = mpmath.log(r0["exact"]) - X0
    X = {i: LD.xpart(rows[i]["x"], rows[i]["a"], rows[i]["b"], **kw)[0] for i in idx}
    ev = [i for i in idx if not rows[i]["hold"]]
    od = [i for i in idx if rows[i]["hold"]]

    def cnt(C, sel):
        n = 0
        for i in sel:
            s = C + X[i]
            E = L.R64(s) if comb == "e" else (s if comb == "m" else mpmath.mpf(L.D(s)))
            n += L.exp_v6(E) == rows[i]["excel"]
        return n

    best = (-1, C0)
    for j in range(-64, 65):
        C = C0 + j * mpmath.mpf(2) ** -55
        n = cnt(C, ev)
        if n > best[0]:
            best = (n, C)
    for j in range(-32, 33):
        C = best[1] + j * mpmath.mpf(2) ** -60
        n = cnt(C, ev)
        if n > best[0]:
            best = (n, C)
    off = float((best[1] - C0) * mpmath.mpf(2) ** 53)
    ho = cnt(best[1], od)
    if not quiet:
        print(f"{blk:10s} ({r0['a']:g},{r0['b']:g}) [{spec}] comb={comb} C-C0={off:+7.3f}u  even {best[0]}/{len(ev)}"
              f"  hold {ho}/{len(od)}", flush=True)
    return best, ho, len(od)


if __name__ == "__main__":
    for spec in sys.argv[2:]:
        run(sys.argv[1], spec)
