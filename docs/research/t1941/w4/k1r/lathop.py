import k1lib as K, numpy as np, sys, pickle, math, itertools
exec(open('latclimb.py').read().split("RUNS=K.runs")[0])
J=jac(K.N0,K.D0); V,f=basis(J); NV=int(sys.argv[2]); V=V[:NV]; f=f[:NV]
if sys.argv[1]=='all':
    x,g,c=K.lib2.g_rows(); m=(x>=1)&(x<2); x,g=x[m],g[m]; x,ui=np.unique(x,return_index=True); g=g[ui]
    rs=np.random.default_rng(7); per=int(sys.argv[3]); sel=[]
    for t in range(10):
        ii=np.where((x>=1+t/10)&(x<1+(t+1)/10))[0]; sel.append(rs.choice(ii,min(per,len(ii)),replace=False))
    sel=np.concatenate(sel); Y=x[sel]; T=g[sel]; TEN=np.floor((Y-1)*10).astype(int)
else:
    RUNS=K.runs(sys.argv[1].split(','))
    Y=np.concatenate([r[1][::int(sys.argv[3])] for r in RUNS]); T=np.concatenate([r[2][::int(sys.argv[3])] for r in RUNS])
if len(sys.argv)>5 and sys.argv[5]!='excel':
    ctl=pickle.load(open(sys.argv[5],'rb')); Nt,Dt=apply(K.N0,K.D0,ctl['v']); T=K.gsimpy.Sim(Nt,Dt).G(Y)
    kt=np.linalg.lstsq(V.T.astype(float),ctl['v'].astype(float),rcond=None)[0]; print('truth in these coords (lstsq):',np.round(kt,2))
FT=0.05; KICK=int(__import__("os").environ.get("KICK","3"))
cache={}
FRZ=[int(t) for t in __import__('os').environ.get('FRZ','').split(',') if t]
def score(k):
    if any(k[j]!=0 for j in FRZ): return -1
    key=tuple(k)
    if key not in cache:
        v=k@V
        cache[key]=-1 if np.max(np.abs(J@v))>FT else K.gsimpy.Sim(*apply(K.N0,K.D0,v)).count(Y,T)
    return cache[key]
def local(k):
    cur=score(k)
    while True:
        best=(cur,None)
        moves=[(j,s) for j in range(NV) for s in (1,-1,2,-2)]
        for j,s in moves:
            kk=k.copy(); kk[j]+=s; sc=score(kk)
            if sc>best[0]: best=(sc,kk)
        for a,b in itertools.combinations(range(NV),2):
            for sa,sb in itertools.product((1,-1),repeat=2):
                kk=k.copy(); kk[a]+=sa; kk[b]+=sb; sc=score(kk)
                if sc>best[0]: best=(sc,kk)
        if best[1] is None: return k,cur
        k,cur=best[1],best[0]
rng=np.random.default_rng(0); k=np.zeros(NV,int); k,cur=local(k); gbest=(cur,k.copy())
print('local from AN2: %d / %d  k=%s'%(cur,len(Y),k),flush=True)
for hop in range(int(sys.argv[4])):
    kk=gbest[1].copy(); j=rng.choice(NV,rng.integers(1,4),replace=False); kk[j]+=rng.integers(-KICK,KICK+1,len(j))
    kk,sc=local(kk)
    if sc>gbest[0]: gbest=(sc,kk.copy()); print('hop %d NEW BEST %d (%.4f) k=%s'%(hop,sc,sc/len(Y),kk),flush=True)
print('final',gbest[0],gbest[1])
if sys.argv[1]=='all':
    for lab,kk in (('AN2',np.zeros(NV,int)),('best',gbest[1])):
        G=K.gsimpy.Sim(*apply(K.N0,K.D0,kk@V)).G(Y); ok=(G==T)
        print(lab,' '.join('%.3f'%ok[TEN==t].mean() for t in range(10)),' all %.4f'%ok.mean())
pickle.dump(gbest,open(sys.argv[6] if len(sys.argv)>6 else 'lathop_best.pkl','wb'))
