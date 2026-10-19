# T-1941 W6: extract Excel's inner stored R for (1.5,2): all doubles R with fl(t*fl(m*R)) == B (t = x-1, m = x-2).
# Writes "x_hex B_hex Rlo_hex Rhi_hex ncand case" (candidates are consecutive doubles Rlo..Rhi).
import sys, numpy as np
sys.path.insert(0,sys.path[0]); from lib6 import *
X,B,C=load(sys.argv[1],1.5,2.0); out=open(sys.argv[2],'w')
def nb(v,d):
    for _ in range(abs(d)): v=np.nextafter(v,np.inf if d>0 else -np.inf)
    return v
t=X-1.0; m=X-2.0; R0=B/(t*m)
hit=[]; 
for d in range(-5,6):
    R=nb(R0,d); hit.append(np.where(st(L(t)*L(st(L(m)*L(R))))==B,R,np.nan))
H=np.array(hit); lo=np.nanmin(H,0); hi=np.nanmax(H,0); n=np.sum(~np.isnan(H),0)
for i in range(len(X)):
    if n[i]: out.write('%s %s %s %s %d %s\n'%(X[i].hex(),B[i].hex(),lo[i].hex(),hi[i].hex(),n[i],C[i]))
print(len(X),'rows; ncand hist',np.bincount(n))
