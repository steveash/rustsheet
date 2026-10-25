"""R dbeta / dbinom_raw chain with stored returns:
   lc = fl(Cs + X)            Cs = stirlerr(n) - stirlerr(k) - stirlerr(n-k) (Loader table / formula, w6's Arith 'D')
   g  = fl(lc - h)            h = 0.5*lf (a double; scanned +-NH ulps around the ideal 0.5*lf)
   E  = fl(L5 + g)            L5 = Excel LN(fl(a+b-1))
   density = EXP_V6(E)
h is fitted on the even half, scored on the hold-out half.  Usage: s4_chain.py BLOCK knobs [lcmode] [gmode]"""
import math
import sys
import mpmath
import dlib as L
import loader as LD
import s2_scan as S

sys.path.append(str(L.T / "w6-literature"))
import h2_loader as H  # noqa: E402

rows = S.rows
A = H.Arith("D")


def lf_ideal(k, n):
    return mpmath.log(2 * mpmath.pi) + mpmath.log(k) + mpmath.log(n - k) - mpmath.log(n)


def prep(blk, spec, lcmode="x", limit=None):
    idx = L.blocks(rows)[blk][:limit]
    kw = S.parse(spec)
    kw.pop("comb", None)
    r0 = rows[idx[0]]
    a, b = r0["a"], r0["b"]
    k, n = a - 1, a + b - 2
    Cs = A.stirlerr(n) - A.stirlerr(k) - A.stirlerr(n - k)
    lc = {i: L.rnd(Cs + LD.xpart(rows[i]["x"], rows[i]["a"], rows[i]["b"], **kw)[0], lcmode) for i in idx}
    return idx, a, b, k, n, lc


def chain(lc, h, L5, gmode="x", emode="x"):
    g = L.rnd(lc - h, gmode)
    return L.exp_v6(L.rnd(L5 + g, emode))


def run(blk, spec, lcmode="x", gmode="x", emode="x", NH=48, quiet=False, idx_lc=None):
    idx, a, b, k, n, lc = idx_lc or prep(blk, spec, lcmode)
    L5 = mpmath.mpf(L.LN(a + b - 1))
    h0 = float(lf_ideal(k, n) / 2)
    u = math.ulp(h0)
    ev = [i for i in idx if not rows[i]["hold"]]
    od = [i for i in idx if rows[i]["hold"]]
    cnt = lambda h, sel: sum(chain(lc[i], h, L5, gmode, emode) == rows[i]["excel"] for i in sel)  # noqa: E731
    best = max(((cnt(mpmath.mpf(h0 + j * u), ev), j) for j in range(-NH, NH + 1)))
    hb = mpmath.mpf(h0 + best[1] * u)
    ho = cnt(hb, od)
    if not quiet:
        print(f"{blk:10s} ({a:g},{b:g}) [{spec}] lc={lcmode} g={gmode} E={emode} h=h0{best[1]:+d}ulp even {best[0]}/{len(ev)}"
              f" hold {ho}/{len(od)}", flush=True)
    return best, ho, len(od), hb


if __name__ == "__main__":
    blk, spec = sys.argv[1], sys.argv[2]
    modes = sys.argv[3:] or ["x:x:x", "e:x:x", "x:e:x", "x:x:e"]
    for m in modes:
        lcm, gm, em = m.split(":")
        run(blk, spec, lcm, gm, em)
