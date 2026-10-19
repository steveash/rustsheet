# T-1941 W6: EM bootstrap from a seed chain. Each round: for every grown track, predict all laws (T/S/Pn1/Pn2, both
# chains, sign, int) with the current chain; accept the best law if its rms < thr and it is unique by a margin;
# re-solve function + ridge + all accepted tracks; tighten thr. Usage: emtracks.py TRACKS.json SEED OUT rounds [truth]
import sys, json, numpy as np
sys.path.insert(0,sys.path[0]); from assemble import solve; from lawrows import rows_for, predict; from synth import read_chain; from stagelaws import stage_F
T=json.load(open(sys.argv[1])); N,D=read_chain(sys.argv[2]); N=list(np.array(N)/D[0]); D=list(np.array(D)/D[0])
n=len(N)-1; N0,D0,_,_=solve(n,n,[],iters=1,K=40); c0=N0+D0[1:]
truth=read_chain(sys.argv[5]) if len(sys.argv)>5 else None
def pts(t):
    sub=np.linspace(0,len(t['x'])-1,min(12,len(t['x']))).astype(int); return (np.array(t['x'])+32768*2.0**-52)[sub],np.array(t['F'])[sub]
laws=[(ch,law,j) for ch in 'ND' for law in ('T','S','Pn1','Pn2') for j in range(n-1 if law[0]=='P' else n)]
thr=float(sys.argv[6]) if len(sys.argv)>6 else 1e-4
for rnd in range(int(sys.argv[4])):
    acc=[]
    for ti,t in enumerate(T):
        x,F=pts(t); best=[]
        for ch,law,j in laws:
            pr=predict(N,D,ch,law,j,x)
            for sg in (1,-1):
                k=np.round(np.median(pr-sg*F)); r=np.sqrt(np.mean((pr-(sg*F+k))**2)); best.append((r,(ch,law,j,sg,int(k))))
        best.sort(key=lambda b:b[0])
        if best[0][0]<thr and best[1][0]>3*best[0][0]: acc.append((ti,best[0][1],best[0][0]))
    rows=[]
    for ti,(ch,law,j,sg,k),r in acc:
        x,F=pts(T[ti]); rows+=rows_for(ch,N if ch=='N' else D,law,j,sg,k,x,F)
    N,D,_,_=solve(n,n,rows,iters=1,ridge=100,c0=c0,K=40)
    msg='round %d thr %.0e: %d tracks accepted %s'%(rnd,thr,len(acc),' '.join('%d:%s.%s%d'%(ti,a[0],a[1],a[2]) for ti,a,r in acc))
    if truth:
        Nt=np.array(truth[0])/truth[1][0]; Dt=np.array(truth[1])/truth[1][0]; xg=np.linspace(1.5,2,11)
        A=stage_F(np.array(N),xg); B=stage_F(Nt,xg); A2=stage_F(np.array(D),xg); B2=stage_F(Dt,xg)
        msg+='\n   stage err N '+' '.join('%.0e'%np.max(np.abs(A['S%d'%j][1]/B['S%d'%j][1]-1)) for j in range(n-1,-1,-1))+' | D '+' '.join('%.0e'%np.max(np.abs(A2['S%d'%j][1]/B2['S%d'%j][1]-1)) for j in range(n-1,-1,-1))
    print(msg,flush=True)
    thr=max(thr/3,2e-6)
open(sys.argv[3],'w').write('N '+' '.join(float(v).hex() for v in N)+'\nD '+' '.join(float(v).hex() for v in D)+'\n')
