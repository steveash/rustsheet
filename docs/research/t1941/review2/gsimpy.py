# ctypes wrapper for gsim.c (K1 exact simulator with per-stage nudges)
import ctypes, numpy as np, os
HERE=os.path.dirname(os.path.abspath(__file__))
_l=ctypes.CDLL(os.path.join(HERE,'libgsim.so'))
P=ctypes.POINTER(ctypes.c_double)
_l.gsim.argtypes=[ctypes.c_int]+[P]*6+[ctypes.c_long,P,P]
_l.gcount.argtypes=[ctypes.c_int]+[P]*6+[ctypes.c_long,P,P]; _l.gcount.restype=ctypes.c_long
_l.gtrace.argtypes=[ctypes.c_int,P,ctypes.c_long,P,P,P,P]
def _p(a): return np.ascontiguousarray(a,dtype=np.float64).ctypes.data_as(P)
class Sim:
    def __init__(s,N,D): s.N=np.array(N,float); s.D=np.array(D,float); s.n=len(N)-1; s.z=np.zeros(s.n)
    def G(s,y,pn=None,sn=None,pd=None,sd=None):
        y=np.ascontiguousarray(y,float); out=np.empty(len(y))
        _l.gsim(s.n,_p(s.N),_p(s.D),_p(pn if pn is not None else s.z),_p(sn if sn is not None else s.z),
                _p(pd if pd is not None else s.z),_p(sd if sd is not None else s.z),len(y),_p(y),_p(out)); return out
    def count(s,y,target,pn=None,sn=None,pd=None,sd=None):
        y=np.ascontiguousarray(y,float); target=np.ascontiguousarray(target,float)
        return _l.gcount(s.n,_p(s.N),_p(s.D),_p(pn if pn is not None else s.z),_p(sn if sn is not None else s.z),
                _p(pd if pd is not None else s.z),_p(sd if sd is not None else s.z),len(y),_p(y),_p(target))
    def trace(s,which,y,pn=None,sn=None):
        c=s.N if which=='N' else s.D; y=np.ascontiguousarray(y,float); t=np.empty((len(y),s.n,2))
        _l.gtrace(s.n,_p(c),len(y),_p(y),_p(pn if pn is not None else s.z),_p(sn if sn is not None else s.z),_p(t)); return t
