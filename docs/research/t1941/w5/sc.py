"""T-1941 W5 helpers: data loading and the exact scorer (libsc.so)."""
import ctypes, os, numpy as np
_d=os.path.dirname(os.path.abspath(__file__))
_lib=ctypes.CDLL(os.path.join(_d,'libsc.so'))
Pd=ctypes.POINTER(ctypes.c_double); Pl=ctypes.POINTER(ctypes.c_long)
_lib.score.restype=ctypes.c_long
_lib.score.argtypes=[Pd,Pd,ctypes.c_long,Pd,ctypes.c_int,Pd,ctypes.c_int]
_lib.resv.argtypes=[Pd,ctypes.c_long,Pd,ctypes.c_int,Pd,ctypes.c_int,Pd]
_lib.score_batch.argtypes=[Pd,Pd,ctypes.c_long,Pd,ctypes.c_int,Pd,ctypes.c_int,ctypes.c_int,Pl]
def _p(a): return a.ctypes.data_as(Pd)
def _c(a): return np.ascontiguousarray(a,dtype=np.float64)
def score(x,b,N,D):
    N=_c(N);D=_c(D); return _lib.score(_p(x),_p(b),len(x),_p(N),len(N),_p(D),len(D))/len(x)
def res(x,N,D):
    N=_c(N);D=_c(D);o=np.empty_like(x); _lib.resv(_p(x),len(x),_p(N),len(N),_p(D),len(D),_p(o)); return o
def score_batch(x,b,P,Q):
    P=_c(P);Q=_c(Q);o=np.empty(P.shape[0],dtype=np.int64)
    _lib.score_batch(_p(x),_p(b),len(x),_p(P),P.shape[1],_p(Q),Q.shape[1],P.shape[0],o.ctypes.data_as(Pl)); return o/len(x)
ROWS=os.environ.get('ROWS','/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows.txt')
def load(lo=0.7,hi=1.5,cache=os.path.join(os.path.dirname(ROWS),'near1.npz')):
    """Unique (x,B,case) with lo<=x<hi and finite B."""
    if os.path.exists(cache):
        z=np.load(cache); return z['x'],z['b'],z['c']
    X=[];B=[];C=[]
    for l in open(ROWS):
        xh,bh,_,_,nm=l.split()
        if bh=='nan': continue
        x=float.fromhex(xh)
        if lo<=x<hi: X.append(x);B.append(float.fromhex(bh));C.append(nm)
    x=np.array(X);b=np.array(B);c=np.array(C)
    x,ui=np.unique(x,return_index=True); b=b[ui]; c=c[ui]
    np.savez(cache,x=x,b=b,c=c); return x,b,c
def candidate(tag,path=os.path.join(_d,'..','near1_candidate_w1.txt')):
    for l in open(path):
        if l.startswith(tag+' '): return np.array([float.fromhex(v) if v!='0' else 0.0 for v in l.split()[2:]])
    raise KeyError(tag)
