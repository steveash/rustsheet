# Per-cluster hand-off rule on W2's 1,024 dense clusters (512 consecutive doubles each, x0 = 2 + (k+0.5+0.2357 frac(0.618034 k))/1024):
# profiles R(delta mod 2) for N (s4) and D (d3) on the 1/8 grid, best rule, gain; saved for the global fit. Then the
# neighbour test: are the rules of adjacent clusters (~0.001 apart) similar (smooth delta) or unrelated?
import sys, numpy as np, time, csv
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def tail(c,k,s,x):
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
def rn(v,g,odd):
    q=v/g; f=np.floor(q); r=q-f; fo=np.mod(f,2)==1
    up=(r>0.5)|((r==0.5)&(fo if not odd else ~fo)); return (f+up)*g
DELTA=np.arange(16)/8.0
def variant(v,g,delta):
    k=np.floor(delta+0.5); f=delta-k; return rn(v+f*g,g,bool(int(k)%2))
rows=[]; t0=time.time()
for nn in range(1,17):
    for h in 'ab':
        case='q2%02d%s'%(nn,h)
        try: X0,Y0=load(case)
        except Exception as e: print("missing",case,e); continue
        br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
        for a,b in zip(br,br[1:]):
            if b-a<256: continue
            X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2)); n=b-a; x=float(X[n//2])
            s=LD(P0[0])+0*X
            for i in range(1,4): s=sd(sd(s*X)+LD(P0[i]))
            v4=sd(s*X)+LD(P0[4]); s4=sd(v4); u=ulp(s4)
            d=LD(Q0[0])+0*X
            for i in range(1,3): d=sd(sd(d*X)+LD(Q0[i]))
            v3=sd(d*X)+LD(Q0[3]); d3=sd(v3); uD=ulp(d3)
            def yof(s4v,d3v): return sd(t*sd(tail(P0,4,s4v,X)/tail(Q0,3,d3v,X))).astype(np.float64)
            pN=np.array([np.mean(yof(variant(v4,u,dl),d3)==Y) for dl in DELTA])
            kN=int(np.argmax(pN)); pD=np.array([np.mean(yof(variant(v4,u,DELTA[kN]),variant(v3,uD,dl))==Y) for dl in DELTA])
            kD=int(np.argmax(pD))
            k=int(round((x-2)*1024-0.5))   # approximate cluster id
            rows.append((k,case,a,n,x,pN[0],DELTA[kN],pN[kN],DELTA[kD],pD[kD],pN,pD))
print(f"{len(rows)} clusters in {time.time()-t0:.0f}s", flush=True)
rows.sort()
import pickle; pickle.dump(rows,open('clusters.pkl','wb'))
with open('clusters.csv','w') as fh:
    w=csv.writer(fh); w.writerow(['k','case','row0','n','x','stored','deltaN_mod2','rateN','deltaD_mod2','rateND'])
    for r in rows: w.writerow([r[0],r[1],r[2],r[3],f"{r[4]:.9f}",f"{r[5]:.4f}",f"{r[6]:.3f}",f"{r[7]:.4f}",f"{r[8]:.3f}",f"{r[9]:.4f}"])
x=np.array([r[4] for r in rows]); st=np.array([r[5] for r in rows]); dN=np.array([r[6] for r in rows]); rN=np.array([r[7] for r in rows]); dD=np.array([r[8] for r in rows]); rD=np.array([r[9] for r in rows])
gN=rN-st; gD=rD-rN
print(f"stored mean {st.mean():.3f}; N-rule gain mean {gN.mean():.3f} (median {np.median(gN):.3f}, >0.02 on {np.mean(gN>0.02):.2f}); +D gain mean {gD.mean():.3f}")
print("delta_N mod 2 histogram (bins of 1/8):", np.bincount((dN*8).astype(int),minlength=16))
def circ(a,b): d=np.mod(a-b,2.0); return np.minimum(d,2-d)
sel=gN>=0.02; idx=np.where(sel)[0]; adj=[(i,j) for i,j in zip(idx[:-1],idx[1:]) if j==i+1]
if adj:
    da=np.array([circ(dN[i],dN[j]) for i,j in adj]); rng=np.random.default_rng(0); dr=circ(dN[rng.choice(idx,2000)],dN[rng.choice(idx,2000)])
    print(f"adjacent informative clusters (n={len(adj)}, spacing ~0.001): mean circular |d delta| {da.mean():.3f}, P(same rule) {np.mean(da<1e-9):.2f}; random pairs: {dr.mean():.3f}, {np.mean(dr<1e-9):.2f}")
