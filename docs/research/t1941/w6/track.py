# T-1941 W6: link sweep peaks (x, aliased f, amp) into tracks of the UNALIASED frequency F(x) (reflections at 0 and
# 0.5 unwrapped by slope continuity). Output per track: points, quadratic fit of F around 1.75, x-range, mean amp.
import sys, numpy as np
d=np.loadtxt(sys.argv[1]); d=d[d[:,2]>float(sys.argv[2])]
xs=np.unique(d[:,0]); byx={x:d[d[:,0]==x] for x in xs}
def alias(F): F=np.mod(F,1.0); return np.minimum(F,1-F)
tracks=[]  # each: list of (x, F_unaliased, amp)
active=[]
for ix,x in enumerate(xs):
    P=byx[x]; used=np.zeros(len(P),bool)
    # predict
    cand=[]
    for t in active:
        pts=t['pts']
        if len(pts)>=3:
            X=np.array([p[0] for p in pts[-6:]]); Fv=np.array([p[1] for p in pts[-6:]]); co=np.polyfit(X-X[-1],Fv,1 if len(pts)<8 else 2)
            Fp=np.polyval(co,x-X[-1]); tol=0.004
        elif len(pts)==2:
            Fp=pts[-1][1]+(pts[-1][1]-pts[-2][1])/(pts[-1][0]-pts[-2][0])*(x-pts[-1][0]); tol=0.006
        else:
            Fp=None; tol=0.006
        for j in range(len(P)):
            if Fp is None:
                for sgn in (1,-1):
                    for base in (0,1):
                        Fc=base+sgn*P[j,1] if base else sgn*P[j,1]
                        if abs(Fc-pts[-1][1])<tol: cand.append((abs(Fc-pts[-1][1]),id(t),j,Fc))
            else:
                # unaliased candidates near Fp
                k=np.floor(Fp)
                for Fc in (k+P[j,1],k-P[j,1],k+1-P[j,1],k+1+P[j,1],k-1+P[j,1]):
                    if abs(Fc-Fp)<tol: cand.append((abs(Fc-Fp),id(t),j,Fc))
    cand.sort(); took=set(); byid={id(t):t for t in active}
    for dist,tid,j,Fc in cand:
        if tid in took or used[j]: continue
        took.add(tid); used[j]=True; byid[tid]['pts'].append((x,Fc,P[j,2])); byid[tid]['miss']=0
    for t in active:
        if id(t) not in took: t['miss']+=1
    done=[t for t in active if t['miss']>3]; tracks+=done; active=[t for t in active if t['miss']<=3]
    for j in range(len(P)):
        if not used[j]: active.append({'pts':[(x,P[j,1],P[j,2])],'miss':0})
tracks+=active
tracks=[t for t in tracks if len(t['pts'])>=int(sys.argv[3])]
tracks.sort(key=lambda t:-len(t['pts']))
import json
out=[]
for t in tracks:
    X=np.array([p[0] for p in t['pts']]); Fv=np.array([p[1] for p in t['pts']]); A=np.array([p[2] for p in t['pts']])
    # normalise sign so F>=0 at the start
    if Fv[0]<0: Fv=-Fv
    co=np.polyfit(X-1.75,Fv,2); res=Fv-np.polyval(co,X-1.75)
    out.append({'x':X.tolist(),'F':Fv.tolist(),'amp':A.tolist()})
    print('n=%3d x %.4f..%.4f  F(1.75)=%.5f F\'=%+.5f F\'\'=%+.4f rms %.5f amp %.2f'%(len(X),X[0],X[-1],co[2],co[1],2*co[0],res.std(),A.mean()))
json.dump(out,open(sys.argv[4],'w'))
