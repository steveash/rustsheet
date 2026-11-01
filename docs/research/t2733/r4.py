"""T-2733 R4 guards (decision 0188 R4.1, R4.2, R2): hold-out table and fresh-sweep Gate A envelope.

Inputs are `rustsheet-eval show --all` dumps (old engine = main, new engine = this branch) and the case's oracle
sidecar. The 50-digit math evaluates Excel's documented PRICE/DURATION/MDURATION sums from the coupon parts
(COUPDAYBS A, COUPDAYS E, COUPDAYSNC DSC, COUPNUM N): Excel's own cells for the fresh sweep, the engine's (exact
integers) for the hold-out case.
Usage: python3 r4.py holdout CASE.case.json ORACLE.json OLD.show NEW.show COUP.json
       python3 r4.py fresh CASE.case.json ORACLE.json NEW.show
"""

import hashlib
import json
import re
import struct
import sys

import mpmath

mpmath.mp.dps = 50
CALL = re.compile(r"=(PRICE|DURATION|MDURATION)\((.*)\)$")


def show(path):
    out, cell = {}, None
    for line in open(path):
        m = re.match(r"Calc!(\w+)", line)
        if m:
            cell = m.group(1)
        m = re.match(r"\s+actual\s*: (.*)", line)
        if m:
            out[cell] = m.group(1).strip()
    return out


def num(v):
    try:
        return float(v)
    except (TypeError, ValueError):
        return None


def ulps(a, b):
    def key(x):
        i = struct.unpack("<q", struct.pack("<d", x))[0]
        return i if i >= 0 else -(i & 0x7FFFFFFFFFFFFFFF)
    return abs(key(a) - key(b))


def digits15(a, b):
    return f"{a:.14e}" == f"{b:.14e}"


def math_value(fn, args, parts):
    a = [mpmath.mpf(x) for x in args]
    if fn == "PRICE":
        rate, yld, red, f = a[2], a[3], a[4], a[5]
    else:
        rate, yld, red, f = a[2], a[3], mpmath.mpf(100), a[4]
    A, E, DSC, N = (mpmath.mpf(p) for p in parts)
    basis = (args[6] if fn == "PRICE" else args[5]) if len(args) > (6 if fn == "PRICE" else 5) else "0"
    if basis.strip() in ("2", "3"):
        # Actual/360 and actual/365: Excel's sums use E - A, not COUPDAYSNC (every basis-2/3 cell of the
        # fresh sweep disagrees with COUPDAYSNC beyond 1e-9 and agrees with E - A).
        DSC = E - A
    c = 100 * rate / f
    v = 1 + yld / f
    if fn == "PRICE":
        if N == 1:
            return (c + red) / (1 + (E - A) / E * yld / f) - c * A / E
        s = red / v ** (N - 1 + DSC / E)
        s += mpmath.fsum(c / v ** (k - 1 + DSC / E) for k in range(1, int(N) + 1))
        return s - c * A / E
    t = [k - 1 + DSC / E for k in range(1, int(N) + 1)]
    cf = [c] * int(N)
    cf[-1] += red
    pv = [x / v**tk for x, tk in zip(cf, t)]
    d = mpmath.fsum(tk * p for tk, p in zip(t, pv)) / mpmath.fsum(pv) / f
    return d if fn == "DURATION" else d / v


def oracle(path):
    return json.load(open(path))["sheets"]["Calc"]


def xval(o):
    return o["v"] if o.get("t") == "n" else ("#" + str(o.get("v")))


def envelope(engine, excel, m):
    e = num(engine)
    if isinstance(excel, str) or m is None:
        return isinstance(excel, str) and engine == excel or (e is not None and m is not None and abs(e - m) <= max(1e-9 * abs(m), 1e-300))
    if e is None:
        return False
    return digits15(e, excel) or abs(e - m) <= max(1e-9 * abs(m), 1e-300)


