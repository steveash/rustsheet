"""K3 masked x87 model (T-1941 W5 part 5). Raw-x Horner, C[-1] leading; ops at coefficient index j are
P_j = S_{j+1} * x and S_j = P_j + C_j. Mask "stored below k": those two ops are rounded fl53(fl64) iff j < k, otherwise
kept in 80-bit (64-bit mantissa, numpy longdouble). A = fl(N/D) and res = fl(fl(x-1) * A) are always stored.
k = 99: everything stored (the plain model of sc.c); k = 0: the whole chain stays 80-bit until the division."""
import numpy as np
from meter import L, st
def chain(C, xl, k):
    s = L(C[-1])
    for j in range(len(C)-2, -1, -1):
        p = s*xl; p = L(st(p)) if j < k else p
        s = p+L(C[j]); s = L(st(s)) if j < k else s
    return s
def res(x, N, D, kn=99, kd=99):
    xl = L(x); t = L(st(xl-L(1.0)))
    return st(t*L(st(chain(N, xl, kn)/chain(D, xl, kd))))
