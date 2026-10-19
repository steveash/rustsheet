# T-1941 W6: Hough search for straight unaliased line tracks F(x) = a + b*(x-1.75) in the per-segment peak table
# (alias(F) observed). Reports (a mod 1, b), support (#segments) and x-range, then removes the explained peaks.
import sys, numpy as np
d=np.loadtxt(sys.argv[1]); d=d[d[:,2]>float(sys.argv[2])]
x=d[:,0]-1.75; f=d[:,1]; w=d[:,2]; tol=0.0012
def alias(F): F=np.mod(F,1.0); return np.minimum(F,1-F)
used=np.zeros(len(f),bool)
for it in range(int(sys.argv[3])):
    best=(0,)
    for b in np.arange(-4,4.0001,0.0025):
        # each unused peak votes for a = +-f - b x (mod 1)
        a1=np.mod(f-b*x,1.0); a2=np.mod(-f-b*x,1.0)
        A=np.concatenate([a1[~used],a2[~used]]); W=np.concatenate([w[~used],w[~used]])
        H,edges=np.histogram(A,bins=2000,range=(0,1),weights=W)
        H2=H+np.roll(H,1)
        j=np.argmax(H2)
        if H2[j]>best[0]: best=(H2[j],b,edges[j])
    _,b,a=best
    m=(np.abs(alias(a+b*x)-f)<tol)&~used
    # refine by LS on unaliased reconstruction
    used|=m; xs=x[m]+1.75
    print('track a(1.75)=%.5f b=%+.4f  n=%d  x %.3f..%.3f  mean amp %.2f'%(a,b,m.sum(),xs.min(),xs.max(),w[m].mean()))
