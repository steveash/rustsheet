"""T-1941 W1 exact coordinate search. Usage: python3 c1search5.py in.pkl out.pkl <|x-1| window, e.g. 1e-3 or 2400 for all> <npts> <sweeps>
pkl = (N, D) ascending coefficients."""
import numpy as np, pickle, sys
from exsearch import res_of
from csim import simA
from load2 import load_runs
R=load_runs()
X=[];B=[]
for k,a in R.items():
    x=a[:,0]; b=a[:,1]; s=(np.abs(x-1)<float(sys.argv[3]))&(x!=1)&np.isfinite(b)
    if s.any(): X.append(x[s]); B.append(b[s])
x=np.concatenate(X); b=np.concatenate(B); x,ui=np.unique(x,return_index=True); b=b[ui]
rng=np.random.default_rng(1); sub=rng.choice(len(x),min(len(x),int(sys.argv[4])),replace=False); xs=x[sub]; bs=b[sub]
N,D=pickle.load(open(sys.argv[1],'rb')); N=np.array(N,float); D=np.array(D,float)
def sc(N,D,xx=xs,bb=bs): return np.mean(res_of(xx,simA(xx,N,D))==bb)
nD=len(D)-1; nN=len(N)-1
best=sc(N,D); print('start %.4f (n=%d)'%(best,len(xs)),flush=True)
def try_moves(moves):
    global N,D,best
    for mv in moves:
        N2,D2=mv(N.copy(),D.copy()); s=sc(N2,D2)
        if s>best: best=s; N,D=N2,D2
def psums(c):
    d=c[::-1]; s=d[0]; out=[s]
    for a in d[1:]: s=float(np.longdouble(s)+np.longdouble(a)); out.append(s)
    return out
PSD=psums(D); PSN=psums(N)        # partial sums S_j(1): stage moves are in each stage's own ulp
def stage_move(chain,jidx,delta):
    def f(N,D):
        if chain=='D': D[nD-jidx]+=delta*np.spacing(abs(PSD[jidx]))
        else: N[nN-jidx]+=delta*np.spacing(abs(PSN[jidx]))
        return N,D
    return f
def coef_move(chain,i,k):
    def f(N,D):
        arr=D if chain=='D' else N; v=arr[i]
        for _ in range(abs(k)): v=np.nextafter(v,np.inf if k>0 else -np.inf)
        arr[i]=v; return N,D
    return f
def p0_move(k):
    def f(N,D): N[0]+=k*np.spacing(abs(N.sum()))/8; return N,D
    return f
compose=lambda a,b_: (lambda N,D: b_(*a(N,D)))
for sw in range(int(sys.argv[5])):
    try_moves([stage_move(ch,j,d) for ch,js in [('D',range(1,nD)),('N',range(1,nN))] for j in js for d in [-3,-2,-1,1,2,3]])
    print(' sw%d stage moves %.4f'%(sw,best),flush=True)
    try_moves([compose(stage_move('D',jd,dd),stage_move('N',jn,dn)) for jd in range(1,nD) for jn in range(1,nN) for dd in [-2,-1,1,2] for dn in [-3,-2,-1,1,2,3]])
    print(' sw%d pair moves %.4f'%(sw,best),flush=True)
    pm=[]
    for ch,js in [('D',list(range(1,nD))),('N',list(range(1,nN)))]:
        for i1 in range(len(js)):
            for i2 in range(i1+1,len(js)):
                for d1 in [-2,-1,1,2]:
                    for d2 in [-2,-1,1,2]: pm.append(compose(stage_move(ch,js[i1],d1),stage_move(ch,js[i2],d2)))
    try_moves(pm); print(' sw%d same-chain pairs %.4f'%(sw,best),flush=True)
    try_moves([coef_move(ch,i,k) for ch,idxs in [('D',[1,2,3,4,5]),('N',[1,2,3,4])] for i in idxs for k in [-8,-4,-2,-1,1,2,4,8]])
    print(' sw%d coef moves %.4f'%(sw,best),flush=True)
    try_moves([p0_move(k) for k in range(-16,17) if k]); print(' sw%d p0 %.4f'%(sw,best),flush=True)
print('full window %.4f'%sc(N,D,x,b))
pickle.dump((N,D),open(sys.argv[2],'wb'))