def scores(cells, vals, excel):
    exact = ulp4 = d15 = 0
    for c in cells:
        x, v = excel[c], vals.get(c)
        if isinstance(x, str):
            ok = v is not None and str(v).upper().replace("#", "") in x.upper()
            exact += ok; ulp4 += ok; d15 += ok
            continue
        v = num(v)
        if v is None:
            continue
        exact += v == x
        ulp4 += ulps(v, x) <= 4
        d15 += digits15(v, x)
    return exact, ulp4, d15


def calls(case):
    out = {}
    for c, f in json.load(open(case))["sheets"][0]["cells"].items():
        m = CALL.match(str(f))
        if m:
            out[c] = (m.group(1), m.group(2).split(","))
    return out


def parts_key(fn, a):
    return (a[0], a[1], a[5], a[6] if len(a) > 6 else "0") if fn == "PRICE" else (a[0], a[1], a[4], a[5] if len(a) > 5 else "0")


def holdout(case, orc, old, new, coup):
    cid = "probes/bond-extreme-coupon-yield"
    cs = calls(case)
    ex = {c: xval(o) for c, o in oracle(orc).items() if c in cs}
    cj = json.load(open(coup))
    cr = {}
    for c, (fn, a) in cs.items():
        st, mt, f, b = parts_key(fn, a)
        p = [cj[f"={n}({st},{mt},{f},{b})"] for n in ("COUPDAYBS", "COUPDAYS", "COUPDAYSNC", "COUPNUM")]
        try:
            cr[c] = repr(float(math_value(fn, [x for x in a], p)))
        except (OverflowError, ZeroDivisionError, ValueError):
            cr[c] = None
    hold = sorted(c for c in cs if hashlib.sha256(f"T-2733:{cid}!{c}".encode()).digest()[0] < 77)
    train = sorted(set(cs) - set(hold))
    print(f"cells {len(cs)}: train {len(train)}, hold-out {len(hold)}")
    for name, vals in (("old (main)", show(old)), ("new (T-2733)", show(new)), ("correctly rounded", cr)):
        tr, ho = scores(train, vals, ex), scores(hold, vals, ex)
        print(f"| {name} | {tr[0]}/{len(train)} | {tr[1]} | {tr[2]} | {ho[0]}/{len(hold)} | {ho[1]} | {ho[2]} |")


def fresh(case, orc, new):
    cs = calls(case)
    o = oracle(orc)
    eng = show(new)
    strata = {}
    outside = []
    for c, (fn, a) in cs.items():
        row = int(re.sub(r"\D", "", c))
        p = [o.get(f"{col}{row}", {}).get("v") for col in "DEFG"]
        x = xval(o[c])
        try:
            m = math_value(fn, a, p) if all(isinstance(q, (int, float)) for q in p) else None
            m = float(m) if m is not None and abs(m) < 1.8e308 else None
        except (OverflowError, ZeroDivisionError, ValueError):
            m = None
        s = (row - 1) // 50
        st = strata.setdefault(s, [0, 0, 0, 0, 0])
        e = num(eng.get(c))
        st[0] += 1
        st[1] += not isinstance(x, str) and e is not None and digits15(e, x)
        st[2] += m is not None and e is not None and abs(e - m) <= max(1e-9 * abs(m), 1e-300)
        st[3] += isinstance(x, str) and eng.get(c) == x
        ok = envelope(eng.get(c), x, m)
        st[4] += ok
        if not ok:
            outside.append((c, o[c] if True else None, eng.get(c), m, json.load(open(case))["sheets"][0]["cells"][c]))
    names = ["rate 1E-4..1, yld 1E-4..1E-1", "rate 1E-4..1, yld 1E-1..10", "rate 1E-4..1, yld 10..1E4",
             "rate 1..1E4, yld 1E-4..1E-1", "rate 1..1E4, yld 1E-1..10", "rate 1..1E4, yld 10..1E4"]
    for s, st in sorted(strata.items()):
        print(f"| {names[s]} | {st[0]} | {st[1]} | {st[2]} | {st[3]} | {st[4]} | {st[0]-st[4]} |")
    print("outside:")
    for r in outside:
        print(" ", r)


if __name__ == "__main__":
    if sys.argv[1] == "holdout":
        holdout(*sys.argv[2:7])
    else:
        fresh(*sys.argv[2:5])
