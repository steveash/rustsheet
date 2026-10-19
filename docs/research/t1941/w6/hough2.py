# T-1941 W6: Hough seeds (straight unaliased tracks) refined iteratively into cubic F(x) models; each track's peaks
# are unwrapped to unaliased F. Output tracks.json: list of {coef (poly in x-1.75, highest first), x, F, amp}.
import sys, json, numpy as np
d=np.loadtxt(sys.argv[1]); d=d[d[:,2]>float(sys.argv[2])]
x=d[:,0]-1.75; f=d[:,1]; w=d[:,2]
def alias(F): F=np.mod(F,1.0); return np.minimum(F,1-F)
def unwrap(Fm,fo):
    k=np.floor(Fm); c=np.stack([k+fo,k-fo,k+1-fo,k+1+fo,k-1+fo,k+2-fo])
    return c[np.argmin(np.abs(c-Fm),0),np.arange(len(fo))]
used=np.zeros(len(f),bool); out=[]
for it in range(int(sys.argv[3])):
    best=(0,)
    for b in np.arange(-6,6.0001,0.0025):
        a1=np.mod(f-b*x,1.0); a2=np.mod(-f-b*x,1.0)
        A=np.concatenate([a1[~used],a2[~used]]); W=np.concatenate([w[~used],w[~used]])
        H,edges=np.histogram(A,bins=2000,range=(0,1),weights=W); H2=H+np.roll(H,1); j=np.argmax(H2)
        if H2[j]>best[0]: best=(H2[j],b,edges[j])
    _,b,a=best; co=np.array([b,a]); tol=0.0015
    for rnd in range(6):
        Fm=np.polyval(co,x); m=(np.abs(alias(Fm)-f)<tol)&~used
        if m.sum()<8: break
        Fu=unwrap(Fm[m],f[m]); deg=min(3,1+rnd//2)
        co=np.polyfit(x[m],Fu,deg,w=w[m])
    Fm=np.polyval(co,x); m=(np.abs(alias(Fm)-f)<0.0012)&~used
    if m.sum()<15: used|=m; continue
    Fu=unwrap(Fm[m],f[m]); res=Fu-np.polyval(co,x[m])
    used|=m
    if co[-1]<0: co=-co; Fu=-Fu
    out.append({'coef':co.tolist(),'x':(x[m]+1.75).tolist(),'F':Fu.tolist(),'amp':w[m].tolist()})
    print('T%-2d n=%3d x %.3f..%.3f  F(1.75)=%.5f  dF=%+.5f d2F=%+.4f d3F=%+.3f rms %.5f amp %.2f'%(len(out)-1,m.sum(),x[m].min()+1.75,x[m].max()+1.75,
          np.polyval(co,0),np.polyval(np.polyder(co),0),np.polyval(np.polyder(co,2),0),np.polyval(np.polyder(co,3),0) if len(co)>3 else 0,res.std(),w[m].mean()))
json.dump(out,open(sys.argv[4],'w'))
