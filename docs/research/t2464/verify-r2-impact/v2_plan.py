"""Verifier step 2: re-score r2-impact's PLAN (BEST + Excel log1p, LDe density for min(a,b) > 1, engine kept for
T.DIST* in the bgrat family) with my own cell reader, my own cell -> kernel mapping, my own engine-value lookup and
my own tallies. Only the candidate model functions are imported (bratio_excel.kernel / Kernel.density,
excel_log1p.log1p_series, r2-density-large model.density). All caches redirected into this directory.

Run: /home/user/rustsheet/bindings/python/.venv/bin/python -I v2_plan.py   (writes v2_cells.pkl here)
"""
import csv, hashlib, json, math, pickle, sys
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
T = HERE.parent
sys.path.insert(0, str(HERE))
from v1_offcells import REPORT, w15  # noqa: E402

UNGATED = {"probes-numeric", "gen-m8a", "gen-m8b"}
TFN = {"T.DIST", "T.DIST.RT", "T.DIST.2T", "TDIST"}
BGRAT = {"bgrat", "bup+bgrat", "L140 bup+bgrat", "L140 bup+bup+bgrat"}


def arg(s):
    if s in ("True", "False"):
        return s == "True"
    return float(s)


def tr(v):
    return float(math.trunc(v))


def mapping(fn, a):
    """-> ('cdf', a, b, x, y, tail, post) | ('pdf', x, a, b, width) | None. Written from w5's verified forms."""
    n = len(a)
    if fn in ("BETA.DIST", "BETADIST"):
        if fn == "BETA.DIST":
            if n < 4:
                return None
            x, p, q, cum = a[0], a[1], a[2], a[3]
            lo = a[4] if n > 4 else 0.0
            hi = a[5] if n > 5 else 1.0
        else:
            x, p, q, cum = a[0], a[1], a[2], True
            lo = a[3] if n > 3 else 0.0
            hi = a[4] if n > 4 else 1.0
        if isinstance(cum, float):
            cum = cum != 0.0
        if not (p > 0 and q > 0 and lo < hi and lo <= x <= hi):
            return None
        t = x if (lo == 0.0 and hi == 1.0) else (x - lo) / (hi - lo)
        if not (0.0 < t < 1.0):  # also rejects nan from (x - lo)/(hi - lo) overflow
            return None
        if cum:
            return ("cdf", p, q, t, 1.0 - t, "w", None)
        return ("pdf", t, p, q, hi - lo)
    if fn in ("F.DIST", "F.DIST.RT", "FDIST"):
        if fn == "F.DIST" and not (n > 3 and a[3] is True):
            return None
        X, d1, d2 = a[0], tr(a[1]), tr(a[2])
        if X <= 0 or not (1 <= d1 < 1e10 and 1 <= d2 < 1e10):
            return None
        u = d1 * X
        return ("cdf", d1 / 2, d2 / 2, 1 / (1 + d2 / u), 1 / (1 + u / d2), "w" if fn == "F.DIST" else "w1", None)
    if fn in TFN:
        t, df = a[0], tr(a[1])
        if fn == "T.DIST" and not (n > 2 and a[2] is True):
            return None
        if t == 0 or not (1 <= df < 1e10):
            return None
        if fn == "T.DIST":
            op = "L" if t < 0 else "1-L"
        elif fn == "T.DIST.RT":
            op = "L" if t > 0 else "1-L"
        elif fn == "T.DIST.2T":
            if t < 0:
                return None
            op = "2L"
        else:
            tails = tr(a[2]) if n > 2 else 0
            if t < 0 or tails not in (1.0, 2.0):
                return None
            op = "L" if tails == 1 else "2L"
        u = abs(t)
        s = df + u * u
        y = df / s
        z = (u * u) / s
        if df == 2 or (u > 1 and y <= 0.99):
            return ("cdf", df / 2, 0.5, y, 1.0 - y, "w", ("tail", op))
        return ("cdf", 0.5, df / 2, z, y, "w", ("centre", op))
    if fn in ("BINOM.DIST", "BINOMDIST"):
        if not (n > 3 and a[3] is True):
            return None
        k, N, p = math.trunc(a[0]), math.trunc(a[1]), a[2]
        if not (0 < p < 1) or k < 1 or k >= N:  # k = 0 is Excel's own EXP(n l) shortcut, not the kernel
            return None
        return ("cdf", float(k + 1), float(N - k), p, 1.0 - p, "w1", None)
    if fn == "NEGBINOM.DIST":
        if not (n > 3 and a[3] is True):
            return None
        f, r, p = math.trunc(a[0]), math.trunc(a[1]), a[2]
        if not (0 < p < 1) or f < 0 or r < 1:
            return None
        return ("cdf", float(r), float(f + 1), p, 1.0 - p, "w", None)
    return None


