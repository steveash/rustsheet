"""Continuous-template phase meter (T-1941 W5 part 5).
For a run of consecutive doubles and a rounding site k of the candidate (N/D chain stage j, P or S), the site's
rounding error with its pre-rounding value shifted by phi*u is e_phi(x) = (RN53(RN64(v + phi*u)) - (v + phi*u))/u.
Weighted by the site's propagation weight w_k(x) (relative output change per unit u at that site), it is the line
the site leaves in the output. The phase is phi* = argmax_phi corr(r, w*e_phi), where r = (B - smooth ref)/|ref|
in relative ulps, zero-meaned per run. Law-free: sawtooth, triangle, tie and doublet behaviour all come from the
exact rounding of v + phi*u."""

import numpy as np

L = np.longdouble


def st(v):
    return np.asarray(v, dtype=L).astype(np.float64)


PH = np.arange(64) / 64.0


def chain_sites(C, x):
    """stored values and exact pre-rounding values per site; returns dict (kind, j) -> (pre, stored), and S_j stored"""
    s = np.full_like(x, C[-1])
    out = {}
    for j in range(len(C) - 2, -1, -1):
        p_pre = L(s) * L(x)
        p = st(p_pre)
        out[("P", j)] = (p_pre, p)
        s_pre = L(p) + L(C[j])
        s = st(s_pre)
        out[("S", j)] = (s_pre, s)
    return out, s


def sites_and_weights(N, D, x):
    sn, Nv = chain_sites(N, x)
    sd, Dv = chain_sites(D, x)
    A = Nv / Dv
    t = x - 1.0
    ref = t * A
    sg = np.sign(ref)  # the residual is (B - ref)/abs(ref): relative change times sign(ref)
    W = {}
    for (kind, j), (
        pre,
        sv,
    ) in sn.items():  # dN at stage j propagates as x^j; relative output change dN*x^j/N
        W[("N", kind, j)] = (pre, sv, sg * x**j / Nv)
    for (kind, j), (pre, sv) in sd.items():
        W[("D", kind, j)] = (pre, sv, -sg * x**j / Dv)
    return W, ref


def template(pre, sv, w, phis=PH):
    """matrix (len(phis), n): weighted rounding-error sequences in relative-output units of 2^-52"""
    u = np.spacing(np.abs(sv)).astype(L)
    T = np.empty((len(phis), len(sv)))
    for i, ph in enumerate(phis):
        v = pre + L(ph) * u
        e = L(st(v)) - v  # absolute error
        T[i] = (e * L(w)).astype(float) / 2.0**-52
    return T


def resid(x, b, ref):
    r = ((L(b) - L(ref)) / (np.abs(L(ref)) * L(2.0**-52))).astype(float)
    return r - r.mean()


def site_phase(r, T):
    Tc = T - T.mean(1, keepdims=True)
    nrm = np.sqrt((Tc * Tc).sum(1))
    c = (Tc @ r) / np.maximum(nrm, 1e-30)
    i = int(np.argmax(c))
    rn = np.sqrt((r * r).sum())
    return PH[i], c[i] / max(rn, 1e-30), float(np.median(nrm))
