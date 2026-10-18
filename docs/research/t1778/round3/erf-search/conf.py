import sys, numpy as np, bisect
rows=[]
for l in open(sys.argv[1]):
    a=l.split()
    if a[0]!='C': continue
    rows.append((float.fromhex(a[1]),float(a[2]),float(a[3])))
rows.sort(); Z=np.array([r[0] for r in rows]); LO=np.array([r[1] for r in rows]); HI=np.array([r[2] for r in rows])
rel=float(sys.argv[2]) if len(sys.argv)>2 else 1e-6
conf=0; out=[]
for i in range(len(Z)):
    if HI[i]<0 or LO[i]>0:
        a=bisect.bisect_left(Z,Z[i]*(1-rel)); b=bisect.bisect_right(Z,Z[i]*(1+rel))
        mxlo=LO[a:b].max(); mnhi=HI[a:b].min()
        c = mxlo>mnhi
        conf+=c; out.append((Z[i],LO[i],HI[i],mxlo,mnhi,b-a,c))
print('violated',len(out),'locally conflicting (window rel %g)'%rel,conf)
for o in out[:15]: print('%.6e lo=%.1f hi=%.1f  window maxlo=%.1f minhi=%.1f n=%d conflict=%d'%o)
