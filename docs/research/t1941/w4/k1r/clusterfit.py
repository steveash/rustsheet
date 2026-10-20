# clusterfit: localfit.py with cluster-aware data/start/save (run lists are '+'-joined prefixes, e.g. p145+p146+p147+p148)
import k1lib as K, numpy as np, math, sys, pickle, struct
def nextd(v,n):
    if n==0: return v
    b=struct.unpack('<q',struct.pack('<d',v))[0]; s=1 if v>0 else -1
    return struct.unpack('<d',struct.pack('<q',b+n*s if v!=0 else n))[0]
run=sys.argv[1]; seed=int(sys.argv[2]) if len(sys.argv)>2 else 0
import os; YLO=float(os.environ.get('YLO','0')); YHI=float(os.environ.get('YHI','9')); R=[r for r in K.runs(run.split('+')) if YLO<=r[1][0]<YHI]; nm=run; ST=int(os.environ.get('ST','1')); Y=np.concatenate([r[1][::ST] for r in R]); T=np.concatenate([r[2][::ST] for r in R])
Nd=list(K.N0[::-1]); Dd=list(K.D0[::-1])                     # descending: c[0]=top
if seed:
    rng=np.random.default_rng(seed)
    for c in (Nd,Dd):
        for i in range(1,5): c[i]=nextd(c[i],int(rng.integers(-3,4)))
if os.environ.get('START'): _st=pickle.load(open(os.environ['START'],'rb')); Nd=list(np.array(_st['N'])[::-1]); Dd=list(np.array(_st['D'])[::-1])
def score(Nd,Dd): return K.gsimpy.Sim(np.array(Nd[::-1]),np.array(Dd[::-1])).count(Y,T)
xg=np.linspace(Y.min(),Y.max(),20)
def null_dir(c,k):
    u=[math.ulp(abs(v)) for v in c[:k+1]]
    A=np.array([[x**(k-i)*u[i] for i in range(1,k+1)] for x in xg]); b=-np.array([x**k*u[0] for x in xg])
    return np.concatenate([[1.0],np.linalg.lstsq(A,b,rcond=None)[0]])
cur=score(Nd,Dd); print('%s start %d/%d (%.4f)'%(nm,cur,len(Y),cur/len(Y)),flush=True)
for rnd in range(int(os.environ.get('ROUNDS','4'))):
    for ch in ('N','D'):
        for k in (1,2,3,4):
            for off in range(0,8-k):
                c=Nd if ch=='N' else Dd
                if ch=='D' and off+k>=7: continue          # keep D0 (-1/16) fixed
                d=null_dir(c[off:off+k+1],k)
                for T_,st in ((3000,10),(30,1)):
                    best=(cur,None)
                    for t in range(-T_,T_+1,st):
                        cc=list(c)
                        for i,di in enumerate(d): cc[off+i]=nextd(c[off+i],int(round(t*di)))
                        s=score(cc,Dd) if ch=='N' else score(Nd,cc)
                        if s>best[0]: best=(s,cc)
                    if best[1] is not None:
                        cur=best[0]
                        if ch=='N': Nd=best[1]
                        else: Dd=best[1]
                        c=Nd if ch=='N' else Dd
    print(' round %d: %d/%d (%.4f)'%(rnd,cur,len(Y),cur/len(Y)),flush=True)
    pickle.dump(dict(N=Nd[::-1],D=Dd[::-1],score=cur,n=len(Y),y0=Y[0]),open(os.environ.get('OUT','cf_out.pkl'),'wb'))