def post(pp, v):
    if pp is None:
        return v
    kind, op = pp
    L = v / 2 if kind == "tail" else 0.5 - v / 2
    return {"L": L, "1-L": 1.0 - L, "2L": 2 * L}[op]


def load_engine():
    rep = json.load(open(REPORT))
    status, fails = {}, {}
    for c in rep["cases"]:
        status[c["id"]] = (c["status"], c["checked"], c["passed"], len(c["cells"]))
        for cell in c["cells"]:
            fails[(c["id"], f"{cell['sheet']}!{cell['cell']}")] = cell
    return status, fails


def build():
    status, fails = load_engine()
    rows, seen, dup = [], set(), 0
    nostat = Counter()
    for r in csv.DictReader(open(T / "r2-impact" / "cells_all.tsv"), delimiter="\t"):
        if "t2464-r2-" in r["case"]:
            continue
        try:
            ex = float(r["excel"])
        except ValueError:
            continue
        k = (r["case"], r["cell"])
        if k in seen:
            dup += 1
            continue
        seen.add(k)
        m = mapping(r["fn"], tuple(arg(s) for s in r["args"].split("|")))
        if m is None:
            continue
        st = status.get(r["case"])
        if st is None or st[0] not in ("pass", "fail"):
            nostat[(st or ("missing",))[0]] += 1
            continue
        f = fails.get(k)
        if f is None:
            eng = ex
        else:
            av = f["actual"]
            eng = float(av["v"]) if isinstance(av, dict) and av.get("t") == "n" else math.nan
            if f["verdict"] not in ("pass", "deviation"):
                pass
        rows.append(dict(k=k, fn=r["fn"], args=r["args"], m=m, excel=ex, eng=eng,
                         pop="w2f" if "/t2464-w2f-" in r["case"] else "pre",
                         gated=r["case"].split("/")[0] not in UNGATED))
    print("dup (case,cell) rows skipped:", dup, " cells in non-pass/fail cases:", dict(nostat), file=sys.stderr)
    return rows


def run_models(rows):
    sys.path.insert(0, str(T / "r2-bratio"))
    sys.path.insert(0, str(T / "r2-log1p"))
    import bratio_excel as bx
    import excel_log1p as el
    gl = HERE / "gammaln_cache.pkl"
    for src in (T / "r2-impact" / "gammaln_cache.pkl", gl):
        if src.exists():
            for v, r in pickle.loads(src.read_bytes()).items():
                bx._GL.setdefault(v, r)
    bx._GLF = gl
    bx.log1p_cr = el.log1p_series  # Excel log1p in the density (= r2-impact's "BEST+log1p")
    sys.path.append(str(T / "r2-density-large"))
    import dlib
    dlib._GC = HERE / "gl_cache_dl.pkl"
    import model as LDM
    kern = bx.Kernel(bx.BEST)
    for _ in range(5):
        for c in rows:
            m = c["m"]
            if m[0] == "cdf":
                w, w1, br = bx.kernel(m[1], m[2], m[3], m[4], bx.BEST)
                c["best"] = post(m[6], w if m[5] == "w" else w1)
                c["br"] = br.replace("/sw", "")
            else:
                x, p, q, width = m[1:]
                if min(p, q) > 1:
                    v = float(LDM.density(x, p, q, dict(stir="exact", lf="LF0")))
                    c["br"] = "pdf>1"
                else:
                    v, _ = kern.density(x, p, q)
                    c["br"] = "pdfM1"
                c["best"] = v / width if width != 1.0 else v
        if not bx._GL_MISSING:
            break
        print("fetching", len(bx._GL_MISSING), file=sys.stderr)
        bx.fetch_missing()
    for c in rows:
        c["plan"] = c["eng"] if (c["fn"] in TFN and c["br"] in BGRAT) else c["best"]
    return rows


def main():
    p = HERE / "v2_cells.pkl"
    if p.exists():
        rows = pickle.loads(p.read_bytes())
    else:
        rows = run_models(build())
        p.write_bytes(pickle.dumps(rows))
    report(rows)


def ulp4(m, e):
    return m == e or (math.isfinite(m) and math.isfinite(e) and e != 0 and abs(m - e) <= 4 * math.ulp(e))


