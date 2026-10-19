"""T-1941 W5: rational Remez (mpmath) for f on [a,b], weight w (relative by default).
Returns ascending raw-x coefficients (p, q) with q[0] = 1, plus the levelled error E."""
import mpmath as mp
def remez(f, a, b, m, n, w=None, dps=60, iters=40, grid=4000, verbose=False, ref=None):
    mp.mp.dps = dps
    a = mp.mpf(a); b = mp.mpf(b); c = (a+b)/2; h = (b-a)/2
    if w is None: w = f
    K = m+n+2
    if ref is None:
        X = [c - h*mp.cos(mp.pi*i/(K-1)) for i in range(K)]
    else: X = ref
    def u(x): return (x-c)/h
    def basis(x, k): return mp.chebyt(k, u(x))
    def solve(X, E):
        M = mp.matrix(K, K)
        for i, x in enumerate(X):
            fi = f(x); wi = w(x); s = (-1)**i
            for k in range(m+1): M[i, k] = basis(x, k)
            for k in range(n+1): M[i, m+1+k] = -(fi + s*E*wi)*basis(x, k)
        return M
    def detE(E): return mp.det(solve(X, E))
    E = mp.mpf(0)
    gx = [a + (b-a)*mp.mpf(i)/grid for i in range(grid+1)]
    for it in range(iters):
        # E root of det
        E = mp.findroot(detE, (E, E + mp.mpf('1e-30') if E == 0 else E*(1+mp.mpf('1e-6'))), solver='secant')
        M = solve(X, E)
        # null vector: fix q0 coefficient (Chebyshev T0) = 1 by solving reduced system
        A = mp.matrix(K, K-1); rhs = mp.matrix(K, 1)
        cols = [j for j in range(K) if j != m+1]
        for i in range(K):
            for jj, j in enumerate(cols): A[i, jj] = M[i, j]
            rhs[i] = -M[i, m+1]
        sol = mp.lu_solve(A[:K-1, :], rhs[:K-1, :])
        coef = list(sol[:m+1]) + [mp.mpf(1)] + list(sol[m+1:])
        P = coef[:m+1]; Q = coef[m+1:]
        def R(x): return sum(P[k]*basis(x, k) for k in range(m+1))/sum(Q[k]*basis(x, k) for k in range(n+1))
        def err(x): return (R(x)-f(x))/w(x)
        ev = [err(x) for x in gx]
        # alternating extrema
        segs = []; cur = None
        for i, e in enumerate(ev):
            sg = 1 if e >= 0 else -1
            if cur is None or sg != cur[0]: segs.append([sg, i]); cur = segs[-1]
            elif abs(e) > abs(ev[cur[1]]): cur[1] = i
        while len(segs) > K:
            # drop smallest end
            if abs(ev[segs[0][1]]) < abs(ev[segs[-1][1]]): segs.pop(0)
            else: segs.pop()
        if len(segs) < K:
            if verbose: print('lost alternation', len(segs)); 
            break
        newX = []
        for sg, i in segs:
            lo = gx[max(i-1, 0)]; hi = gx[min(i+1, grid)]
            try:
                xm = mp.findroot(lambda t: mp.diff(err, t), gx[i]) if 0 < i < grid else gx[i]
                if not (lo <= xm <= hi): xm = gx[i]
            except Exception: xm = gx[i]
            newX.append(xm)
        mx = max(abs(err(x)) for x in newX)
        if verbose: print(it, mp.nstr(E, 8), mp.nstr(mx, 8))
        X = newX
        if abs(mx/abs(E)-1) < mp.mpf('1e-8'): break
    # convert Chebyshev-in-u to raw x monomials
    def tomono(C):
        out = [mp.mpf(0)]*len(C)
        for k, ck in enumerate(C):
            tk = mp.chebyt  # build T_k(u) poly in x via recurrence
        # recurrence on polynomials in x
        T0 = [mp.mpf(1)]; T1 = [-c/h, 1/h]
        Ts = [T0, T1]
        for k in range(2, len(C)):
            a1 = [mp.mpf(0)] + [2*v/h for v in Ts[-1]]
            a2 = [-2*c/h*v for v in Ts[-1]] + [mp.mpf(0)]
            t = [a1[i]+a2[i] - (Ts[-2][i] if i < len(Ts[-2]) else 0) for i in range(len(a1))]
            Ts.append(t)
        res = [mp.mpf(0)]*len(C)
        for k, ck in enumerate(C):
            for i, v in enumerate(Ts[k]): res[i] += ck*v
        return res
    p = tomono(P); q = tomono(Q); q0 = q[0]
    return [v/q0 for v in p], [v/q0 for v in q], E, X
