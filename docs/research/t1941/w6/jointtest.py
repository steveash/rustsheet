# T-1941 W6: joint assignment beam search over several precise tracks. Each track has a candidate list (from
# enumlong.py); partial assignments are scored by the worst per-track rms after one joint solve (function + ridge +
# all assigned tracks). Usage: jointtest.py TRACKS.json "ti:candfile,ti:candfile,..." n ridge beam
import sys, json, numpy as np
sys.path.insert(0,sys.path[0]); from assemble import solve; from lawrows import rows_for, predict
T=json.load(open(sys.argv[1])); spec=[(int(a),b) for a,b in (s.split(':') for s in sys.argv[2].split(','))]
n=int(sys.argv[3]); ridge=float(sys.argv[4]); beam=int(sys.argv[5])
N0,D0,_,_=solve(n,n,[],iters=1,K=40); c0=N0+D0[1:]
def pts(ti):
    t=T[ti]; sub=np.linspace(0,len(t['x'])-1,min(10,len(t['x']))).astype(int)
    return (np.array(t['x'])+32768*2.0**-52)[sub],np.array(t['F'])[sub]
P={ti:pts(ti) for ti,_ in spec}; C={ti:[tuple(c) for c in json.load(open(f))] for ti,f in spec}
def score(assign):
    rows=[]
    for ti,cc in assign:
        ch,law,j,sg,kk=cc[:5]; eo=cc[5] if len(cc)>5 else 0
        x,F=P[ti]; rows+=rows_for(ch,N0 if ch=='N' else D0,law,j,sg,kk,x,F,eoff=eo)
    N1,D1,_,_=solve(n,n,rows,iters=1,ridge=ridge,c0=c0,K=40)
    return max(np.sqrt(np.mean((predict(N1,D1,cc[0],cc[1],cc[2],P[ti][0])-(cc[3]*P[ti][1]+cc[4]))**2)) for ti,cc in assign)
B=[((),0.0)]
for ti,_ in spec:
    nb=[]
    for a,_ in B:
        for c in C[ti]:
            aa=a+((ti,c),); nb.append((aa,score(aa)))
    nb.sort(key=lambda t:t[1]); B=nb[:beam]
    print('after track %d: best'%ti,' '.join('%.1e'%s for _,s in B[:6]),flush=True)
for a,s in B[:8]: print('%.2e'%s,[('%d:'%ti+'.'.join(str(v) for v in c)) for ti,c in a])
