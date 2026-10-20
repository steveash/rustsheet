# ctypes wrapper for gsimh.c (K1 stored Horner with per-sum hidden e/y terms).
import ctypes, numpy as np, os
_l=ctypes.CDLL(os.path.join(os.path.dirname(os.path.abspath(__file__)),'libgsimh.so')); P=ctypes.POINTER(ctypes.c_double)
_l.gcounth.argtypes=[ctypes.c_int,P,P,P,P,ctypes.c_long,P,P]; _l.gcounth.restype=ctypes.c_long
_l.gsimh.argtypes=[ctypes.c_int,P,P,P,P,ctypes.c_long,P,P]
def _p(a): return np.ascontiguousarray(a,dtype=np.float64).ctypes.data_as(P)
def count(N,D,eN,eD,y,t): return _l.gcounth(7,_p(N),_p(D),_p(eN),_p(eD),len(y),_p(y),_p(t))
def G(N,D,eN,eD,y): o=np.empty(len(y)); _l.gsimh(7,_p(N),_p(D),_p(eN),_p(eD),len(y),_p(y),_p(o)); return o
