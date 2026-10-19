# T-1941 W6: model-free track growing across long runs (precise per-run peaks, runs ~0.02 apart in x).
# Seeds: peaks (a,b,c) in runs i,i+1,i+2 with unaliased values a, +-b+k, +-c+k forming a smooth quadratic (|F'| <= 8).
# Growth: extrapolate (cubic once >= 4 points) to the next run; accept the closest alias within tol.
# Output: tracks = list of (run indices, unaliased F). Usage: grow.py PEAKS.npz OUT.json tol mintrack
import sys, json, numpy as np
z=np.load(sys.argv[1]); nr=len([k for k in z.files if k.startswith('x')])
X=np.array([z['x%d'%i][0] for i in range(nr)]); P=[z['p%d'%i][:,0] for i in range(nr)]; A=[z['p%d'%i][:,1] for i in range(nr)]
tol=float(sys.argv[3]); minlen=int(sys.argv[4])
def cands(f,near,win):
    out=[]
    for s in (1,-1):
        base=s*f; k=np.round(near-base); 
        for kk in (k-1,k,k+1):
            v=base+kk
            if abs(v-near)<=win: out.append(v)
    return out
def extend(tr,direction):
    while True:
        idx=[t[0] for t in tr]; F=[t[1] for t in tr]
        nxt=idx[-1]+1 if direction>0 else idx[0]-1
        if nxt<0 or nxt>=nr: return tr
        use=tr[-4:] if direction>0 else tr[:4]
        xs=np.array([X[t[0]] for t in use]); fs=np.array([t[1] for t in use])
        co=np.polyfit(xs-X[nxt],fs,min(len(use)-1,3)); pred=co[-1]
        best=None
        for j,f in enumerate(P[nxt]):
            for v in cands(f,pred,0.01):
                if abs(v-pred)<tol and (best is None or abs(v-pred)<abs(best[1]-pred)): best=(j,v)
        if best is None: return tr
        if direction>0: tr=tr+[(nxt,best[1],best[0])]
        else: tr=[(nxt,best[1],best[0])]+tr
used=[set() for _ in range(nr)]; tracks=[]
for i in range(nr-2):
    for ja,fa in enumerate(P[i]):
        if ja in used[i]: continue
        found=None
        for jb,fb in enumerate(P[i+1]):
            if jb in used[i+1]: continue
            for vb in cands(fb,fa,8*(X[i+1]-X[i])):
                pred=vb+(vb-fa)*(X[i+2]-X[i+1])/(X[i+1]-X[i])
                for jc,fc in enumerate(P[i+2]):
                    if jc in used[i+2]: continue
                    for vc in cands(fc,pred,0.05):
                        tr=[(i,fa,ja),(i+1,vb,jb),(i+2,vc,jc)]
                        tr=extend(tr,1)
                        if len(tr)>=minlen and (found is None or len(tr)>len(found)): found=tr
        if found:
            tr=extend(found,-1)
            for (r,v,j) in tr: used[r].add(j)
            tracks.append(tr)
json.dump([{'run':[t[0] for t in tr],'x':[float(X[t[0]]) for t in tr],'F':[float(t[1]) for t in tr],'amp':[float(A[t[0]][t[2]]) for t in tr]} for tr in tracks],open(sys.argv[2],'w'))
print('tracks',len(tracks),'lengths',sorted([len(t) for t in tracks],reverse=True)[:30])
