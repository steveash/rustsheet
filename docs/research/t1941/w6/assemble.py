# T-1941 W6: assemble a raw-x (n,d) chain for R from (1) exact function rows N(x) - R(x)D(x) = 0 on [1.5,2] (D0 = 1)
# and (2) stage-law track rows. A track row set is (chain, stage j, law, sign, int, gexp, x[], Fobs[]): it says the
# unaliased line frequency sign*Fobs + int equals s_j'(x) / 2^E, where E = exponent of the stage value (law 'S' or 'P'),
# or E+1 (law 'T'), and E is derived from the solution itself (fixed point; gexp = global exponent offset for a D0 = 2^k
# normalisation). Linear least squares in mpmath; iterate E. Returns the chain and per-track residuals.
import numpy as np, mpmath as mp
mp.mp.dps=60
def Rt(x): return mp.loggamma(x)/((x-1)*(x-2)) if x!=2 else 1-mp.euler
def stage_val_der(c,j,x):
    """s_j(x) and s_j'(x) as linear forms in c (lists of coefficient weights), Horner sums S_j = sum_{k>=j} c_k x^(k-j)."""
    n=len(c)-1
    val=[mp.mpf(0)]*(n+1); der=[mp.mpf(0)]*(n+1)
    for k in range(j,n+1):
        val[k]=x**(k-j)
        if k>j: der[k]=(k-j)*x**(k-j-1)
    return val,der
def solve(n,d,tracks,K=60,wf=1e16,wt=1e4,iters=4,E0=None,ridge=0.0,c0=None):
    xs=[mp.mpf(1.75)+mp.mpf(0.25)*mp.cos(mp.pi*(k+mp.mpf(1)/2)/K) for k in range(K)]
    nun=n+1+d   # N0..Nn, D1..Dd
    E={i:(E0[i] if E0 else None) for i in range(len(tracks))}
    sol=None
    for it in range(iters):
        rows=[]; rhs=[]
        for x in xs:
            r=Rt(x); w=wf/r
            rows.append([x**j*w for j in range(n+1)]+[-r*x**j*w for j in range(1,d+1)]); rhs.append(r*w)
        for ti,t in enumerate(tracks):
            if E[ti] is None: continue
            ch,j,law=t['chain'],t['j'],t['law']; Eexp=E[ti]+(1 if law=='T' else 0)+t.get('gexp',0)
            for x,F in zip(t['x'],t['F']):
                x=mp.mpf(x); target=(t['sign']*mp.mpf(F)+t['int'])*mp.mpf(2)**Eexp
                cdeg=n if ch=='N' else d
                _,der=stage_val_der([0]*(cdeg+1),j,x)
                row=[mp.mpf(0)]*nun
                if ch=='N':
                    for k in range(cdeg+1): row[k]=der[k]
                else:
                    for k in range(1,cdeg+1): row[n+k]=der[k]
                w=wt/abs(target) if target!=0 else wt
                rows.append([v*w for v in row]); rhs.append(target*w)
        if ridge and c0 is not None:
            for k in range(nun):
                row=[mp.mpf(0)]*nun; row[k]=mp.mpf(ridge)/abs(mp.mpf(c0[k])); rows.append(row); rhs.append(mp.mpf(ridge)*mp.sign(c0[k]))
        A=mp.matrix(rows); b=mp.matrix(rhs)
        sol=mp.qr_solve(A,b)[0]
        N=[sol[k] for k in range(n+1)]; D=[mp.mpf(1)]+[sol[n+k] for k in range(1,d+1)]
        # update E from the stage values of the solution at each track's midpoint
        for ti,t in enumerate(tracks):
            c=N if t['chain']=='N' else D; xm=mp.mpf(np.median(t['x']))
            v,_=stage_val_der(c,t['j'],xm); sv=sum(a*b_ for a,b_ in zip(v,c))
            if t['law']=='P': sv=sv-c[t['j']]   # P_j = S_j - c_j
            E[ti]=int(mp.floor(mp.log(abs(sv),2)))-t.get('gexp',0)
    # residuals per track
    res=[]
    for ti,t in enumerate(tracks):
        c=N if t['chain']=='N' else D; Eexp=E[ti]+(1 if t['law']=='T' else 0)+t.get('gexp',0); r=[]
        for x,F in zip(t['x'],t['F']):
            x=mp.mpf(x); _,der=stage_val_der(c,t['j'],x); sd=sum(a*b_ for a,b_ in zip(der,c))
            r.append(float(sd/mp.mpf(2)**Eexp-(t['sign']*mp.mpf(F)+t['int'])))
        res.append(np.sqrt(np.mean(np.square(r))))
    return [float(v) for v in N],[float(v) for v in D],res,E
