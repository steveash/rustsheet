import ctypes, numpy as np, os
_l=ctypes.CDLL(os.path.join(os.path.dirname(os.path.abspath(__file__)),'libgsimk.so')); P=ctypes.POINTER(ctypes.c_double)
_l.gcountk.argtypes=[ctypes.c_int,P,P,ctypes.c_uint,ctypes.c_uint,ctypes.c_long,P,P]; _l.gcountk.restype=ctypes.c_long
_l.gsimk.argtypes=[ctypes.c_int,P,P,ctypes.c_uint,ctypes.c_uint,ctypes.c_long,P,P]
def _p(a): return np.ascontiguousarray(a,dtype=np.float64).ctypes.data_as(P)
def mask(ops):   # ops like ['add4'] (1-based stage)
    m=0
    for o in ops: k=int(o[3:])-1; m|=1<<(2*k+(1 if o[:3]=='add' else 0))
    return m
def count(N,D,mN,mD,y,t): y=np.ascontiguousarray(y,float); t=np.ascontiguousarray(t,float); return _l.gcountk(7,_p(N),_p(D),mN,mD,len(y),_p(y),_p(t))
def G(N,D,mN,mD,y): y=np.ascontiguousarray(y,float); o=np.empty(len(y)); _l.gsimk(7,_p(N),_p(D),mN,mD,len(y),_p(y),_p(o)); return o
