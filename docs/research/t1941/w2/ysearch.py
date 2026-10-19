# T-1941 W2: exact-match scorer and function-preserving ("null direction") coefficient-segment search over [2,3].
import numpy as np,ctypes,math,pickle,sys,struct
L=ctypes.CDLL('./libpoly.so');DP=ctypes.POINTER(ctypes.c_double)
L.gcount.restype=ctypes.c_int
def P(a): return np.ascontiguousarray(a,float).ctypes.data_as(DP)
class Scorer:
    def __init__(s,X,Y): s.X=np.ascontiguousarray(X,float); s.Y=np.ascontiguousarray(Y,float); s.N=len(X)
    def __call__(s,p,q): return L.gcount(P(p),len(p),P(q),len(q),s.X.ctypes.data_as(DP),s.Y.ctypes.data_as(DP),s.N)
def nextd(v,k):
    i=struct.unpack('<q',struct.pack('<d',v))[0]; i+= k if v>=0 else -k
    return struct.unpack('<d',struct.pack('<q',i))[0]
def null_dir(c,k,xg=np.linspace(2,3,20)):
    u=[math.ulp(abs(v)) for v in c[:k+1]]
    A=np.array([[x**(k-i)*u[i] for i in range(1,k+1)] for x in xg]); b=-np.array([x**k*u[0] for x in xg])
    return np.concatenate([[1.0],np.linalg.lstsq(A,b,rcond=None)[0]])
def dsearch(p,q,sc,which,k,T,step=1,off=0):
    """which: 'p'/'q'; moves coefs c[off..off+k] along the function-preserving direction over [2,3]"""
    c=list(p if which=='p' else q); seg=c[off:off+k+1]; d=null_dir(seg,k)
    best=(sc(p,q),0,None)
    for t in range(-T,T+1,step):
        cc=list(c)
        for i,di in enumerate(d): cc[off+i]=nextd(c[off+i],int(round(t*di)))
        s=sc(cc,q) if which=='p' else sc(p,cc)
        if s>best[0]: best=(s,t,cc)
    if best[2] is None: return p,q,best[:2]
    return (best[2],q,best[:2]) if which=='p' else (p,best[2],best[:2])
