"""FFT scan of the integer N7 shift m (T-1941 W5 part 5).
Given hypothesis (N7_0, N8, N9), S7 is simulated exactly; S7 + m*u (u = ulp(S7)) is exact for every m, and the product
P6 = RN((S7 + m u) x) has rounding error e(theta), theta = (S7 x + m u x)/u_P. Expanding the round-to-nearest sawtooth
in harmonics, the P6 objective O(m) = sum_p a_p e(theta_p(m)) (a_p = weight * residual) is a non-uniform DFT in m,
evaluated for all integer m in [-M, M] by binning the frequencies h*x*u/u_P and one FFT per harmonic.
usage (library): mscan(data, N7, N8, N9, M)"""

import numpy as np, sys, os

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from meter import L, st


def top_S7(x, N7, N8, N9):
    s = np.full(len(x), N9)
    p = st(L(s) * L(x))
    s = st(L(p) + L(N8))
    p = st(L(s) * L(x))
    s = st(L(p) + L(N7))
    return s


def mscan(d, N7, N8, N9, M=20000, H=6, OS=8):
    x = d.x
    S7 = top_S7(x, N7, N8, N9)
    u = float(np.spacing(abs(np.median(S7))))
    v = L(S7) * L(x)
    uP = np.spacing(np.abs(st(v))).astype(L)
    F = d.weights("N", 7, 7)[0]  # weight of stage 6 (P6), units of 2^-52 per absolute
    a = F * d.r * uP.astype(float)  # error e in units of uP -> absolute
    th0 = (v / uP).astype(L)
    th0 = (th0 - np.floor(th0)).astype(float)
    fr = (L(u) * L(x) / uP).astype(float)  # cycles per unit m
    out = np.zeros(2 * M + 1)
    ms = np.arange(-M, M + 1)
    NF = 1 << 24                                 # df = 1/NF: q = m exactly; bins wrap mod NF (m integer)
    for h in range(1, H + 1):
        ch = ((-1) ** h) / (np.pi * h)          # round(t) - t = sum_h (-1)^h sin(2 pi h t)/(pi h)
        z = a * ch * np.exp(2j * np.pi * h * th0)
        k = np.round(h * fr * NF).astype(np.int64) % NF
        g = np.zeros(NF, complex); np.add.at(g, k, z)
        Gi = np.fft.ifft(g) * NF                 # Gi[q] = sum_k g_k e^{2 pi i k q / NF}
        out += np.imag(Gi[ms % NF])
    return ms, out, u