def tally(rs, key="plan"):
    t = Counter(n=len(rs))
    for c in rs:
        e, g, m = c["excel"], c["eng"], c[key]
        t["eng_ex"] += g == e
        t["eng_vis"] += not w15(e, g)
        t["ex"] += m == e
        t["u4"] += ulp4(m, e)
        t["w15"] += w15(e, m)
        t["vis"] += not w15(e, m)
        t["fixed"] += (not w15(e, g)) and w15(e, m)
        t["broke"] += w15(e, g) and not w15(e, m)
        t["lost_ex"] += g == e and m != e
    return t


def md5odd(s):
    return int(hashlib.md5(s.encode()).hexdigest(), 16) % 2 == 1


def report(rows):
    pre = [c for c in rows if c["pop"] == "pre"]
    w2f = [c for c in rows if c["pop"] == "w2f"]
    K = ("n", "eng_ex", "eng_vis", "ex", "u4", "w15", "vis", "fixed", "broke", "lost_ex")
    show = lambda name, rs: print(f"{name:44s} " + " ".join(f"{k}={tally(rs)[k]}" for k in K))
    show("pre all", pre)
    show("pre md5(case|cell) odd [worker's hold-out]", [c for c in pre if md5odd(f"{c['k'][0]}|{c['k'][1]}")])
    show("pre md5(case|cell) even", [c for c in pre if not md5odd(f"{c['k'][0]}|{c['k'][1]}")])
    show("pre by-case split: md5(case) odd", [c for c in pre if md5odd("v:" + c["k"][0])])
    show("pre by-case split: md5(case) even", [c for c in pre if not md5odd("v:" + c["k"][0])])
    show("pre non-BETA", [c for c in pre if c["fn"] not in ("BETA.DIST", "BETADIST")])
    show("pre gated", [c for c in pre if c["gated"]])
    show("w2f all", w2f)
    # duplicates by (fn, args): the same evaluation repeated across cases
    seen = Counter((c["fn"], c["args"]) for c in pre)
    uniq = {}
    for c in pre:
        uniq.setdefault((c["fn"], c["args"]), c)
    print("pre distinct (fn,args):", len(uniq), " max repeat:", max(seen.values()))
    show("pre distinct (fn,args), first occurrence", list(uniq.values()))
    # long literals: more than 15 significant digits in a numeric argument (Excel's parser cuts them)
    def long(c):
        for s in c["args"].split("|"):
            if s in ("True", "False"):
                continue
            d = s.lower().split("e")[0].replace("-", "").replace(".", "").lstrip("0")
            if len(d) > 15:
                return True
        return False
    lg = [c for c in pre if long(c)]
    show("pre cells with a >15-digit literal arg", lg)
    # ladder: per-step visibly-off left, using my own grouping
    steps = [("S1 closed", lambda b: b.startswith("closed")),
             ("S2 M1 pdf", lambda b: b == "pdfM1"),
             ("S3 bpser", lambda b: b in ("w_bpser", "w1_bpser", "bpser(a,b>1)")),
             ("S4 L140 bpser", lambda b: b == "L140 bup+bpser"),
             ("S5 bfrac/basym", lambda b: b in ("bfrac", "basym")),
             ("S6 bgrat not T", lambda b: b in BGRAT),
             ("S7 LDe", lambda b: b == "pdf>1")]
    for name, rs in (("pre", pre), ("pre md5 odd", [c for c in pre if md5odd(f"{c['k'][0]}|{c['k'][1]}")]),
                     ("pre by-case odd", [c for c in pre if md5odd("v:" + c["k"][0])]), ("w2f", w2f)):
        on = []
        out = []
        for lab, pr in steps:
            on.append(pr)
            vis = 0
            for c in rs:
                use = any(p(c["br"]) for p in on) and not (c["fn"] in TFN and c["br"] in BGRAT)
                m = c["best"] if use else c["eng"]
                vis += not w15(c["excel"], m)
            out.append(vis)
        print(f"ladder {name:16s} eng {sum(not w15(c['excel'], c['eng']) for c in rs)} -> " + " ".join(map(str, out)))
    print("branch counts pre:", sorted(Counter(c["br"] for c in pre).items(), key=lambda kv: -kv[1]))
    # positive control: closed-form cells bit-exact
    cl = [c for c in pre if c["br"].startswith("closed")]
    print("control closed forms pre: ", sum(c["best"] == c["excel"] for c in cl), "/", len(cl))


if __name__ == "__main__":
    main()
