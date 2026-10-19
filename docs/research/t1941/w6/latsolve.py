# T-1941 W6: lattice (CVP by embedding + LLL/BKZ) recovery of the integer coefficient offsets m (in ulps of each
# coefficient; D0 fixed) between a candidate chain and the truth, from per-run line phases (mod 1) and the function
# constraint. Phase rows: T line of stage j: frac(dS_j/(2u)); n=1 sideband of P_j: frac(x dS_{j+1}/u_P - dS_{j+1}/u_M)
# (exact in mpmath; dS linear in m). Usage (control): latsolve.py TRUTH CAND ideal|noise sigma
import sys, numpy as np, mpmath as mp
from fpylll import IntegerMatrix, LLL, BKZ
sys.path.insert(0,sys.path[0]); from synth import read_chain; from longspec import X0
mp.mp.dps=80
def ulp(v): return mp.mpf(np.spacing(abs(float(v))))
def stages(c,x):
    n=len(c)-1; s=c[n]; S={n:s}; P={}
    for j in range(n-1,-1,-1): P[j]=s*x; s=P[j]+c[j]; S[j]=s
    return S,P
def phase_rows(N,D,lines,xs):
    """rows: list of (coefficient-gradient vector g (len nN+nD-1, per unit ulp), value phi_cand) for each (line, x)."""
    nN=len(N); out=[]
    uN=[ulp(v) for v in N]; uD=[ulp(v) for v in D]
    for x in xs:
        x=mp.mpf(x)
        for ch,law,j in lines:
            c=N if ch=='N' else D; uc=uN if ch=='N' else uD; S,P=stages([mp.mpf(v) for v in c],x)
            g=[mp.mpf(0)]*(nN+len(D)-1); off=0 if ch=='N' else nN-1
            def put(k,w):
                if ch=='D' and k==0: return
                g[(k if ch=='N' else off+k)]+=w
            if law=='T':
                uS=ulp(S[j]); val=S[j]/(2*uS)
                for k in range(j,len(c)): put(k,x**(k-j)*uc[k]/(2*uS))
            else:   # Pn1
                uP=ulp(P[j]); uM=ulp(S[j+1]); val=P[j]/uP-S[j+1]/uM
                for k in range(j+1,len(c)): put(k,x**(k-j)*uc[k]/uP-x**(k-j-1)*uc[k]/uM)
            out.append((g,val))
    return out
if __name__=='__main__':
    Nt,Dt=read_chain(sys.argv[1]); Nc,Dc=read_chain(sys.argv[2])
    Nt=[v/Dt[0] for v in Nt]; Dt=[v/Dt[0] for v in Dt]; Nc=[v/Dc[0] for v in Nc]; Dc=[v/Dc[0] for v in Dc]
    sig=float(sys.argv[4]); rng=np.random.default_rng(0)
    import os
    if os.environ.get('LINES'):
        lines=[tuple(v if i<2 else int(v) for i,v in enumerate(t.split(':'))) for t in os.environ['LINES'].split(',')]
    else:
        lines=[('N','T',4),('N','T',5),('D','T',4),('D','T',5)]+[('N','Pn1',j) for j in range(4)]+[('D','Pn1',j) for j in range(1,5)]
    xs=[x+32768*2.0**-52 for x in X0][:int(__import__('os').environ.get('NRUN','24'))]
    rows=phase_rows(Nc,Dc,lines,xs); rowsT=phase_rows(Nt,Dt,lines,xs)
    phi=[float((rt[1]-rc[1])%1) for rc,rt in zip(rows,rowsT)]
    if sys.argv[3]=='noise': phi=[(p+sig*rng.standard_normal())%1 for p in phi]
    nu=len(Nc)+len(Dc)-1
    m_true=[int(round((mp.mpf(a)-mp.mpf(b))/ulp(b))) for a,b in zip(Nt,Nc)]+[int(round((mp.mpf(a)-mp.mpf(b))/ulp(b))) for a,b in zip(Dt[1:],Dc[1:])]
    print('true m:',m_true)
    # function rows: (N - R D)/(R*Dc) at nodes, linear in m
    K=40; fx=[mp.mpf(1.75)+mp.mpf(0.25)*mp.cos(mp.pi*(k+mp.mpf(1)/2)/K) for k in range(K)]
    frows=[]
    for x in fx:
        r=mp.loggamma(x)/((x-1)*(x-2)); Dv=mp.polyval([mp.mpf(v) for v in Dc[::-1]],x); Nv=mp.polyval([mp.mpf(v) for v in Nc[::-1]],x)
        g=[x**k*ulp(Nc[k])/(r*Dv) for k in range(len(Nc))]+[-r*x**k*ulp(Dc[k])/(r*Dv) for k in range(1,len(Dc))]
        frows.append((g,(Nv-r*Dv)/(r*Dv)))   # want sum g m = -(this) + O(1e-16)
    # embedding lattice
    B=[float(abs(v))+1 for v in m_true] if len(sys.argv)>5 else None
    bound=[mp.mpf(10)**int(sys.argv[5])]*nu if len(sys.argv)>5 else [mp.mpf(1e15)]*nu
    Wp=mp.mpf(2)**int(np.ceil(np.log2(1/max(sig,1e-4))))*64   # phase scale
    Wf=mp.mpf(10)**16*64
    ne=len(rows); nf=len(frows); dim=nu+ne+1; cols=nu+ne+nf+1
    M=[[0]*cols for _ in range(dim)]
    SC=mp.mpf(2)**60
    for k in range(nu):
        M[k][k]=int(SC/bound[k])
        for e,(g,_) in enumerate(rows): M[k][nu+e]=int(mp.nint(SC*Wp*g[k]/1e3))
        for f,(g,_) in enumerate(frows): M[k][nu+ne+f]=int(mp.nint(SC*Wf*g[k]/1e3))
    for e in range(ne): M[nu+e][nu+e]=int(mp.nint(SC*Wp/1e3))
    t=nu+ne
    for e in range(ne): M[t][nu+e]=int(mp.nint(-SC*Wp*phi[e]/1e3))
    for f,(g,v) in enumerate(frows): M[t][nu+ne+f]=int(mp.nint(SC*Wf*v/1e3))
    M[t][cols-1]=int(SC)
    A=IntegerMatrix.from_matrix(M)
    LLL.reduction(A)
    BKZ.reduction(A,BKZ.Param(block_size=int(__import__("os").environ.get("BKZ","20"))))
    best=None
    for i in range(A.nrows):
        row=[A[i,j] for j in range(A.ncols)]
        if abs(row[-1])==int(SC):
            sgn=1 if row[-1]>0 else -1
            m=[sgn*mp.mpf(row[k])/int(SC/bound[k]) for k in range(nu)]
            best=[int(round(float(v))) for v in m]; break
    print('rec  m:',best)
    if best: print('match:',best==m_true, ' max |err| =',max(abs(a-b) for a,b in zip(best,m_true)))
