# T-1941: score a raw-y Horner N/D candidate for GAMMA's G on [1,2) against the g_rows.py dump (GAMMA(x), 1 <= x < 2).
import numpy as np, sys
L=np.longdouble
def st(v): return np.asarray(v,dtype=L).astype(np.float64)
def horner(c,y):
    s=np.full_like(y,c[-1])
    for ci in c[-2::-1]: s=st(L(st(L(s)*L(y)))+L(ci))
    return s
N=[float.fromhex(v) for v in sys.argv[1].split(',')]; D=[float.fromhex(v) for v in sys.argv[2].split(',')]
X=[];G=[]
# rows from g_rows.py: x_hex g_hex case (only cells whose formula is exactly =GAMMA(A<row>))
for l in open(sys.argv[3] if len(sys.argv)>3 else 'g_rows.txt'):
    p=l.split(); X.append(float.fromhex(p[0])); G.append(float.fromhex(p[1]))
y=np.array(X); g=np.array(G)
m=st(L(horner(N,y))/L(horner(D,y)))
e=np.abs(m-g)/np.spacing(np.abs(g))
print(len(y),'points; exact %.4f; within 1 ulp %.4f'%((m==g).mean(),(e<=1).mean()))
for k in range(10):
    s=(y>=1+k/10)&(y<1+(k+1)/10); print(' %.1f: %.3f'%(1+k/10,(m[s]==g[s]).mean()), end='')
print()
