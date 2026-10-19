# multi-anchor joint linearised phase solve. runs: (anchor a, y array, excel, resid)
import numpy as np, math, sys, json, ctypes, os
from fractions import Fraction as Fr
import mpmath as mp

L = ctypes.CDLL("./libgeval.so")
P = ctypes.POINTER(ctypes.c_double)


def model(x, y):
    q = np.empty_like(y)
    N = np.ascontiguousarray(x[:9])
    D = np.ascontiguousarray(x[9:])
    L.geval(len(y), y.ctypes.data_as(P), q.ctypes.data_as(P), N.ctypes.data_as(P), D.ctypes.data_as(P))
    return q


EPS = Fr(1, 2**52)
ULP = 2.0**-53
FREEZE = [int(v) for v in os.environ.get("FREEZE", "").split(",") if v]
LINES = (
    [("D", 0, "saw"), ("D", 1, "tri"), ("N", 0, "tri")]
    + ([("D", 2, "tri"), ("N", 1, "tri")] if os.environ.get("MORE") else [])
    + ([("N", 0, "saw"), ("D", 1, "saw")] if os.environ.get("MORE2") else [])
)


def Spoly(C, jj):
    return C[jj:]


def pval(c, y):
    s = Fr(0)
    for cc in reversed(c):
        s = s * y + cc
    return s


def theta_and_row(x, ch, j, typ, a, y):
    C = [Fr(v) for v in (x[:9] if ch == "N" else x[9:])]
    off = 0 if ch == "N" else 9
    jj = j + 1 if typ == "saw" else j
    S = Spoly(C, jj)
    Sa = pval(S, a)
    e = math.frexp(float(Sa))[1] - 1
    row = np.zeros(18)
    if typ == "saw":
        sc = Fr(2) ** e * EPS
        th = ((y - 1) * pval(S, y) - (a - 1) * Sa) / sc
        for i in range(jj, 9):
            row[off + i] = float(((y - 1) * y ** (i - jj) - (a - 1) * a ** (i - jj)) / sc)
    else:
        sc = Fr(2) ** (e + 1) * EPS
        th = (pval(S, y) - Sa) / sc
        for i in range(jj, 9):
            row[off + i] = float((y ** (i - jj) - a ** (i - jj)) / sc)
    return th, row, e


def measure_run(x, ch, j, typ, a, yv, g, rd):
    n = len(yv)
    mid = n // 2
    th0, row, e = theta_and_row(x, ch, j, typ, a, Fr(yv[mid]))
    th1, _, _ = theta_and_row(x, ch, j, typ, a, Fr(yv[mid + 1]))
    thm, _, _ = theta_and_row(x, ch, j, typ, a, Fr(yv[mid - 1]))
    f = float(((th1 - thm) / 2) % 1)
    fdd = float(th1 - 2 * th0 + thm)
    th0 = float(th0 - math.floor(th0))
    k = np.arange(n) - mid
    ph = th0 + f * k + 0.5 * fdd * k * k
    Ez = np.exp(-2j * np.pi * ph) * np.hanning(n)
    q = model(x, yv)
    rs = rd + (q - g) / ULP
    Ad = np.dot(rd - rd.mean(), Ez)
    As = np.dot(rs - rs.mean(), Ez)
    return np.angle(Ad / As) / 2 / np.pi, row, abs(Ad), abs(As)


