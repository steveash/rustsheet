# T-1941 W6: extract Excel's inner stored R on the kernel's range (kern.py, env KERNEL): all doubles R with
# outer(x, R) == B (k6: fl((x-1)*fl((x-2)*R)) on (1.5,2)).
# Writes "x_hex B_hex Rlo_hex Rhi_hex ncand case" (candidates are consecutive doubles Rlo..Rhi).
import sys, numpy as np
sys.path.insert(0,sys.path[0]); from lib6 import *; import kern
X,B,C=load(sys.argv[1],kern.LO,kern.HI); out=open(sys.argv[2],'w')
lo,hi,n=kern.extract(X,B,5)
for i in range(len(X)):
    if n[i]: out.write('%s %s %s %s %d %s\n'%(X[i].hex(),B[i].hex(),lo[i].hex(),hi[i].hex(),n[i],C[i]))
print(len(X),'rows; ncand hist',np.bincount(n))
