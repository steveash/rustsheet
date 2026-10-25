"""Composition after lc: lc = fl(Cs + X) (Cs = Loader's stirlerr table combination, X = loader.xpart, extended sum,
lc stored), then
  'E'  : density = EXP(fl(K + lc))            K free (scanned)
  'Ee' : density = EXP(fl64(K + lc))
  'V'  : density = fl(EXP(lc) * K)            K free
  'Vd' : density = fl(EXP(lc) / K)
Usage: s3_lc.py BLOCK 'knobs' [lcmode=x|e] ..."""
import sys
import mpmath
import dlib as L
import loader as LD
import s2_scan as S

sys.path.append(str(L.T / "w6-literature"))
import h2_loader as H  # noqa: E402  (stirlerr table SF, Arith.stirlerr)

rows = S.rows
A = H.Arith("D")


def run(blk, spec, comb, lcmode="x", limit=800):
    idx = L.blocks(rows)[blk][:limit]
    kw = S.parse(spec)
    kw.pop("comb", None)
    r0 = rows[idx[0]]
    a, b = r0["a"], r0["b"]
    k, n = a - 1, a + b - 2
    Cs = A.stirlerr(n) - A.stirlerr(k) - A.stirlerr(n - k)
    lc = {}
    for i in idx:
        X = LD.xpart(rows[i]["x"], a, b, **kw)[0]
        lc[i] = L.rnd(Cs + X, lcmode)
    i0 = idx[0]
    if comb.startswith("E"):
        K0 = mpmath.log(rows[i0]["exact"]) - lc[i0]
    elif comb == "V":
        K0 = rows[i0]["exact"] / L.exp_v6(lc[i0])
    else:
        K0 = L.exp_v6(lc[i0]) / rows[i0]["exact"]

    def f(K, i):
        if comb == "E":
            return L.exp_v6(mpmath.mpf(L.D(K + lc[i])))
        if comb == "Ee":
            return L.exp_v6(L.R64(K + lc[i]))
        e = mpmath.mpf(L.exp_v6(lc[i]))
        return L.D(e * K) if comb == "V" else L.D(e / K)

    ev = [i for i in idx if not rows[i]["hold"]]
    od = [i for i in idx if rows[i]["hold"]]
    cnt = lambda K, sel: sum(f(K, i) == rows[i]["excel"] for i in sel)  # noqa: E731
    best = (-1, K0)
    scale = abs(K0) if comb.startswith("V") else 1
    for j in range(-64, 65):
        K = K0 + j * mpmath.mpf(2) ** -55 * scale
        c = cnt(K, ev)
        if c > best[0]:
            best = (c, K)
    for j in range(-32, 33):
        K = best[1] + j * mpmath.mpf(2) ** -60 * scale
        c = cnt(K, ev)
        if c > best[0]:
            best = (c, K)
    print(f"{blk:10s} ({a:g},{b:g}) [{spec}] {comb} lc={lcmode} K={mpmath.nstr(best[1], 20)} even {best[0]}/{len(ev)}"
          f" hold {cnt(best[1], od)}/{len(od)}", flush=True)


if __name__ == "__main__":
    blk, spec = sys.argv[1], sys.argv[2]
    for comb in sys.argv[3:] or ["E", "Ee", "V", "Vd"]:
        c, *lm = comb.split(":")
        run(blk, spec, c, lm[0] if lm else "x")
