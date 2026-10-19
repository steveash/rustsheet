# T-1941 W6: synthetic Excel-like data for positive controls. Given a chain file (lines "N hex..." "D hex..." ascending),
# simulate B = fl(t*fl(m*fl(N/D))) (every op stored) on the x grid of a real R.txt (rows of the cases given) and write
# a synthetic R.txt in the same format (R candidates re-extracted from the synthetic B, like rext.py).
import sys, numpy as np
sys.path.insert(0,sys.path[0]); from lib6 import st, L, horner
def read_chain(p):
    N=D=None
    for l in open(p):
        q=l.split()
        if q and q[0]=='N': N=[float.fromhex(v) for v in q[1:]]
        if q and q[0]=='D': D=[float.fromhex(v) for v in q[1:]]
    return N,D
def simB(N,D,x):
    R=st(L(horner(N[::-1],x))/L(horner(D[::-1],x))); return st(L(x-1.0)*L(st(L(x-2.0)*L(R)))),R
if __name__=='__main__':
    N,D=read_chain(sys.argv[1]); rows=[l.split() for l in open(sys.argv[2]) if l.split()[5].startswith(sys.argv[4])]
    x=np.array([float.fromhex(r[0]) for r in rows]); B,_=simB(N,D,x)
    t=x-1.0; m=x-2.0; R0=B/(t*m); H=[]
    def nb(v,k):
        for _ in range(abs(k)): v=np.nextafter(v,np.inf if k>0 else -np.inf)
        return v
    for k in range(-5,6):
        Rk=nb(R0,k); H.append(np.where(st(L(t)*L(st(L(m)*L(Rk))))==B,Rk,np.nan))
    H=np.array(H); lo=np.nanmin(H,0); hi=np.nanmax(H,0); n=np.sum(~np.isnan(H),0)
    with open(sys.argv[3],'w') as o:
        for i,r in enumerate(rows): o.write('%s %s %s %s %d %s\n'%(r[0],B[i].hex(),lo[i].hex(),hi[i].hex(),n[i],r[5]))
