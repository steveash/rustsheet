"""T-1941 W5: robust rational Remez (mpmath). Newton on (p, q, E) at the reference, real-only extremum search.
remez2(f, a, b, m, n, w) -> (p, q, E, X): ascending raw-x coefficients with q[0] = 1, levelled error E, reference X.
The error minimised is (R - f)/w; w = f gives relative error."""

import mpmath as mp


def _cheb_to_mono(C, c, h):
    T = [[mp.mpf(1)], [-c / h, 1 / h]]
    for k in range(2, len(C)):
        a1 = [mp.mpf(0)] + [2 * v / h for v in T[-1]]
        a2 = [-2 * c / h * v for v in T[-1]] + [mp.mpf(0)]
        T.append([a1[i] + a2[i] - (T[-2][i] if i < len(T[-2]) else 0) for i in range(len(a1))])
    out = [mp.mpf(0)] * len(C)
    for k, ck in enumerate(C):
        for i, v in enumerate(T[k]):
            out[i] += ck * v
    return out


def remez2(f, a, b, m, n, w=None, dps=40, iters=30, grid=600, ref=None, verbose=False, tol="1e-10"):
    mp.mp.dps = dps
    a = mp.mpf(a)
    b = mp.mpf(b)
    c = (a + b) / 2
    h = (b - a) / 2
    if w is None:
        w = f
    K = m + n + 2
    X = list(ref) if ref is not None else [c - h * mp.cos(mp.pi * i / (K - 1)) for i in range(K)]
    T = lambda x, k: mp.chebyt(k, (x - c) / h)
    P = [mp.mpf(0)] * (m + 1)
    Q = [mp.mpf(1)] + [mp.mpf(0)] * n
    E = mp.mpf(0)

    def R(x):
        return mp.fsum(P[k] * T(x, k) for k in range(m + 1)) / mp.fsum(Q[k] * T(x, k) for k in range(n + 1))

    def err(x):
        return (R(x) - f(x)) / w(x)

    for it in range(iters):
        fx = [f(x) for x in X]
        wx = [w(x) for x in X]
        tx = [[T(x, k) for k in range(max(m, n) + 1)] for x in X]
        for nt in range(30):  # Newton on unknowns p0..pm, q1..qn, E
            Fv = mp.matrix(K, 1)
            J = mp.matrix(K, K)
            for i in range(K):
                s = (-1) ** i
                qv = mp.fsum(Q[k] * tx[i][k] for k in range(n + 1))
                pv = mp.fsum(P[k] * tx[i][k] for k in range(m + 1))
                g = fx[i] + s * E * wx[i]
                Fv[i] = pv - g * qv
                for k in range(m + 1):
                    J[i, k] = tx[i][k]
                for k in range(1, n + 1):
                    J[i, m + k] = -g * tx[i][k]
                J[i, K - 1] = -s * wx[i] * qv
            d = mp.lu_solve(J, -Fv)
            for k in range(m + 1):
                P[k] += d[k]
            for k in range(1, n + 1):
                Q[k] += d[m + k]
            E += d[K - 1]
            if mp.norm(d) < mp.mpf(10) ** (-dps + 8):
                break
        # new reference: alternating extrema of err on a grid, refined by local golden search
        gx = [a + (b - a) * mp.mpf(i) / grid for i in range(grid + 1)]
        ev = [err(x) for x in gx]
        segs = []
        for i, e in enumerate(ev):
            sg = 1 if e >= 0 else -1
            if not segs or sg != segs[-1][0]:
                segs.append([sg, i])
            elif abs(e) > abs(ev[segs[-1][1]]):
                segs[-1][1] = i
        while len(segs) > K:
            if abs(ev[segs[0][1]]) < abs(ev[segs[-1][1]]):
                segs.pop(0)
            else:
                segs.pop()
        if len(segs) < K:
            raise RuntimeError("alternation lost (%d of %d)" % (len(segs), K))
        newX = []
        for sg, i in segs:
            if i == 0 or i == grid:
                newX.append(gx[i])
                continue
            lo, hi = gx[i - 1], gx[i + 1]
            g = (mp.sqrt(5) - 1) / 2
            for _ in range(40):
                x1 = hi - g * (hi - lo)
                x2 = lo + g * (hi - lo)
                if sg * err(x1) > sg * err(x2):
                    hi = x2
                else:
                    lo = x1
            newX.append((lo + hi) / 2)
        mx = max(abs(err(x)) for x in newX)
        if verbose:
            print(it, mp.nstr(E, 10), mp.nstr(mx, 10))
        X = newX
        if abs(mx / abs(E) - 1) < mp.mpf(tol):
            break
    p = _cheb_to_mono(P, c, h)
    q = _cheb_to_mono(Q, c, h)
    q0 = q[0]
    return [v / q0 for v in p], [v / q0 for v in q], E, X