def solve_joint(x, runs, lines=LINES, WF=5e14, kmin=5e3, log=True):
    meas = []
    for ai, (a, yv, g, rd) in enumerate(runs):
        K = float((Fr(yv[len(yv) // 2]) - a) / EPS)
        for li, (ch, j, typ) in enumerate(lines):
            dphi, row, Ad, As = measure_run(x, ch, j, typ, a, yv, g, rd)
            if min(Ad, As) < 50:
                continue
            meas.append(dict(li=li, a=float(a), K=abs(K), ph=dphi, row=row, amp=min(Ad, As)))
    keys = sorted(set((m["li"], m["a"]) for m in meas))
    kidx = {k: i for i, k in enumerate(keys)}
    nc = len(keys)
    mp.mp.dps = 40
    ys = [mp.mpf(1.5) + mp.mpf(0.5) * mp.cos(mp.pi * (i + mp.mpf(0.5)) / 60) for i in range(60)]
    FR = []
    FB = []
    for t in ys:
        gg = mp.gamma(t)
        Nv = sum(mp.mpf(x[i]) * t**i for i in range(9))
        Dv = sum(mp.mpf(x[9 + i]) * t**i for i in range(9))
        FR.append(
            [float(WF * t**i / (gg * Dv)) for i in range(9)]
            + [float(-WF * t**i / Dv) for i in range(9)]
            + [0] * nc
        )
        FB.append(float(-WF * (Nv - gg * Dv) / (gg * Dv)))
    FR = np.array(FR)
    FB = np.array(FB)
    order = sorted(range(len(meas)), key=lambda i: meas[i]["K"])

    def lsq(sel, nint):
        A = np.zeros((len(sel), 18 + nc))
        b = np.zeros(len(sel))
        for r, i in enumerate(sel):
            m = meas[i]
            A[r, :18] = m["row"]
            A[r, 18 + kidx[(m["li"], m["a"])]] = 1
            b[r] = m["ph"] + nint[i]
        A = np.vstack([A, FR])
        b = np.concatenate([b, FB])
        keep = [c for c in range(A.shape[1]) if c not in FREEZE]
        Ak = A[:, keep]
        sc = np.linalg.norm(Ak, axis=0)
        sc[sc == 0] = 1
        s, *_ = np.linalg.lstsq(Ak / sc, b, rcond=1e-15)
        full = np.zeros(A.shape[1])
        full[keep] = s / sc
        return full

    nint = {i: 0 for i in order}
    kmax = kmin
    sel = [i for i in order if meas[i]["K"] <= kmax]
    sol = lsq(sel, nint)
    while kmax < 3e14:
        kmax *= 1.5
        new = [i for i in order if kmax / 2 < meas[i]["K"] <= kmax]
        if not new:
            continue
        for i in new:
            m = meas[i]
            p = m["row"] @ sol[:18] + sol[18 + kidx[(m["li"], m["a"])]]
            nint[i] = int(round(p - m["ph"]))
        sel = [i for i in order if meas[i]["K"] <= kmax]
        sol = lsq(sel, nint)
        res = {}
        for i in sel:
            m = meas[i]
            r = m["row"] @ sol[:18] + sol[18 + kidx[(m["li"], m["a"])]] - (m["ph"] + nint[i])
            res.setdefault((m["li"], m["a"]), []).append(r)
        if log:
            print(
                "kmax %.1e " % kmax
                + " ".join("L%d@%.2f %.4f" % (k[0], k[1], np.std(v)) for k, v in sorted(res.items())),
                flush=True,
            )
    global LAST_CONST
    LAST_CONST = {k: float(sol[18 + i]) for k, i in kidx.items()}
    return sol[:18]


def load_e3():
    rows = [l.split() for l in open("../e3res.txt")]
    y = np.array([float.fromhex(r[0]) for r in rows])
    g = np.array([float.fromhex(r[1]) for r in rows])
    rd = np.array([float(r[2]) for r in rows])
    brk = [0] + [i for i in range(1, len(y)) if not (0 < y[i] - y[i - 1] < 1e-12)] + [len(y)]
    out = [(Fr(1), y[a:b], g[a:b], rd[a:b]) for a, b in zip(brk[:-1], brk[1:]) if y[a] < 1.5]
    if os.environ.get("A2"):
        out += [
            (Fr(2), y[a:b], g[a:b], rd[a:b])
            for a, b in zip(brk[:-1], brk[1:])
            if 1.5 <= y[a] < 2 and b - a >= 1024
        ]
    return out


def apply(x, dx):
    return np.array([float(Fr(x[i]) + Fr(dx[i])) for i in range(18)])


def load_rows(fn, prefix, anchors=(Fr(5, 4), Fr(3, 2), Fr(7, 4)), minlen=1024):
    mp.mp.dps = 30
    rows = []
    for l in open(fn):
        p = l.split()
        if not p[4].startswith(prefix) or p[3] == "nan":
            continue
        rows.append((float.fromhex(p[0]), float.fromhex(p[3])))
    rows.sort()
    out = []
    s = 0
    for i in range(1, len(rows) + 1):
        if i == len(rows) or rows[i][0] - rows[i - 1][0] != math.ulp(rows[i - 1][0]):
            if i - s >= minlen:
                yv = np.array([r[0] for r in rows[s:i]])
                g = np.array([r[1] for r in rows[s:i]])
                rd = np.array(
                    [float((mp.mpf(b) - mp.gamma(mp.mpf(a))) / mp.mpf(2) ** -53) for a, b in rows[s:i]]
                )
                a = min(anchors, key=lambda A: abs(float(A) - yv[0]))
                out.append((a, yv, g, rd))
            s = i
    return out
