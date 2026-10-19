# T-1941 W6: observed R-residual peaks per long run (top-K) vs. predicted stage-line frequencies of a raw-x Horner
# candidate R = N/D: product P_j = s_{j+1}*x (law P'h/ulp(P)), sum S_j = P_j + c_j (sawtooth S'h/ulp(S) and triangle
# S'h/(2ulp(S))). Reports, for each candidate, how many strong peaks are explained within tol.
import sys, numpy as np, mpmath as mp, collections
sys.path.insert(0,sys.path[0]); from rfit import fit
mp.mp.dps=60
def alias(f): f=float(f)%1.0; return min(f,1-f)
def peaks(path,runs,K=12):
    rows=collections.defaultdict(list)
    for l in open(path):
        p=l.split()
        if p[5] in runs: rows[p[5]].append((float.fromhex(p[0]),float.fromhex(p[2]),float.fromhex(p[3])))
    Rf=lambda z: mp.loggamma(z)/((z-1)*(z-2)); out={}
    for cs in runs:
        v=np.array(rows[cs]); n=1<<int(np.log2(len(v))); v=v[:n]; x=v[:,0]; Rm=(v[:,1]+v[:,2])/2; u=np.spacing(v[:,1])
        x0=mp.mpf(x[0]); g=[mp.diff(Rf,x0,k)/mp.factorial(k) for k in range(3)]
        e=np.array([float((mp.mpf(Rm[i])-(g[0]+(mp.mpf(x[i])-x0)*(g[1]+(mp.mpf(x[i])-x0)*g[2])))/mp.mpf(u[i])) for i in range(n)])
        F=np.abs(np.fft.rfft(e-e.mean()))/n*2; fr=np.fft.rfftfreq(n)
        top=[j for j in np.argsort(F)[::-1] if j>2][:K]; out[cs]=(float(x0),[(fr[j],F[j]) for j in top])
    return out
def predict(N,D,x0):
    """N,D ascending mp coefficients (already as doubles' values). Return list of (label,freq)."""
    x=mp.mpf(x0); h=mp.mpf(2)**-52; res=[]
    for nm,c in (('N',N),('D',D)):
        n=len(c)-1; s=c[n]; sd=mp.mpf(0)
        for j in range(n-1,-1,-1):
            P=s*x; Pd=sd*x+s
            S=P+c[j]; Sd=Pd
            uP=mp.mpf(np.spacing(abs(float(P)))); uS=mp.mpf(np.spacing(abs(float(S))))
            res.append(('%s.P%d'%(nm,j),alias(Pd*h/uP))); res.append(('%s.S%d'%(nm,j),alias(Sd*h/uS))); res.append(('%s.T%d'%(nm,j),alias(Sd*h/(2*uS))))
            s,sd=S,Sd
    return res
if __name__=='__main__':
    runs=['kmyst-gam-r711:B','kmyst-gam-p201:B','kmyst-gam-r712:B','kmyst-gam-p202:B','kmyst-gam-r111:B','kmyst-gam-p203:B','kmyst-gam-r713:B','kmyst-gam-r714:B','kmyst-gam-p204:B']
    pk=peaks(sys.argv[1],runs); tol=float(sys.argv[2])
    import pickle; pickle.dump(pk,open(sys.argv[1]+'.peaks.pkl','wb'))
    for spec in sys.argv[3:]:
        n,d,lo,hi,sc=spec.split(','); a,b,e=fit(int(n),int(d),mp.mpf(lo),mp.mpf(hi))
        sc=mp.mpf(sc); a=[mp.mpf(float(v*sc)) for v in a]; b=[mp.mpf(float(v*sc)) for v in b]
        hit=0; tot=0; names=collections.Counter()
        for cs,(x0,pl) in pk.items():
            pr=predict(a,b,x0)
            for f,A in pl:
                tot+=1; m=[nm for nm,pf in pr if abs(pf-f)<tol]
                if m: hit+=1; names.update(m)
        print('%s: %d/%d peaks matched; %s'%(spec,hit,tot,names.most_common(8)))
