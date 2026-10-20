"""K1 helpers for the fingerprint (loader of N/D candidate files, masked G)."""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, "..", "p5"))
from fp import L, st
from maskmodel import chain
def ldND(path):
    N = D = None
    for l in open(path):
        if l.startswith('N '): N = np.array([float.fromhex(v) for v in l.split()[1:]])
        if l.startswith('D '): D = np.array([float.fromhex(v) for v in l.split()[1:]])
    return N, D
def G(x, N, D, kn=99, kd=99):
    xl = L(x); return st(chain(N, xl, kn)/chain(D, xl, kd))
