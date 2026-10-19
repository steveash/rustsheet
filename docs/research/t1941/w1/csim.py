import ctypes, numpy as np, os
_lib=ctypes.CDLL(os.path.join(os.path.dirname(os.path.abspath(__file__)),'libsimA.so'))
D=ctypes.POINTER(ctypes.c_double)
_lib.simA.argtypes=[D,ctypes.c_int,D,ctypes.c_int,D,ctypes.c_int,D]
_lib.simA_batch.argtypes=[D,ctypes.c_int,D,ctypes.c_int,D,ctypes.c_int,ctypes.c_int,D]
def _p(a): return a.ctypes.data_as(D)
def simA(x,p,q):
    x=np.ascontiguousarray(x,dtype=np.float64); p=np.ascontiguousarray(p,dtype=np.float64); q=np.ascontiguousarray(q,dtype=np.float64)
    out=np.empty_like(x); _lib.simA(_p(x),len(x),_p(p),len(p),_p(q),len(q),_p(out)); return out
def simA_batch(x,P,Q):
    x=np.ascontiguousarray(x,dtype=np.float64); P=np.ascontiguousarray(P,dtype=np.float64); Q=np.ascontiguousarray(Q,dtype=np.float64)
    m=P.shape[0]; out=np.empty((m,len(x))); _lib.simA_batch(_p(x),len(x),_p(P),P.shape[1],_p(Q),Q.shape[1],m,_p(out)); return out
